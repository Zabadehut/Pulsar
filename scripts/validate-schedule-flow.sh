#!/usr/bin/env bash
set -euo pipefail

binary="${1:-target/debug/sysray}"
binary_dir="$(cd "$(dirname "${binary}")" && pwd)"
binary="${binary_dir}/$(basename "${binary}")"
platform="$(uname -s)"
managed_home=1
root=""
if [[ -n "${SYSRAY_VALIDATION_HOME:-}" ]]; then
  managed_home=0
  export HOME="${SYSRAY_VALIDATION_HOME}"
  mkdir -p "${HOME}"
else
  root="$(mktemp -d)"
  export HOME="${root}/home"
  mkdir -p "${HOME}"
fi

cleanup() {
  "${binary}" schedule uninstall >/dev/null 2>&1 || true
  if [[ ${managed_home} -eq 1 ]]; then
    rm -rf "${root}"
  fi
}

trap cleanup EXIT

case "${platform}" in
  Linux)
    config_path="${HOME}/.config/sysray/sysray.toml"
    runner_dir="${HOME}/.local/share/sysray/schedule"
    artifact_paths=(
      "${runner_dir}/snapshot.sh"
      "${runner_dir}/prune.sh"
      "${runner_dir}/archive.sh"
      "${HOME}/.config/systemd/user/sysray-snapshot.service"
      "${HOME}/.config/systemd/user/sysray-snapshot.timer"
      "${HOME}/.config/systemd/user/sysray-prune.service"
      "${HOME}/.config/systemd/user/sysray-prune.timer"
      "${HOME}/.config/systemd/user/sysray-archive.service"
      "${HOME}/.config/systemd/user/sysray-archive.timer"
    )
    status_is_optional=1
    if [[ "${SYSRAY_REQUIRE_LIVE_STATUS:-0}" == "1" ]]; then
      status_is_optional=0
    fi
    ;;
  Darwin)
    config_path="${HOME}/Library/Application Support/Sysray/sysray.toml"
    runner_dir="${HOME}/Library/Application Support/Sysray/schedule"
    artifact_paths=(
      "${runner_dir}/snapshot.sh"
      "${runner_dir}/prune.sh"
      "${runner_dir}/archive.sh"
      "${HOME}/Library/LaunchAgents/com.zabadehut.sysray.snapshot.plist"
      "${HOME}/Library/LaunchAgents/com.zabadehut.sysray.prune.plist"
      "${HOME}/Library/LaunchAgents/com.zabadehut.sysray.archive.plist"
    )
    status_is_optional=1
    ;;
  *)
    echo "unsupported platform: ${platform}" >&2
    exit 1
    ;;
esac

install_output=""
install_status=0
install_output="$("${binary}" schedule install 2>&1)" || install_status=$?

if [[ ${install_status} -ne 0 && ${status_is_optional} -eq 0 ]]; then
  echo "${install_output}" >&2
  exit "${install_status}"
fi

if [[ ! -e "${config_path}" ]]; then
  echo "missing expected schedule config: ${config_path}" >&2
  echo "${install_output}" >&2
  exit 1
fi

for path in "${artifact_paths[@]}"; do
  if [[ ! -e "${path}" ]]; then
    echo "missing expected schedule artifact: ${path}" >&2
    echo "${install_output}" >&2
    exit 1
  fi
done

for runner in "${runner_dir}/snapshot.sh" "${runner_dir}/prune.sh" "${runner_dir}/archive.sh"; do
  grep -F "\"${binary}\"" "${runner}" >/dev/null
done

case "${platform}" in
  Linux)
    grep -F "${runner_dir}/snapshot.sh" "${HOME}/.config/systemd/user/sysray-snapshot.service" >/dev/null
    grep -F "${runner_dir}/prune.sh" "${HOME}/.config/systemd/user/sysray-prune.service" >/dev/null
    grep -F "${runner_dir}/archive.sh" "${HOME}/.config/systemd/user/sysray-archive.service" >/dev/null
    ;;
  Darwin)
    grep -F "${runner_dir}/snapshot.sh" "${HOME}/Library/LaunchAgents/com.zabadehut.sysray.snapshot.plist" >/dev/null
    grep -F "${runner_dir}/prune.sh" "${HOME}/Library/LaunchAgents/com.zabadehut.sysray.prune.plist" >/dev/null
    grep -F "${runner_dir}/archive.sh" "${HOME}/Library/LaunchAgents/com.zabadehut.sysray.archive.plist" >/dev/null
    ;;
esac

status_output=""
status_status=0
status_output="$("${binary}" schedule status 2>&1)" || status_status=$?

if [[ ${status_status} -ne 0 && ${status_is_optional} -eq 0 ]]; then
  echo "${status_output}" >&2
  exit "${status_status}"
fi

"${binary}" schedule uninstall >/dev/null

for path in "${artifact_paths[@]}"; do
  if [[ -e "${path}" ]]; then
    echo "schedule artifact should have been removed: ${path}" >&2
    exit 1
  fi
done
