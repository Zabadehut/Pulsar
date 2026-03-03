#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

VERSION="$(sed -n 's/^version *= *"\([^"]*\)"/\1/p' Cargo.toml | head -n 1)"
HOST_TARGET="$(rustc -vV | sed -n 's/^host: //p')"
BINARY_NAME="sysray"
if [[ "$HOST_TARGET" == *windows* ]]; then
  BINARY_NAME="sysray.exe"
fi

RELEASE_DIR="target/release"
BINARY_PATH="$RELEASE_DIR/$BINARY_NAME"
BUNDLE_NAME="sysray-${VERSION}-${HOST_TARGET}"
DIST_DIR="$ROOT_DIR/dist"
WORK_DIR="$DIST_DIR/$BUNDLE_NAME"
STANDALONE_DIR="$WORK_DIR/standalone"
PREREQS_DIR="$WORK_DIR/install-prereqs"
CHECKSUMS_PATH="$DIST_DIR/${BUNDLE_NAME}.SHA256SUMS"
SIGNATURE_PATH="${CHECKSUMS_PATH}.asc"
SIGNING_KEY="${SYSRAY_GPG_KEY_ID:-}"
GENERATED_ARCHIVES=()

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Missing required command: $1" >&2
    exit 1
  fi
}

sha256_file() {
  local file_path="$1"

  if command -v sha256sum >/dev/null 2>&1; then
    (
      cd "$(dirname "$file_path")"
      sha256sum "$(basename "$file_path")"
    )
    return
  fi

  if command -v shasum >/dev/null 2>&1; then
    (
      cd "$(dirname "$file_path")"
      shasum -a 256 "$(basename "$file_path")"
    )
    return
  fi

  echo "Missing required command: sha256sum or shasum" >&2
  exit 1
}

to_native_path() {
  local file_path="$1"

  if command -v cygpath >/dev/null 2>&1; then
    cygpath -w "$file_path"
    return
  fi

  printf '%s\n' "$file_path"
}

create_zip_archive() {
  local archive_path="$1"

  if command -v zip >/dev/null 2>&1; then
    (
      cd "$DIST_DIR"
      zip -qr "$(basename "$archive_path")" "$BUNDLE_NAME"
    )
    return
  fi

  if command -v powershell.exe >/dev/null 2>&1; then
    local source_path
    local destination_path
    source_path="$(to_native_path "$WORK_DIR")"
    destination_path="$(to_native_path "$archive_path")"

    powershell.exe -NoLogo -NoProfile -Command \
      "Compress-Archive -Path '$source_path' -DestinationPath '$destination_path' -Force" \
      >/dev/null
    return
  fi

  echo "Missing required command: zip or powershell.exe" >&2
  exit 1
}

generate_signature() {
  if ! command -v gpg >/dev/null 2>&1; then
    echo "==> Release signature skipped (gpg not available)"
    return
  fi

  if [[ -z "$SIGNING_KEY" ]]; then
    echo "==> Release signature skipped (set SYSRAY_GPG_KEY_ID to enable signing)"
    return
  fi

  gpg --batch --yes --armor --detach-sign \
    --local-user "$SIGNING_KEY" \
    --output "$SIGNATURE_PATH" \
    "$CHECKSUMS_PATH"

  echo "Signature:         $SIGNATURE_PATH"
}

require_command cargo
require_command rustc
require_command tar

echo "==> Validation"
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test

echo "==> Release build"
cargo build --release

if [[ ! -f "$BINARY_PATH" ]]; then
  echo "Release binary not found at $BINARY_PATH" >&2
  exit 1
fi

echo "==> Assemble dist bundle"
rm -rf "$WORK_DIR"
mkdir -p "$STANDALONE_DIR" "$PREREQS_DIR/linux" "$PREREQS_DIR/macos" "$PREREQS_DIR/windows"
rm -f "$DIST_DIR/${BUNDLE_NAME}.tar.gz" "$DIST_DIR/${BUNDLE_NAME}.zip" "$CHECKSUMS_PATH" "$SIGNATURE_PATH"

cp "$BINARY_PATH" "$STANDALONE_DIR/$BINARY_NAME"
cp config/sysray.toml.example "$STANDALONE_DIR/sysray.toml.example"
cp README.md "$STANDALONE_DIR/README.md"

cp deploy/systemd/sysray.service "$PREREQS_DIR/linux/sysray.service"
cp deploy/launchd/com.zabadehut.sysray.plist "$PREREQS_DIR/macos/com.zabadehut.sysray.plist"
cp deploy/windows/sysray-task.xml "$PREREQS_DIR/windows/sysray-task.xml"
cp config/sysray.toml.example "$PREREQS_DIR/linux/sysray.toml.example"
cp config/sysray.toml.example "$PREREQS_DIR/macos/sysray.toml.example"
cp config/sysray.toml.example "$PREREQS_DIR/windows/sysray.toml.example"

cat > "$STANDALONE_DIR/BUILD-INFO.txt" <<EOF
Sysray standalone bundle
Version: $VERSION
Target: $HOST_TARGET
Binary: $BINARY_NAME
Built at: $(date -u +"%Y-%m-%dT%H:%M:%SZ")
EOF

cat > "$PREREQS_DIR/README.txt" <<EOF
Install prerequisite files bundled by OS:

- linux/sysray.service: systemd user service template
- macos/com.zabadehut.sysray.plist: launchd agent template
- windows/sysray-task.xml: Task Scheduler template

Each OS folder also includes sysray.toml.example as a starter configuration.
The standalone binary is available under ../standalone/.
EOF

TAR_ARCHIVE_PATH="$DIST_DIR/${BUNDLE_NAME}.tar.gz"
tar -czf "$TAR_ARCHIVE_PATH" -C "$DIST_DIR" "$BUNDLE_NAME"
GENERATED_ARCHIVES+=("$TAR_ARCHIVE_PATH")

if [[ "$HOST_TARGET" == *windows* ]]; then
  ZIP_ARCHIVE_PATH="$DIST_DIR/${BUNDLE_NAME}.zip"
  create_zip_archive "$ZIP_ARCHIVE_PATH"
  GENERATED_ARCHIVES+=("$ZIP_ARCHIVE_PATH")
fi

: > "$CHECKSUMS_PATH"
for archive_path in "${GENERATED_ARCHIVES[@]}"; do
  sha256_file "$archive_path" >> "$CHECKSUMS_PATH"
done

generate_signature

echo "==> Complete"
echo "Standalone bundle: $STANDALONE_DIR"
echo "Install prereqs:   $PREREQS_DIR"
for archive_path in "${GENERATED_ARCHIVES[@]}"; do
  echo "Archive:           $archive_path"
done
echo "Checksums:         $CHECKSUMS_PATH"
