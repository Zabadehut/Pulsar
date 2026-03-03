#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

OUTPUT_ROOT="${OUTPUT_ROOT:-$ROOT_DIR/.validation}"
SYSRAY_BIN="${SYSRAY_BIN:-}"
SYSRAY_CONFIG="${SYSRAY_CONFIG:-$HOME/.config/sysray/sysray.toml}"

usage() {
  cat <<'EOF'
Usage: ./scripts/validate-linux-metrics.sh [--output-dir DIR]

Captures one Sysray snapshot and compares key Linux metrics against /proc and basic system commands.

Options:
  --output-dir DIR  Root directory for validation artifacts (default: ./.validation)
  -h, --help        Show this help message

Environment:
  SYSRAY_BIN        Override Sysray binary path
  SYSRAY_CONFIG     Override Sysray config path
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-dir)
      OUTPUT_ROOT="${2:-}"
      shift
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

require_command awk
require_command jq
require_command hostname
require_command nproc
require_command uname

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "This validation script is intended for Linux only." >&2
  exit 1
fi

find_pulsar_bin() {
  if [[ -n "$SYSRAY_BIN" && -x "$SYSRAY_BIN" ]]; then
    printf '%s\n' "$SYSRAY_BIN"
    return
  fi

  if [[ -x "$ROOT_DIR/target/debug/sysray" ]]; then
    printf '%s\n' "$ROOT_DIR/target/debug/sysray"
    return
  fi

  if [[ -x "$HOME/.local/bin/sysray" ]]; then
    printf '%s\n' "$HOME/.local/bin/sysray"
    return
  fi

  cargo build >/dev/null
  printf '%s\n' "$ROOT_DIR/target/debug/sysray"
}

SYSRAY_BIN="$(find_pulsar_bin)"
RUN_ID="$(date -u +%Y%m%dT%H%M%SZ)"
RUN_DIR="$OUTPUT_ROOT/$RUN_ID"
RAW_DIR="$RUN_DIR/raw"
mkdir -p "$RAW_DIR"

SNAPSHOT_JSON="$RUN_DIR/sysray-snapshot.json"
REPORT_MD="$RUN_DIR/report.md"
RESULTS_CSV="$RUN_DIR/results.csv"

"$SYSRAY_BIN" --config "$SYSRAY_CONFIG" snapshot --format json > "$SNAPSHOT_JSON"
cp /proc/loadavg "$RAW_DIR/proc-loadavg.txt"
cp /proc/uptime "$RAW_DIR/proc-uptime.txt"
cp /proc/meminfo "$RAW_DIR/proc-meminfo.txt"
cp /proc/stat "$RAW_DIR/proc-stat.txt"
cp /proc/net/dev "$RAW_DIR/proc-net-dev.txt"
cp /proc/diskstats "$RAW_DIR/proc-diskstats.txt"
hostname > "$RAW_DIR/hostname.txt"
uname -r > "$RAW_DIR/uname-r.txt"
nproc > "$RAW_DIR/nproc.txt"
if command -v vmstat >/dev/null 2>&1; then
  vmstat -s > "$RAW_DIR/vmstat-s.txt"
fi

printf 'check,status,snapshot_value,reference_value,details\n' > "$RESULTS_CSV"

record_result() {
  local check="$1"
  local status="$2"
  local snapshot_value="$3"
  local reference_value="$4"
  local details="$5"
  printf '%s,%s,%s,%s,%s\n' "$check" "$status" "$snapshot_value" "$reference_value" "$details" >> "$RESULTS_CSV"
}

compare_exact() {
  local check="$1"
  local snapshot_value="$2"
  local reference_value="$3"
  local details="${4:-exact match required}"
  if [[ "$snapshot_value" == "$reference_value" ]]; then
    record_result "$check" "PASS" "$snapshot_value" "$reference_value" "$details"
  else
    record_result "$check" "FAIL" "$snapshot_value" "$reference_value" "$details"
  fi
}

compare_abs_tol() {
  local check="$1"
  local snapshot_value="$2"
  local reference_value="$3"
  local tolerance="$4"
  local details="${5:-absolute tolerance}"
  local status

  status="$(awk -v a="$snapshot_value" -v b="$reference_value" -v tol="$tolerance" 'BEGIN {
    diff = a - b
    if (diff < 0) diff = -diff
    if (diff <= tol) print "PASS"; else print "FAIL"
  }')"
  record_result "$check" "$status" "$snapshot_value" "$reference_value" "$details (tol=$tolerance)"
}

