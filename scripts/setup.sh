#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname -- "${BASH_SOURCE[0]}")/.."
if command -v rustup >/dev/null; then
  rustup toolchain install 1.95.0 --profile minimal --component rustfmt --component clippy
  if [[ "$(uname -s)" == Darwin ]]; then rustup target add aarch64-apple-darwin --toolchain 1.95.0; fi
fi
./scripts/doctor.sh
cargo fetch --locked
