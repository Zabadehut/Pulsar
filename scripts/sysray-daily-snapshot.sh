#!/usr/bin/env bash
set -euo pipefail

SYSRAY_BIN="${SYSRAY_BIN:-$HOME/.local/bin/sysray}"
SYSRAY_CONFIG="${SYSRAY_CONFIG:-$HOME/.config/sysray/sysray.toml}"
SYSRAY_DAILY_DIR="${SYSRAY_DAILY_DIR:-$HOME/.local/share/sysray/daily}"

mkdir -p "$SYSRAY_DAILY_DIR"

OUTPUT_FILE="$SYSRAY_DAILY_DIR/$(date +%F).jsonl"
"$SYSRAY_BIN" --config "$SYSRAY_CONFIG" snapshot --format json >> "$OUTPUT_FILE"