compare_monotonic() {
  local check="$1"
  local snapshot_value="$2"
  local reference_value="$3"
  local details="${4:-reference should be >= snapshot}"
  local status

  status="$(awk -v snap="$snapshot_value" -v ref="$reference_value" 'BEGIN {
    if (ref >= snap) print "PASS"; else print "FAIL"
  }')"
  record_result "$check" "$status" "$snapshot_value" "$reference_value" "$details"
}

meminfo_kb() {
  local key="$1"
  awk -v key="$key" '$1 == key ":" { print $2; exit }' "$RAW_DIR/proc-meminfo.txt"
}

snapshot_jq() {
  local query="$1"
  jq -r "$query" "$SNAPSHOT_JSON"
}

compare_exact "system.hostname" \
  "$(snapshot_jq '.system.hostname')" \
  "$(cat "$RAW_DIR/hostname.txt")"

compare_exact "system.kernel_version" \
  "$(snapshot_jq '.system.kernel_version')" \
  "$(cat "$RAW_DIR/uname-r.txt")"

compare_exact "system.cpu_count" \
  "$(snapshot_jq '.system.cpu_count')" \
  "$(cat "$RAW_DIR/nproc.txt")"

compare_abs_tol "system.uptime_seconds" \
  "$(snapshot_jq '.system.uptime_seconds')" \
  "$(awk '{printf "%.0f\n", $1}' "$RAW_DIR/proc-uptime.txt")" \
  "5" \
  "snapshot vs /proc/uptime"

compare_abs_tol "cpu.load_avg_1" \
  "$(snapshot_jq '.cpu.load_avg_1')" \
  "$(awk '{print $1}' "$RAW_DIR/proc-loadavg.txt")" \
  "0.05" \
  "snapshot vs /proc/loadavg"

compare_abs_tol "cpu.load_avg_5" \
  "$(snapshot_jq '.cpu.load_avg_5')" \
  "$(awk '{print $2}' "$RAW_DIR/proc-loadavg.txt")" \
  "0.05" \
  "snapshot vs /proc/loadavg"

compare_abs_tol "cpu.load_avg_15" \
  "$(snapshot_jq '.cpu.load_avg_15')" \
  "$(awk '{print $3}' "$RAW_DIR/proc-loadavg.txt")" \
  "0.05" \
  "snapshot vs /proc/loadavg"

compare_exact "memory.total_kb" \
  "$(snapshot_jq '.memory.total_kb')" \
  "$(meminfo_kb MemTotal)"

compare_abs_tol "memory.free_kb" \
  "$(snapshot_jq '.memory.free_kb')" \
  "$(meminfo_kb MemFree)" \
  "8192" \
  "snapshot vs /proc/meminfo sampled just after snapshot"

compare_abs_tol "memory.available_kb" \
  "$(snapshot_jq '.memory.available_kb')" \
  "$(meminfo_kb MemAvailable)" \
  "8192" \
  "snapshot vs /proc/meminfo sampled just after snapshot"

compare_abs_tol "memory.cached_kb" \
  "$(snapshot_jq '.memory.cached_kb')" \
  "$(meminfo_kb Cached)" \
  "8192" \
  "snapshot vs /proc/meminfo sampled just after snapshot"

compare_exact "memory.buffers_kb" \
  "$(snapshot_jq '.memory.buffers_kb')" \
  "$(meminfo_kb Buffers)"

compare_abs_tol "memory.dirty_kb" \
  "$(snapshot_jq '.memory.dirty_kb')" \
  "$(meminfo_kb Dirty)" \
  "1024" \
  "snapshot vs /proc/meminfo sampled just after snapshot"

compare_exact "memory.swap_total_kb" \
  "$(snapshot_jq '.memory.swap_total_kb')" \
  "$(meminfo_kb SwapTotal)"

compare_exact "memory.swap_used_kb" \
  "$(snapshot_jq '.memory.swap_used_kb')" \
  "$(awk -v total="$(meminfo_kb SwapTotal)" -v free="$(meminfo_kb SwapFree)" 'BEGIN { print total - free }')"

