#!/usr/bin/env python3
"""Native smoke checks. Requires a graphical session and an already-built binary."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--executable', default='target/debug/native-starter', help='Binary path after building or renaming')
args = parser.parse_args()
root = Path(__file__).resolve().parent.parent
binary = Path(args.executable).resolve()
identity = json.loads((root / 'app.json').read_text())
if sys.platform == 'darwin':
    data = Path.home() / 'Library/Application Support' / identity['storage_name']
elif sys.platform == 'linux':
    if not (os.environ.get('DISPLAY') or os.environ.get('WAYLAND_DISPLAY')):
        parser.error('Linux preview checks require a graphical X11/Wayland session')
    data = Path(os.environ.get('XDG_DATA_HOME', Path.home() / '.local/share')) / identity['storage_name']
else:
    parser.error('Only macOS and Linux have desktop adapters')

def snapshot():
    if not data.exists():
        return None
    # Check the application directory and its files, including diagnostics. Read only.
    result = {}
    for path in [data, *sorted(data.rglob('*'))]:
        stat = path.lstat()
        digest = hashlib.sha256(path.read_bytes()).hexdigest() if path.is_file() and not path.is_symlink() else None
        result[str(path.relative_to(data))] = (stat.st_mtime_ns, stat.st_size, digest)
    return result

before = snapshot()
with tempfile.TemporaryDirectory(prefix='starter-preview-check-') as directory:
    unused = Path(directory) / 'must-not-be-created'
    rejected = subprocess.run([str(binary), '--preview', 'home', '--data-dir', str(unused)], capture_output=True, timeout=15)
    assert rejected.returncode != 0 and not unused.exists(), 'Invalid CLI flags accessed storage'
for pane, width, height in [('home', 480, 480), ('settings', 920, 800), ('error', 1440, 900)]:
    run = subprocess.run([str(binary), '--preview', pane, '--preview-width', str(width),
        '--preview-height', str(height), '--quit-after-ms', '800'], capture_output=True, text=True, timeout=15)
    assert run.returncode == 0, f'{pane}: exit {run.returncode}\n{run.stderr}'
    events = []
    for line in run.stderr.splitlines():
        try:
            events.append(json.loads(line).get('fields', {}).get('event'))
        except ValueError:
            pass
    expected = ['startup', 'application_ready', 'window_opened', 'shutdown_requested', 'shutdown_complete']
    positions = [events.index(event) for event in expected]
    assert positions == sorted(positions), f'{pane}: unexpected lifecycle {events}'
    if sys.platform == 'darwin':
        assert 'native_quit_deferred' in events, f'{pane}: native quit gate was not exercised'
    assert before == snapshot(), f'{pane}: preview mutated user storage'
    print(f'{pane}: {width}x{height}, startup/native quit/shutdown verified; storage unchanged')
print('Native preview smoke checks passed. Screenshots and physical interactions are separate checks.')
