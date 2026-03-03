#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="${1:-$ROOT_DIR/docs/screenshots}"
TMP_DIR="${TMPDIR:-/tmp}/sysray-capture"

mkdir -p "$OUT_DIR" "$TMP_DIR"

capture_frame() {
  local name="$1"
  shift
  python3 "$ROOT_DIR/scripts/capture_tui_frame.py" \
    --output "$TMP_DIR/$name.typescript" \
    --rows 40 \
    --cols 140 \
    "$@"
}

render_frame() {
  local name="$1"
  local title="$2"
  python3 "$ROOT_DIR/scripts/render_terminal_capture.py" \
    --input "$TMP_DIR/$name.typescript" \
    --output "$OUT_DIR/$name.svg" \
    --rows 40 \
    --cols 140 \
    --title "$title"
}

capture_frame "tui-overview-real" -- target/debug/sysray --log-level error
render_frame "tui-overview-real" "Sysray TUI overview"

capture_frame "tui-expert-network-real" --keys "8" --keys "q" -- target/debug/sysray --log-level error
render_frame "tui-expert-network-real" "Sysray expert network view"

capture_frame "tui-expert-pressure-real" --keys "7" --keys "q" -- target/debug/sysray --log-level error
render_frame "tui-expert-pressure-real" "Sysray expert pressure view"

capture_frame "tui-expert-jvm-real" --keys "9" --keys "q" -- target/debug/sysray --log-level error
render_frame "tui-expert-jvm-real" "Sysray expert JVM view"

capture_frame "tui-expert-disk-real" --keys "0" --keys "q" -- target/debug/sysray --log-level error
render_frame "tui-expert-disk-real" "Sysray expert disk view"

capture_frame "tui-expert-inventory-real" --keys "g" --keys "q" -- target/debug/sysray --log-level error
render_frame "tui-expert-inventory-real" "Sysray expert disk inventory view"

echo "Generated:"
echo "  $OUT_DIR/tui-overview-real.svg"
echo "  $OUT_DIR/tui-expert-network-real.svg"
echo "  $OUT_DIR/tui-expert-pressure-real.svg"
echo "  $OUT_DIR/tui-expert-jvm-real.svg"
echo "  $OUT_DIR/tui-expert-disk-real.svg"
echo "  $OUT_DIR/tui-expert-inventory-real.svg"
