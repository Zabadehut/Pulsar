#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

JOURNAL_LINES=50

usage() {
  cat <<'EOF'
Usage: ./scripts/redeploy-dev-user-service.sh [--journal-lines N] [--no-journal]

Runs the full local Linux developer redeploy flow for the user service.

Options:
  --journal-lines N  Show the last N journal lines after restart (default: 50)
  --no-journal       Skip the final journalctl output
  -h, --help         Show this help message
EOF
}

SHOW_JOURNAL=1

while [[ $# -gt 0 ]]; do
  case "$1" in
    --journal-lines)
      JOURNAL_LINES="${2:-}"
      if [[ -z "$JOURNAL_LINES" || ! "$JOURNAL_LINES" =~ ^[0-9]+$ ]]; then
        echo "Expected a positive integer after --journal-lines" >&2
        exit 1
      fi
      shift
      ;;
    --no-journal)
      SHOW_JOURNAL=0
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

require_command cargo
require_command systemctl

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "This redeploy flow is intended for Linux only." >&2
  exit 1
fi

echo "==> Format"
cargo fmt --all

echo "==> Lint"
cargo clippy --all-targets -- -D warnings

echo "==> Test"
cargo test

echo "==> Debug build"
cargo build

echo "==> Install fresh release bundle"
"$ROOT_DIR/scripts/install-linux-user.sh" --force-build

echo "==> Restart user service"
systemctl --user restart sysray.service

echo "==> User service status"
systemctl --user status sysray.service --no-pager

if [[ "$SHOW_JOURNAL" -eq 1 ]]; then
  echo "==> Recent logs"
  journalctl --user -u sysray.service -n "$JOURNAL_LINES" --no-pager
fi
