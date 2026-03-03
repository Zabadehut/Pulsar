#!/usr/bin/env bash
set -euo pipefail

SYSRAY_DAILY_DIR="${SYSRAY_DAILY_DIR:-$HOME/.local/share/sysray/daily}"
SYSRAY_RAW_RETENTION_DAYS="${SYSRAY_RAW_RETENTION_DAYS:-15}"

mkdir -p "$SYSRAY_DAILY_DIR"

find "$SYSRAY_DAILY_DIR" \
  -maxdepth 1 \
  -type f \
  -name '*.jsonl' \
  -mtime "+$SYSRAY_RAW_RETENTION_DAYS" \
  -delete
