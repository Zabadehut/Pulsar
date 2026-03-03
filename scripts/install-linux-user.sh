#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

VERSION="$(sed -n 's/^version *= *"\([^"]*\)"/\1/p' Cargo.toml | head -n 1)"
HOST_TARGET="$(rustc -vV | sed -n 's/^host: //p')"

if [[ "$HOST_TARGET" != *linux* ]]; then
  echo "This installer is intended for Linux targets only." >&2
  exit 1
fi

BUILD_IF_MISSING=1
FORCE_BUILD=0
REINSTALL_SERVICE=1

usage() {
  cat <<'EOF'
Usage: ./scripts/install-linux-user.sh [--no-build] [--force-build] [--no-service]

Installs the current Linux release build to ~/.local/bin/sysray.

Options:
  --no-build    Fail instead of running ./scripts/build-complete.sh when dist/ is missing
  --force-build Always run ./scripts/build-complete.sh before install
  --no-service  Skip sysray user service reinstall
  -h, --help    Show this help message
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --no-build)
      BUILD_IF_MISSING=0
      ;;
    --force-build)
      FORCE_BUILD=1
      ;;
    --no-service)
      REINSTALL_SERVICE=0
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
  shift
done

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Missing required command: $1" >&2
    exit 1
  fi
}

require_command rustc
require_command install

BUNDLE_NAME="sysray-${VERSION}-${HOST_TARGET}"
SOURCE_BINARY="$ROOT_DIR/dist/$BUNDLE_NAME/standalone/sysray"
DEST_DIR="$HOME/.local/bin"
DEST_BINARY="$DEST_DIR/sysray"

if [[ "$FORCE_BUILD" -eq 1 ]]; then
  "$ROOT_DIR/scripts/build-complete.sh"
fi

if [[ ! -x "$SOURCE_BINARY" ]]; then
  if [[ "$BUILD_IF_MISSING" -eq 0 ]]; then
    echo "Release binary not found at $SOURCE_BINARY" >&2
    echo "Run ./scripts/build-complete.sh first or omit --no-build." >&2
    exit 1
  fi

  "$ROOT_DIR/scripts/build-complete.sh"
fi

if [[ ! -x "$SOURCE_BINARY" ]]; then
  echo "Release binary not found at $SOURCE_BINARY after build." >&2
  exit 1
fi

mkdir -p "$DEST_DIR"
install -m 755 "$SOURCE_BINARY" "$DEST_BINARY"

echo "Installed binary:  $DEST_BINARY"
"$DEST_BINARY" --version

if [[ "$REINSTALL_SERVICE" -eq 1 ]]; then
  if command -v systemctl >/dev/null 2>&1; then
    "$DEST_BINARY" service uninstall || true
    "$DEST_BINARY" service install
    systemctl --user status sysray.service --no-pager || true
  else
    echo "systemctl not available; skipped user service install" >&2
  fi
fi

case ":$PATH:" in
  *":$HOME/.local/bin:"*) ;;
  *)
    echo "Warning: $HOME/.local/bin is not currently in PATH" >&2
    ;;
esac
