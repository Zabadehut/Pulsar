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
WINDOWS_SIGN_PFX_BASE64="${SYSRAY_WINDOWS_SIGN_PFX_BASE64:-}"
WINDOWS_SIGN_PFX_PASSWORD="${SYSRAY_WINDOWS_SIGN_PFX_PASSWORD:-}"
WINDOWS_SIGN_TIMESTAMP_URL="${SYSRAY_WINDOWS_SIGN_TIMESTAMP_URL:-http://timestamp.digicert.com}"
MACOS_SIGN_CERT_BASE64="${SYSRAY_MACOS_SIGN_CERT_BASE64:-}"
MACOS_SIGN_CERT_PASSWORD="${SYSRAY_MACOS_SIGN_CERT_PASSWORD:-}"
MACOS_SIGN_IDENTITY="${SYSRAY_MACOS_SIGN_IDENTITY:-}"
GENERATED_ARCHIVES=()
GENERATED_FILES=()

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

rpm_arch() {
  case "$HOST_TARGET" in
    x86_64-*-linux-*) printf '%s\n' "x86_64" ;;
    aarch64-*-linux-*) printf '%s\n' "aarch64" ;;
    armv7*-linux-gnueabihf) printf '%s\n' "armv7hl" ;;
    *) return 1 ;;
  esac
}

