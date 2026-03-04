#!/usr/bin/env bash
set -euo pipefail

binary="${1:-target/debug/sysray}"
binary_dir="$(cd "$(dirname "${binary}")" && pwd)"
binary="${binary_dir}/$(basename "${binary}")"

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "This validator is intended for Linux only." >&2
  exit 1
fi

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Missing required command: $1" >&2
    exit 1
  fi
}

require_command dbus-run-session
require_command systemctl
require_command systemd

root="$(mktemp -d)"
trap 'rm -rf "${root}"' EXIT

export SYSRAY_VALIDATION_HOME="${root}/home"
mkdir -p "${SYSRAY_VALIDATION_HOME}"

dbus-run-session -- bash -lc '
set -euo pipefail

export HOME="'"${SYSRAY_VALIDATION_HOME}"'"
export XDG_RUNTIME_DIR="'"${root}"'/runtime"
mkdir -p "$HOME" "$XDG_RUNTIME_DIR"
chmod 700 "$XDG_RUNTIME_DIR"

systemd --user >/tmp/sysray-systemd-user.log 2>&1 &
systemd_user_pid=$!

cleanup() {
  kill "$systemd_user_pid" >/dev/null 2>&1 || true
  wait "$systemd_user_pid" >/dev/null 2>&1 || true
}

trap cleanup EXIT

ready=0
for _ in $(seq 1 30); do
  if systemctl --user list-units >/dev/null 2>&1; then
    ready=1
    break
  fi
  sleep 1
done

if [[ "$ready" -ne 1 ]]; then
  echo "systemd --user did not become ready" >&2
  cat /tmp/sysray-systemd-user.log >&2 || true
  exit 1
fi

SYSRAY_REQUIRE_LIVE_STATUS=1 SYSRAY_VALIDATION_HOME="$HOME" "'"$(pwd)"'/scripts/validate-service-flow.sh" "'"${binary}"'"
SYSRAY_REQUIRE_LIVE_STATUS=1 SYSRAY_VALIDATION_HOME="$HOME" "'"$(pwd)"'/scripts/validate-schedule-flow.sh" "'"${binary}"'"
'
