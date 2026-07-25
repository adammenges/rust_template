#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

status=0

check_command() {
  local command="$1"
  if command -v "$command" >/dev/null 2>&1; then
    printf 'ok    %-14s %s\n' "$command" "$(command -v "$command")"
  else
    printf 'miss  %-14s required\n' "$command"
    status=1
  fi
}

check_command cargo
check_command rustc
check_command rustup

if [[ "${OSTYPE:-}" == darwin* ]]; then
  check_command xcode-select
  if xcode-select -p >/dev/null 2>&1; then
    printf 'ok    %-14s %s\n' "xcode tools" "$(xcode-select -p)"
  else
    printf 'miss  %-14s run: xcode-select --install\n' "xcode tools"
    status=1
  fi

  if rustup target list --installed --toolchain 1.95.0 | grep -qx 'aarch64-apple-darwin'; then
    printf 'ok    %-14s %s\n' "macOS target" "aarch64-apple-darwin"
  else
    printf 'miss  %-14s run: ./scripts/setup.sh\n' "macOS target"
    status=1
  fi
elif [[ "$(uname -s)" == "Linux" ]]; then
  check_command file
  check_command pkg-config

  if command -v pkg-config >/dev/null 2>&1; then
    for module in gtk+-3.0 webkit2gtk-4.1 librsvg-2.0; do
      if pkg-config --exists "$module"; then
        printf 'ok    %-14s %s\n' "$module" "$(pkg-config --modversion "$module")"
      else
        printf 'miss  %-14s run: ./scripts/install_linux_dependencies.sh\n' "$module"
        status=1
      fi
    done
  fi

  LINUX_TARGET="$(rustc -vV | sed -n 's/^host: //p')"
  case "$LINUX_TARGET" in
    x86_64-unknown-linux-gnu|aarch64-unknown-linux-gnu)
      if rustup target list --installed --toolchain 1.95.0 | grep -qx "$LINUX_TARGET"; then
        printf 'ok    %-14s %s\n' "Linux target" "$LINUX_TARGET"
      else
        printf 'miss  %-14s run: ./scripts/setup.sh\n' "Linux target"
        status=1
      fi
      ;;
    *)
      printf 'fail  %-14s unsupported host: %s\n' "Linux target" "$LINUX_TARGET"
      status=1
      ;;
  esac
else
  printf 'fail  %-14s macOS and Linux are supported\n' "operating system"
  status=1
fi

if cargo tauri --version >/dev/null 2>&1; then
  printf 'ok    %-14s %s\n' "tauri" "$(cargo tauri --version)"
else
  printf 'miss  %-14s run: ./scripts/setup.sh\n' "tauri"
  status=1
fi

if cargo metadata --locked --no-deps --format-version 1 >/dev/null 2>&1; then
  printf 'ok    %-14s valid and locked\n' "workspace"
else
  printf 'fail  %-14s metadata or Cargo.lock is stale\n' "workspace"
  status=1
fi

exit "$status"