create_rpm_package() {
  local rpm_arch
  rpm_arch="$(rpm_arch)" || {
    echo "==> RPM packaging skipped (unsupported Linux target: $HOST_TARGET)"
    return
  }

  if ! command -v rpmbuild >/dev/null 2>&1; then
    echo "==> RPM packaging skipped (rpmbuild not available)"
    return
  fi

  local rpm_root
  local spec_path
  local source_root
  local rpm_path
  rpm_root="$(mktemp -d "${TMPDIR:-/tmp}/sysray-rpmbuild.XXXXXX")"
  trap 'rm -rf "$rpm_root"' RETURN

  mkdir -p \
    "$rpm_root/BUILD" \
    "$rpm_root/RPMS" \
    "$rpm_root/SOURCES" \
    "$rpm_root/SPECS" \
    "$rpm_root/SRPMS" \
    "$rpm_root/TMP"

  source_root="$rpm_root/SOURCES/sysray-${VERSION}"
  mkdir -p \
    "$source_root/usr/bin" \
    "$source_root/usr/share/doc/sysray" \
    "$source_root/usr/share/sysray" \
    "$source_root/usr/lib/systemd/user"

  install -m 755 "$STANDALONE_DIR/$BINARY_NAME" "$source_root/usr/bin/sysray"
  install -m 644 LICENSE "$source_root/usr/share/doc/sysray/LICENSE"
  install -m 644 README.md "$source_root/usr/share/doc/sysray/README.md"
  install -m 644 "$STANDALONE_DIR/BUILD-INFO.txt" "$source_root/usr/share/doc/sysray/BUILD-INFO.txt"
  install -m 644 config/sysray.toml.example "$source_root/usr/share/sysray/sysray.toml.example"
  install -m 644 deploy/systemd/sysray.service "$source_root/usr/lib/systemd/user/sysray.service"

  (
    cd "$rpm_root/SOURCES"
    tar -czf "sysray-${VERSION}.tar.gz" "sysray-${VERSION}"
  )

  spec_path="$rpm_root/SPECS/sysray.spec"
  cat > "$spec_path" <<EOF
%global debug_package %{nil}
Name:           sysray
Version:        $VERSION
Release:        1%{?dist}
Summary:        Modern cross-platform system observability engine
License:        Apache-2.0
URL:            https://github.com/Zabadehut/Sysray
Source0:        %{name}-%{version}.tar.gz
BuildArch:      $rpm_arch

%description
Sysray is a local-first system observability engine with an interactive TUI,
recording, exporters, and cross-platform service scaffolding.

%prep
%autosetup

%build

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
cp -a usr %{buildroot}/

%files
%license /usr/share/doc/sysray/LICENSE
/usr/bin/sysray
/usr/lib/systemd/user/sysray.service
/usr/share/doc/sysray/README.md
/usr/share/doc/sysray/BUILD-INFO.txt
/usr/share/sysray/sysray.toml.example

%changelog
* $(LC_ALL=C date +"%a %b %d %Y") Kevin Vanden-Brande <zaba88@hotmail.fr> - $VERSION-1
- Automated Sysray release package
EOF

  rpmbuild \
    --define "_topdir $rpm_root" \
    --define "_tmppath $rpm_root/TMP" \
    -bb "$spec_path"

  rpm_path="$(find "$rpm_root/RPMS/$rpm_arch" -maxdepth 1 -type f -name "sysray-${VERSION}-1*.${rpm_arch}.rpm" | head -n 1)"
  if [[ -z "$rpm_path" ]]; then
    echo "RPM package not found under $rpm_root/RPMS/$rpm_arch" >&2
    exit 1
  fi
  install -m 644 "$rpm_path" "$DIST_DIR/$(basename "$rpm_path")"
  GENERATED_FILES+=("$DIST_DIR/$(basename "$rpm_path")")
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

sign_windows_binary() {
  if [[ "$HOST_TARGET" != *windows* ]]; then
    return
  fi

  if [[ -z "$WINDOWS_SIGN_PFX_BASE64" || -z "$WINDOWS_SIGN_PFX_PASSWORD" ]]; then
    echo "==> Windows Authenticode signing skipped (set SYSRAY_WINDOWS_SIGN_PFX_BASE64 and SYSRAY_WINDOWS_SIGN_PFX_PASSWORD)"
    return
  fi

  if ! command -v powershell.exe >/dev/null 2>&1; then
    echo "==> Windows Authenticode signing skipped (powershell.exe not available)"
    return
  fi

  local target_path
  target_path="$(to_native_path "$STANDALONE_DIR/$BINARY_NAME")"
  local cert_path
  cert_path="$(to_native_path "${TMPDIR:-/tmp}/sysray-windows-signing.pfx")"

  CERT_PATH="$cert_path" \
  CERT_B64="$WINDOWS_SIGN_PFX_BASE64" \
  CERT_PASS="$WINDOWS_SIGN_PFX_PASSWORD" \
  TS_URL="$WINDOWS_SIGN_TIMESTAMP_URL" \
  TARGET_PATH="$target_path" \
  powershell.exe -NoLogo -NoProfile -NonInteractive -Command \
    '$ErrorActionPreference = "Stop"
     [System.IO.File]::WriteAllBytes($env:CERT_PATH, [System.Convert]::FromBase64String($env:CERT_B64))
     $signtool = (Get-Command signtool.exe -ErrorAction Stop).Source
     & $signtool sign /fd SHA256 /td SHA256 /tr $env:TS_URL /f $env:CERT_PATH /p $env:CERT_PASS $env:TARGET_PATH
     if ($LASTEXITCODE -ne 0) { throw "signtool failed with exit code $LASTEXITCODE" }
     & $signtool verify /pa $env:TARGET_PATH
     if ($LASTEXITCODE -ne 0) { throw "signtool verify failed with exit code $LASTEXITCODE" }' \
    >/dev/null

  echo "==> Windows Authenticode signing complete: $STANDALONE_DIR/$BINARY_NAME"
}

sign_macos_binary() {
  if [[ "$HOST_TARGET" != *apple-darwin* ]]; then
    return
  fi

  if [[ -z "$MACOS_SIGN_CERT_BASE64" || -z "$MACOS_SIGN_CERT_PASSWORD" || -z "$MACOS_SIGN_IDENTITY" ]]; then
    echo "==> macOS code signing skipped (set SYSRAY_MACOS_SIGN_CERT_BASE64, SYSRAY_MACOS_SIGN_CERT_PASSWORD, SYSRAY_MACOS_SIGN_IDENTITY)"
    return
  fi

  if ! command -v security >/dev/null 2>&1 || ! command -v codesign >/dev/null 2>&1; then
    echo "==> macOS code signing skipped (security/codesign not available)"
    return
  fi

  local cert_file="${TMPDIR:-/tmp}/sysray-macos-signing.p12"
  local keychain="${TMPDIR:-/tmp}/sysray-signing.keychain-db"
  local keychain_pw="sysray-temp-keychain"

  if base64 --help 2>/dev/null | grep -q -- '--decode'; then
    printf '%s' "$MACOS_SIGN_CERT_BASE64" | base64 --decode > "$cert_file"
  else
    printf '%s' "$MACOS_SIGN_CERT_BASE64" | base64 -D > "$cert_file"
  fi

  security create-keychain -p "$keychain_pw" "$keychain"
  security set-keychain-settings -lut 21600 "$keychain"
  security unlock-keychain -p "$keychain_pw" "$keychain"
  security import "$cert_file" -k "$keychain" -P "$MACOS_SIGN_CERT_PASSWORD" -T /usr/bin/codesign -T /usr/bin/security
  security set-key-partition-list -S apple-tool:,apple: -s -k "$keychain_pw" "$keychain"
  security list-keychains -d user -s "$keychain" login.keychain-db

  codesign --force --options runtime --timestamp --sign "$MACOS_SIGN_IDENTITY" "$STANDALONE_DIR/$BINARY_NAME"
  codesign --verify --deep --strict --verbose=2 "$STANDALONE_DIR/$BINARY_NAME"

  echo "==> macOS code signing complete: $STANDALONE_DIR/$BINARY_NAME"
}

is_generated_archive() {
  local candidate="$1"
  local archive_path
  for archive_path in "${GENERATED_ARCHIVES[@]}"; do
    if [[ "$archive_path" == "$candidate" ]]; then
      return 0
    fi
  done
  return 1
}

require_command cargo
require_command rustc
require_command tar

if [[ "${SYSRAY_SKIP_VALIDATION:-0}" != "1" ]]; then
  echo "==> Validation"
  cargo fmt --all -- --check
  cargo clippy --all-targets -- -D warnings
  cargo test
else
  echo "==> Validation skipped (SYSRAY_SKIP_VALIDATION=1)"
fi

echo "==> Release build"
cargo build --release

if [[ ! -f "$BINARY_PATH" ]]; then
  echo "Release binary not found at $BINARY_PATH" >&2
  exit 1
fi

echo "==> Assemble dist bundle"
rm -rf "$WORK_DIR"
mkdir -p "$STANDALONE_DIR" "$PREREQS_DIR/linux" "$PREREQS_DIR/macos" "$PREREQS_DIR/windows"
rm -f "$DIST_DIR/${BUNDLE_NAME}.tar.gz" "$DIST_DIR/${BUNDLE_NAME}.zip" "$DIST_DIR/${BUNDLE_NAME}.exe" "$DIST_DIR"/*.rpm "$CHECKSUMS_PATH" "$SIGNATURE_PATH"

cp "$BINARY_PATH" "$STANDALONE_DIR/$BINARY_NAME"
cp config/sysray.toml.example "$STANDALONE_DIR/sysray.toml.example"
cp README.md "$STANDALONE_DIR/README.md"

cp deploy/systemd/sysray.service "$PREREQS_DIR/linux/sysray.service"
cp deploy/launchd/com.zabadehut.sysray.plist "$PREREQS_DIR/macos/com.zabadehut.sysray.plist"
cp deploy/windows/sysray-task.xml "$PREREQS_DIR/windows/sysray-task.xml"
cp config/sysray.toml.example "$PREREQS_DIR/linux/sysray.toml.example"
cp config/sysray.toml.example "$PREREQS_DIR/macos/sysray.toml.example"
cp config/sysray.toml.example "$PREREQS_DIR/windows/sysray.toml.example"

sign_windows_binary
sign_macos_binary

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
GENERATED_FILES+=("$TAR_ARCHIVE_PATH")

if [[ "$HOST_TARGET" == *windows* ]]; then
  ZIP_ARCHIVE_PATH="$DIST_DIR/${BUNDLE_NAME}.zip"
  create_zip_archive "$ZIP_ARCHIVE_PATH"
  GENERATED_ARCHIVES+=("$ZIP_ARCHIVE_PATH")
  GENERATED_FILES+=("$ZIP_ARCHIVE_PATH")

  EXE_ARTIFACT_PATH="$DIST_DIR/${BUNDLE_NAME}.exe"
  cp "$STANDALONE_DIR/$BINARY_NAME" "$EXE_ARTIFACT_PATH"
  GENERATED_FILES+=("$EXE_ARTIFACT_PATH")
fi

if [[ "$HOST_TARGET" == *linux* ]]; then
  create_rpm_package
fi

: > "$CHECKSUMS_PATH"
for artifact_path in "${GENERATED_FILES[@]}"; do
  sha256_file "$artifact_path" >> "$CHECKSUMS_PATH"
done

generate_signature

echo "==> Complete"
echo "Standalone bundle: $STANDALONE_DIR"
echo "Install prereqs:   $PREREQS_DIR"
for archive_path in "${GENERATED_ARCHIVES[@]}"; do
  echo "Archive:           $archive_path"
done
for artifact_path in "${GENERATED_FILES[@]}"; do
  if ! is_generated_archive "$artifact_path"; then
    echo "Artifact:          $artifact_path"
  fi
done
echo "Checksums:         $CHECKSUMS_PATH"
