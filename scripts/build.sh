#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname -- "${BASH_SOURCE[0]}")/.."
case "$(uname -s)" in
  Darwin) exec ./scripts/build_macos_app.sh "$@" ;;
  Linux) exec ./scripts/build_linux_app.sh "$@" ;;
  *) echo 'No packaging adapter for this host' >&2; exit 1 ;;
esac
