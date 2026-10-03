#!/usr/bin/env python3
"""Exercise identity instantiation and metadata generation in an isolated directory."""
import json
import shutil
import subprocess
import tempfile
from pathlib import Path
ROOT = Path(__file__).resolve().parent.parent
with tempfile.TemporaryDirectory() as directory:
    root = Path(directory)
    (root / 'scripts').mkdir()
    for file in ('rename.py', 'package_metadata.py'):
        shutil.copy(ROOT / 'scripts' / file, root / 'scripts' / file)
    for file in ('app.json', 'Cargo.toml'):
        shutil.copy(ROOT / file, root / file)
    subprocess.run(['python3', str(root / 'scripts/rename.py'), '--package', 'example-workbench',
        '--executable', 'workbench', '--display-name', 'Example Workbench',
        '--identifier', 'org.example.workbench', '--storage-name', 'example-workbench'], check=True)
    subprocess.run(['python3', str(root / 'scripts/rename.py'), '--check'], check=True)
    app = json.loads((root / 'app.json').read_text())
    assert app['executable'] == 'workbench'
    (root / 'app/Contents/MacOS').mkdir(parents=True)
    (root / 'app/Contents/Resources').mkdir()
    (root / 'app/Contents/MacOS/workbench').touch()
    (root / 'app/Contents/Resources/icon.icns').touch()
    for mode in ('macos', 'verify-macos'):
        subprocess.run(['python3', str(root / 'scripts/package_metadata.py'), mode, str(root / 'app')], check=True)
    subprocess.run(['python3', str(root / 'scripts/package_metadata.py'), 'desktop', str(root / 'app.desktop')], check=True)
    assert 'Exec=workbench' in (root / 'app.desktop').read_text()
    identity_path = root / 'app.json'
    manifest_path = root / 'Cargo.toml'
    original_identity, original_manifest = identity_path.read_bytes(), manifest_path.read_bytes()
    def rejected(arguments):
        before = (identity_path.read_bytes(), manifest_path.read_bytes())
        result = subprocess.run(['python3', str(root / 'scripts/rename.py'), *arguments], capture_output=True)
        assert result.returncode != 0, arguments
        assert before == (identity_path.read_bytes(), manifest_path.read_bytes()), 'Rejected input mutated files'
    for key, value in [('storage-name', '../escape'), ('package', 'Bad Name'), ('identifier', 'no-domain'),
                       ('executable', '$(touch unsafe)'), ('display-name', 'Name\nInjected')]:
        rejected([f'--{key}', value])
    rejected(['--check', '--display-name', 'Different'])
    for invalid in [dict(app, extra='unknown'), dict(app, package=42), {}, ['wrong shape']]:
        identity_path.write_text(json.dumps(invalid))
        rejected(['--check'])
    identity_path.write_bytes(original_identity)
    for invalid in [original_manifest.replace(b'default-run =', b'renamed-run ='),
                    original_manifest.replace(b'name = "workbench"', b'name = "drift"')]:
        manifest_path.write_bytes(invalid)
        rejected(['--check'])
    manifest_path.write_bytes(original_manifest)
    subprocess.run(['python3', str(root / 'scripts/rename.py'), '--check'], check=True)
print('Template identity, rejected-input preservation, drift, and native metadata checks passed')
