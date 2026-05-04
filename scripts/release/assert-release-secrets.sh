#!/usr/bin/env bash
set -euo pipefail

if [ -z "${TAURI_SIGNING_PRIVATE_KEY:-}" ]; then
  echo "TAURI_SIGNING_PRIVATE_KEY is required for updater publishing" >&2
  exit 1
fi
