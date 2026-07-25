#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "error: Linux system dependencies can only be installed on Linux." >&2
  exit 1
fi

privileged=()
if (( EUID != 0 )); then
  if ! command -v sudo >/dev/null 2>&1; then
    echo "error: rerun as root or install sudo." >&2
    exit 1
  fi
  privileged=(sudo)
fi

if command -v apt-get >/dev/null 2>&1; then
  "${privileged[@]}" apt-get update
  "${privileged[@]}" env DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    build-essential \
    curl \
    file \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libssl-dev \
    libwebkit2gtk-4.1-dev \
    libxdo-dev \
    pkg-config \
    wget
elif command -v dnf >/dev/null 2>&1; then
  "${privileged[@]}" dnf install -y \
    curl \
    file \
    gcc \
    gcc-c++ \
    libappindicator-gtk3-devel \
    librsvg2-devel \
    libxdo-devel \
    make \
    openssl-devel \
    pkgconf-pkg-config \
    webkit2gtk4.1-devel \
    wget
elif command -v pacman >/dev/null 2>&1; then
  "${privileged[@]}" pacman -Syu --needed \
    appmenu-gtk-module \
    base-devel \
    curl \
    file \
    libappindicator-gtk3 \
    librsvg \
    openssl \
    pkgconf \
    webkit2gtk-4.1 \
    wget \
    xdotool
elif command -v zypper >/dev/null 2>&1; then
  "${privileged[@]}" zypper --non-interactive install \
    curl \
    file \
    libappindicator3-1 \
    libopenssl-devel \
    librsvg-devel \
    pkg-config \
    webkit2gtk3-devel \
    wget
  "${privileged[@]}" zypper --non-interactive install --type pattern devel_basis
else
  echo "error: unsupported package manager." >&2
  echo "Install Tauri's Linux prerequisites from https://v2.tauri.app/start/prerequisites/." >&2
  exit 1
fi

echo "Linux system dependencies installed."
