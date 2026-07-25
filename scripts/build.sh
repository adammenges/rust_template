#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"

case "$(uname -s)" in
  Darwin)
    exec "$ROOT_DIR/scripts/build_macos_app.sh" "$@"
    ;;
  Linux)
    exec "$ROOT_DIR/scripts/build_linux_app.sh" "$@"
    ;;
  *)
    echo "error: packaging is supported on macOS and Linux." >&2
    exit 1
    ;;
esac
