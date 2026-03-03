#!/usr/bin/env bash
set -euo pipefail

SYSRAY_DAILY_DIR="${SYSRAY_DAILY_DIR:-$HOME/.local/share/sysray/daily}"
SYSRAY_ARCHIVE_DIR="${SYSRAY_ARCHIVE_DIR:-$HOME/.local/share/sysray/archives}"
SYSRAY_ARCHIVE_MIN_DAYS="${SYSRAY_ARCHIVE_MIN_DAYS:-15}"
SYSRAY_ARCHIVE_MAX_DAYS="${SYSRAY_ARCHIVE_MAX_DAYS:-60}"

if ! command -v zip >/dev/null 2>&1; then
  echo "Missing required command: zip" >&2
  exit 1
fi

mkdir -p "$SYSRAY_DAILY_DIR" "$SYSRAY_ARCHIVE_DIR"

mapfile -d '' FILES < <(
  find "$SYSRAY_DAILY_DIR" \
    -maxdepth 1 \
    -type f \
    -name '*.jsonl' \
    -mtime "+$SYSRAY_ARCHIVE_MIN_DAYS" \
    -mtime "-$SYSRAY_ARCHIVE_MAX_DAYS" \
    -print0
)

if [[ "${#FILES[@]}" -eq 0 ]]; then
  exit 0
fi

ARCHIVE_PATH="$SYSRAY_ARCHIVE_DIR/sysray-archive-$(date +%F).zip"
zip -qj "$ARCHIVE_PATH" "${FILES[@]}"
rm -f "${FILES[@]}"

find "$SYSRAY_ARCHIVE_DIR" \
  -maxdepth 1 \
  -type f \
  -name 'sysray-archive-*.zip' \
  -mtime "+$SYSRAY_ARCHIVE_MAX_DAYS" \
  -delete
