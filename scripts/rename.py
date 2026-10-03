#!/usr/bin/env python3
"""Validate or rename the single app identity. No credentials or release accounts."""
import argparse
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--check', action='store_true')
parser.add_argument('--package')
parser.add_argument('--executable')
parser.add_argument('--display-name')
parser.add_argument('--identifier')
parser.add_argument('--storage-name')
args = parser.parse_args()
path = ROOT / 'app.json'
try:
    identity = json.loads(path.read_text())
except (OSError, ValueError) as error:
    parser.error(f'cannot read app.json: {error}')
fields = {'package', 'executable', 'display_name', 'identifier', 'storage_name'}
if not isinstance(identity, dict) or set(identity) != fields or not all(isinstance(v, str) for v in identity.values()):
    parser.error('app.json must contain exactly the five identity fields, all strings')
for key in identity:
    value = getattr(args, key, None)
    if value is not None:
        identity[key] = value
for key in ('package', 'executable', 'storage_name'):
    if not re.fullmatch(r'[a-z][a-z0-9-]{0,62}', identity[key]):
        parser.error(f'{key}: use lowercase letters, digits, and hyphens; start with a letter')
if not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9 ._-]{0,63}', identity['display_name']):
    parser.error('display name: use 1–64 ASCII letters, digits, spaces, dots, underscores, or hyphens')
if len(identity['identifier']) > 255 or any(len(part) > 63 for part in identity['identifier'].split('.')) or not re.fullmatch(r'[a-z][a-z0-9-]*(?:\.[a-z][a-z0-9-]*)+', identity['identifier']):
    parser.error('identifier: use a lowercase reverse-DNS application ID')
manifest = ROOT / 'Cargo.toml'
text = manifest.read_text()
expected = text
for pattern, replacement in (
    (r'(?m)^(\[package\]\n)name = "[^\"]+"', rf'\g<1>name = "{identity["package"]}"'),
    (r'(?m)^default-run = "[^\"]+"', f'default-run = "{identity["executable"]}"'),
    (r'(\[\[bin\]\]\n)name = "[^\"]+"', rf'\g<1>name = "{identity["executable"]}"'),
):
    expected, count = re.subn(pattern, replacement, expected)
    if count != 1:
        parser.error('Cargo.toml must contain one package name, default-run, and binary name in the template layout')
if args.check:
    if expected != text or any(getattr(args, k, None) is not None for k in identity):
        parser.error('identity drift or rename options passed to --check')
    print('Identity is valid and Cargo metadata matches app.json')
else:
    path.write_text(json.dumps(identity, indent=2) + '\n')
    manifest.write_text(expected)
    print('Updated app.json and Cargo.toml. Run cargo check to refresh Cargo.lock, then scripts/check.sh.')
