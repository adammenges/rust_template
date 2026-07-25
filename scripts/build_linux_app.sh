#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"
export LC_ALL=C

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "error: Linux packages must be built on Linux." >&2
  exit 1
fi

for command in cargo file pkg-config rustc rustup; do
  if ! command -v "$command" >/dev/null 2>&1; then
    echo "error: $command is required." >&2
    exit 1
  fi
done

if ! cargo tauri --version >/dev/null 2>&1; then
  echo "error: Tauri CLI is required. Run: ./scripts/setup.sh" >&2
  exit 1
fi

missing_modules=()
for module in gtk+-3.0 webkit2gtk-4.1 librsvg-2.0; do
  if ! pkg-config --exists "$module"; then
    missing_modules+=("$module")
  fi
done

if (( ${#missing_modules[@]} > 0 )); then
  echo "error: missing Linux development libraries: ${missing_modules[*]}" >&2
  echo "Install them with: ./scripts/install_linux_dependencies.sh" >&2
  exit 1
fi

LINUX_TARGET="$(rustc -vV | sed -n 's/^host: //p')"
case "$LINUX_TARGET" in
  x86_64-unknown-linux-gnu)
    EXPECTED_DEB_ARCH="amd64"
    EXPECTED_RPM_ARCH="x86_64"
    EXPECTED_FILE_ARCH="x86-64"
    ;;
  aarch64-unknown-linux-gnu)
    EXPECTED_DEB_ARCH="arm64"
    EXPECTED_RPM_ARCH="aarch64"
    EXPECTED_FILE_ARCH="ARM aarch64"
    ;;
  *)
    echo "error: unsupported Linux host target: $LINUX_TARGET" >&2
    echo "Supported native targets are x86_64-unknown-linux-gnu and aarch64-unknown-linux-gnu." >&2
    exit 1
    ;;
esac

if ! rustup target list --installed --toolchain 1.95.0 | grep -qx "$LINUX_TARGET"; then
  echo "error: $LINUX_TARGET is required. Run: rustup target add $LINUX_TARGET --toolchain 1.95.0" >&2
  exit 1
fi

read_toml_string() {
  local key="$1"
  sed -nE "s/^[[:space:]]*${key}[[:space:]]*=[[:space:]]*\"([^\"]+)\".*/\1/p" src-tauri/Cargo.toml | head -n 1
}

read_json_string() {
  local key="$1"
  sed -nE "s/^[[:space:]]*\"${key}\"[[:space:]]*:[[:space:]]*\"([^\"]+)\".*/\1/p" src-tauri/tauri.conf.json | head -n 1
}

DEFAULT_APP_NAME="$(read_json_string productName)"
DEFAULT_BUNDLE_ID="$(read_json_string identifier)"
DEFAULT_VERSION="$(read_toml_string version)"

if [[ -z "$DEFAULT_APP_NAME" || -z "$DEFAULT_BUNDLE_ID" || -z "$DEFAULT_VERSION" ]]; then
  echo "error: could not read app metadata from src-tauri configuration." >&2
  exit 1
fi

APP_NAME="${APP_NAME:-$DEFAULT_APP_NAME}"
APP_BUNDLE_ID="${APP_BUNDLE_ID:-$DEFAULT_BUNDLE_ID}"
APP_VERSION="${APP_VERSION:-$DEFAULT_VERSION}"
ICON_SOURCE="${ICON_SOURCE:-assets/icons/AppIcon-1024.png}"
DIST_DIR="${DIST_DIR:-dist/linux}"
FORCE_ICONS="${FORCE_ICONS:-0}"
LINUX_BUNDLES="${LINUX_BUNDLES:-deb,appimage}"

if [[ ! "$APP_NAME" =~ ^[[:alnum:]][[:alnum:]\ ._-]{0,62}[[:alnum:]]$ && ! "$APP_NAME" =~ ^[[:alnum:]]$ ]]; then
  echo "error: APP_NAME must start and end with a letter or number and may contain spaces, dots, hyphens, and underscores." >&2
  exit 1
fi

