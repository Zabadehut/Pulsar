#!/usr/bin/env bash
set -euo pipefail

binary="${1:-target/debug/sysray}"
binary_dir="$(cd "$(dirname "${binary}")" && pwd)"
binary="${binary_dir}/$(basename "${binary}")"
platform="$(uname -s)"
root="$(mktemp -d)"
export HOME="${root}/home"
mkdir -p "${HOME}"

cleanup() {
  "${binary}" service uninstall >/dev/null 2>&1 || true
  rm -rf "${root}"
}

trap cleanup EXIT

case "${platform}" in
  Linux)
    runner_path="${HOME}/.local/share/sysray/sysray-service.sh"
    service_path="${HOME}/.config/systemd/user/sysray.service"
    config_path="${HOME}/.config/sysray/sysray.toml"
    status_is_optional=1
    ;;
  Darwin)
    runner_path="${HOME}/Library/Application Support/Sysray/sysray-service.sh"
    service_path="${HOME}/Library/LaunchAgents/com.zabadehut.sysray.plist"
    config_path="${HOME}/Library/Application Support/Sysray/sysray.toml"
    status_is_optional=1
    ;;
  *)
    echo "unsupported platform: ${platform}" >&2
    exit 1
    ;;
esac

install_output=""
install_status=0
install_output="$("${binary}" service install 2>&1)" || install_status=$?

if [[ ${install_status} -ne 0 && ${status_is_optional} -eq 0 ]]; then
  echo "${install_output}" >&2
  exit "${install_status}"
fi

for path in "${runner_path}" "${service_path}" "${config_path}"; do
  if [[ ! -e "${path}" ]]; then
    echo "missing expected service artifact: ${path}" >&2
    echo "${install_output}" >&2
    exit 1
  fi
done

grep -F "\"${binary}\"" "${runner_path}" >/dev/null
grep -F "${runner_path}" "${service_path}" >/dev/null

status_output=""
status_status=0
status_output="$("${binary}" service status 2>&1)" || status_status=$?

if [[ ${status_status} -ne 0 && ${status_is_optional} -ne 1 ]]; then
  echo "${status_output}" >&2
  exit "${status_status}"
fi

"${binary}" service uninstall >/dev/null

for path in "${runner_path}" "${service_path}"; do
  if [[ -e "${path}" ]]; then
    echo "service artifact should have been removed: ${path}" >&2
    exit 1
  fi
done