compare_monotonic "cpu.context_switches" \
  "$(snapshot_jq '.cpu.context_switches')" \
  "$(awk '$1 == "ctxt" { print $2; exit }' "$RAW_DIR/proc-stat.txt")" \
  "/proc/stat ctxt should not be below the snapshot"

compare_monotonic "cpu.interrupts" \
  "$(snapshot_jq '.cpu.interrupts')" \
  "$(awk '$1 == "intr" { print $2; exit }' "$RAW_DIR/proc-stat.txt")" \
  "/proc/stat intr should not be below the snapshot"

SNAPSHOT_IFACES="$(snapshot_jq '.networks[].interface' | sort -u)"
PROC_IFACES="$(awk -F: 'NR > 2 {gsub(/^[ \t]+|[ \t]+$/, "", $1); print $1}' "$RAW_DIR/proc-net-dev.txt" | sort -u)"
MISSING_IFACES="$(comm -23 <(printf '%s\n' "$SNAPSHOT_IFACES") <(printf '%s\n' "$PROC_IFACES") || true)"
if [[ -z "$MISSING_IFACES" ]]; then
  record_result "network.interfaces" "PASS" "$(printf '%s' "$SNAPSHOT_IFACES" | tr '\n' ' ')" "$(printf '%s' "$PROC_IFACES" | tr '\n' ' ')" "snapshot interfaces found in /proc/net/dev"
else
  record_result "network.interfaces" "FAIL" "$(printf '%s' "$SNAPSHOT_IFACES" | tr '\n' ' ')" "$(printf '%s' "$PROC_IFACES" | tr '\n' ' ')" "missing in /proc/net/dev: $(printf '%s' "$MISSING_IFACES" | tr '\n' ' ')"
fi

SNAPSHOT_DISKS="$(snapshot_jq '.disks[].device' | sed '/^$/d' | sort -u)"
PROC_DISKS="$(awk '{print $3}' "$RAW_DIR/proc-diskstats.txt" | sort -u)"
MISSING_DISKS="$(comm -23 <(printf '%s\n' "$SNAPSHOT_DISKS") <(printf '%s\n' "$PROC_DISKS") || true)"
if [[ -z "$MISSING_DISKS" ]]; then
  record_result "disk.devices" "PASS" "$(printf '%s' "$SNAPSHOT_DISKS" | tr '\n' ' ')" "$(printf '%s' "$PROC_DISKS" | tr '\n' ' ')" "snapshot disk devices found in /proc/diskstats"
else
  record_result "disk.devices" "FAIL" "$(printf '%s' "$SNAPSHOT_DISKS" | tr '\n' ' ')" "$(printf '%s' "$PROC_DISKS" | tr '\n' ' ')" "missing in /proc/diskstats: $(printf '%s' "$MISSING_DISKS" | tr '\n' ' ')"
fi

PASS_COUNT="$(awk -F, 'NR > 1 && $2 == "PASS" { count++ } END { print count + 0 }' "$RESULTS_CSV")"
FAIL_COUNT="$(awk -F, 'NR > 1 && $2 == "FAIL" { count++ } END { print count + 0 }' "$RESULTS_CSV")"

{
  echo "# Linux Metrics Validation"
  echo
  echo "- Run ID: \`$RUN_ID\`"
  echo "- Host: \`$(cat "$RAW_DIR/hostname.txt")\`"
  echo "- Sysray binary: \`$SYSRAY_BIN\`"
  echo "- Snapshot file: \`$SNAPSHOT_JSON\`"
  echo "- Pass checks: \`$PASS_COUNT\`"
  echo "- Failed checks: \`$FAIL_COUNT\`"
  echo
  echo "| Check | Status | Snapshot | Reference | Details |"
  echo "|---|---|---:|---:|---|"
  awk -F, 'NR > 1 {
    printf "| %s | %s | `%s` | `%s` | %s |\n", $1, $2, $3, $4, $5
  }' "$RESULTS_CSV"
  echo
  echo "Raw reference files are stored under \`$RAW_DIR\`."
} > "$REPORT_MD"

echo "Validation report: $REPORT_MD"
cat "$REPORT_MD"

if [[ "$FAIL_COUNT" -gt 0 ]]; then
  exit 1
fi
