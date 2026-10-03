#!/usr/bin/env bash
set -euo pipefail
[[ "$(uname -s)" == Linux ]] || { echo 'Linux only' >&2; exit 1; }
command -v apt-get >/dev/null || { echo 'Install equivalent GPUI development libraries for your distribution; this helper targets Ubuntu 22.04.' >&2; exit 1; }
privileged=(env)
if (( EUID != 0 )); then privileged=(sudo); fi
"${privileged[@]}" apt-get update
"${privileged[@]}" apt-get install -y --no-install-recommends build-essential clang cmake pkg-config python3 \
  libfontconfig1-dev libfreetype6-dev libxkbcommon-dev libxkbcommon-x11-dev libwayland-dev \
  fonts-dejavu-core libssl-dev libxcb1-dev libx11-dev libvulkan-dev mesa-vulkan-drivers libfuse2 dpkg-dev