bundle_id_is_valid() {
  local identifier="$1"
  local component
  local -a components

  [[ "$identifier" =~ ^[[:alnum:]][[:alnum:].-]*[[:alnum:]]$ ]] || return 1
  IFS='.' read -r -a components <<< "$identifier"
  [[ ${#components[@]} -ge 2 && ${#identifier} -le 255 ]] || return 1
  for component in "${components[@]}"; do
    [[ "$component" =~ ^[[:alnum:]]([[:alnum:]-]*[[:alnum:]])?$ ]] || return 1
  done
}

if ! bundle_id_is_valid "$APP_BUNDLE_ID"; then
  echo "error: APP_BUNDLE_ID must be a reverse-DNS identifier such as com.example.my-app." >&2
  exit 1
fi

if [[ ! "$APP_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$ ]]; then
  echo "error: APP_VERSION must be a semantic version such as 1.2.3." >&2
  exit 1
fi

if [[ ! "$LINUX_BUNDLES" =~ ^(deb|appimage|rpm)(,(deb|appimage|rpm))*$ ]]; then
  echo "error: LINUX_BUNDLES must be a comma-separated selection of deb, appimage, and rpm." >&2
  exit 1
fi

if [[ ",$LINUX_BUNDLES," == *,deb,* ]] && ! command -v dpkg-deb >/dev/null 2>&1; then
  echo "error: dpkg-deb is required to build and verify Debian packages." >&2
  exit 1
fi

if [[ ",$LINUX_BUNDLES," == *,rpm,* ]] && ! command -v rpmbuild >/dev/null 2>&1; then
  echo "error: rpmbuild is required when LINUX_BUNDLES includes rpm." >&2
  exit 1
fi

if [[ ! -f "$ICON_SOURCE" ]]; then
  if [[ -f src-tauri/icons/icon.png ]]; then
    echo "Icon source missing; using the existing generated icon."
    mkdir -p "$(dirname -- "$ICON_SOURCE")"
    cp src-tauri/icons/icon.png "$ICON_SOURCE"
  else
    echo "error: no icon source or generated fallback is available." >&2
    exit 1
  fi
fi

if [[ "$FORCE_ICONS" == "1" || ! -f src-tauri/icons/icon.png || "$ICON_SOURCE" -nt src-tauri/icons/icon.png ]]; then
  echo "Generating platform icons from $ICON_SOURCE..."
  cargo tauri icon "$ICON_SOURCE"
fi

TEMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/rust-tauri-linux-build.XXXXXX")"
OVERRIDE_CONFIG="$TEMP_DIR/override.json"
trap 'rm -rf "$TEMP_DIR"' EXIT

printf '%s\n' \
  '{' \
  "  \"productName\": \"$APP_NAME\"," \
  "  \"version\": \"$APP_VERSION\"," \
  "  \"identifier\": \"$APP_BUNDLE_ID\"" \
  '}' > "$OVERRIDE_CONFIG"

echo "Building $APP_NAME ($APP_BUNDLE_ID) v$APP_VERSION for $LINUX_TARGET..."
cargo tauri build \
  --bundles "$LINUX_BUNDLES" \
  --ci \
  --config "$OVERRIDE_CONFIG" \
  --target "$LINUX_TARGET" \
  -- \
  --locked

BUNDLE_ROOT="target/$LINUX_TARGET/release/bundle"
mkdir -p "$DIST_DIR"
artifact_count=0

copy_artifacts() {
  local bundle_dir="$1"
  local pattern="$2"
  local artifact
  local artifact_arch
  local artifact_version

  while IFS= read -r -d '' artifact; do
    if [[ ! -s "$artifact" ]]; then
      echo "error: bundle artifact is empty: $artifact" >&2
      exit 1
    fi

    case "$artifact" in
      *.deb)
        artifact_version="$(dpkg-deb --field "$artifact" Version)"
        artifact_arch="$(dpkg-deb --field "$artifact" Architecture)"
        if [[ "$artifact_version" != "$APP_VERSION" ]]; then
          echo "error: expected Debian version $APP_VERSION; found $artifact_version." >&2
          exit 1
        fi
        if [[ "$artifact_arch" != "$EXPECTED_DEB_ARCH" ]]; then
          echo "error: expected Debian architecture $EXPECTED_DEB_ARCH; found $artifact_arch." >&2
          exit 1
        fi
        ;;
      *.AppImage)
        if [[ ! -x "$artifact" ]]; then
          echo "error: AppImage is not executable: $artifact" >&2
          exit 1
        fi
        if [[ "$(file -b "$artifact")" != *"$EXPECTED_FILE_ARCH"* ]]; then
          echo "error: AppImage architecture does not match $LINUX_TARGET." >&2
          exit 1
        fi
        ;;
      *.rpm)
        artifact_version="$(rpm -qp --queryformat '%{VERSION}' "$artifact")"
        artifact_arch="$(rpm -qp --queryformat '%{ARCH}' "$artifact")"
        if [[ "$artifact_version" != "$APP_VERSION" ]]; then
          echo "error: expected RPM version $APP_VERSION; found $artifact_version." >&2
          exit 1
        fi
        if [[ "$artifact_arch" != "$EXPECTED_RPM_ARCH" ]]; then
          echo "error: expected RPM architecture $EXPECTED_RPM_ARCH; found $artifact_arch." >&2
          exit 1
        fi
        ;;
    esac

    cp "$artifact" "$DIST_DIR/"
    printf 'Verified %s\n' "$DIST_DIR/$(basename -- "$artifact")"
    artifact_count=$((artifact_count + 1))
  done < <(find "$bundle_dir" -maxdepth 1 -type f -name "$pattern" -newer "$OVERRIDE_CONFIG" -print0)
}

IFS=',' read -r -a bundles <<< "$LINUX_BUNDLES"
for bundle in "${bundles[@]}"; do
  case "$bundle" in
    deb)
      copy_artifacts "$BUNDLE_ROOT/deb" '*.deb'
      ;;
    appimage)
      copy_artifacts "$BUNDLE_ROOT/appimage" '*.AppImage'
      ;;
    rpm)
      copy_artifacts "$BUNDLE_ROOT/rpm" '*.rpm'
      ;;
  esac
done

if (( artifact_count != ${#bundles[@]} )); then
  echo "error: expected ${#bundles[@]} Linux package(s), found $artifact_count." >&2
  exit 1
fi

echo "Built and verified $artifact_count Linux package(s) in $DIST_DIR"
