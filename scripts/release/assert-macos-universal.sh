#!/usr/bin/env bash
set -euo pipefail

app_path="${1:-}"
if [ -z "${app_path}" ] || [ ! -d "${app_path}" ] || [ "${app_path##*.}" != "app" ]; then
  echo "usage: scripts/release/assert-macos-universal.sh <path-to-app.app>" >&2
  exit 1
fi

plist_path="${app_path}/Contents/Info.plist"
executable_name=""
if [ -f "${plist_path}" ] && command -v /usr/libexec/PlistBuddy >/dev/null 2>&1; then
  executable_name="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleExecutable' "${plist_path}" 2>/dev/null || true)"
fi

if [ -z "${executable_name}" ]; then
  executable_name="$(basename "${app_path}" .app)"
fi

executable_path="${app_path}/Contents/MacOS/${executable_name}"
if [ ! -x "${executable_path}" ]; then
  fallback_path="$(find "${app_path}/Contents/MacOS" -maxdepth 1 -type f -perm -111 | head -n 1 || true)"
  if [ -n "${fallback_path}" ]; then
    executable_path="${fallback_path}"
  fi
fi

if [ ! -x "${executable_path}" ]; then
  echo "macOS app executable not found in ${app_path}" >&2
  exit 1
fi

archs="$(lipo -archs "${executable_path}")"
case " ${archs} " in
  *" x86_64 "*" arm64 "*) ;;
  *" arm64 "*" x86_64 "*) ;;
  *)
    echo "macOS app executable is not universal: ${archs}" >&2
    exit 1
    ;;
esac

echo "macOS universal executable verified: ${archs}"
