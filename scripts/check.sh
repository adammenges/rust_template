#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname -- "${BASH_SOURCE[0]}")/.."
python3 scripts/rename.py --check
python3 scripts/test_template.py
for script in scripts/*.sh; do bash -n "$script"; done
cargo fmt --all -- --check
cargo test --locked --all-targets
cargo clippy --locked --all-targets -- -D warnings
