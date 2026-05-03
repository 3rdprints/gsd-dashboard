#!/usr/bin/env bash
set -euo pipefail

check_mode=false
if [ "${1:-}" = "--check" ]; then
  check_mode=true
  shift
fi

if [ "$#" -gt 0 ]; then
  echo "usage: $0 [--check]" >&2
  exit 2
fi

VERSION="${VERSION:-$(git describe --tags --always)}"
OUTPUT_DIR="${OUTPUT_DIR:-dist-release}"
OUTPUT_PATH="${OUTPUT_PATH:-${OUTPUT_DIR}/gsd-dashboard-source-${VERSION}.tar.gz}"
TEMP_DIR="$(mktemp -d)"
BUNDLE_ROOT="${TEMP_DIR}/gsd-dashboard-source-${VERSION}"

cleanup() {
  rm -rf "${TEMP_DIR}"
}
trap cleanup EXIT

mkdir -p "${BUNDLE_ROOT}" "${OUTPUT_DIR}"

npm ci
npm run build

(cd src-tauri && cargo vendor "${TEMP_DIR}/vendor")

mkdir -p \
  "${BUNDLE_ROOT}/docs/distribution" \
  "${BUNDLE_ROOT}/src-tauri" \
  "${BUNDLE_ROOT}/vendor"

if [ -f Cargo.lock ]; then
  cp Cargo.lock "${BUNDLE_ROOT}/Cargo.lock"
else
  cp src-tauri/Cargo.lock "${BUNDLE_ROOT}/Cargo.lock"
fi
cp package-lock.json "${BUNDLE_ROOT}/package-lock.json"
cp package.json "${BUNDLE_ROOT}/package.json"
cp src-tauri/Cargo.toml "${BUNDLE_ROOT}/src-tauri/Cargo.toml"
cp src-tauri/Cargo.lock "${BUNDLE_ROOT}/src-tauri/Cargo.lock"
cp docs/distribution/BUILD.md "${BUNDLE_ROOT}/docs/distribution/BUILD.md"
cp -R "${TEMP_DIR}/vendor/." "${BUNDLE_ROOT}/vendor/"

if [ "${check_mode}" = true ]; then
  test -f "${BUNDLE_ROOT}/docs/distribution/BUILD.md"
  test -f "${BUNDLE_ROOT}/package-lock.json"
  test -f "${BUNDLE_ROOT}/src-tauri/Cargo.lock"
  test -d "${BUNDLE_ROOT}/vendor"
fi

tar -C "${TEMP_DIR}" -czf "${OUTPUT_PATH}" "gsd-dashboard-source-${VERSION}"

if [ "${check_mode}" = true ]; then
  echo "source bundle check passed"
else
  echo "source bundle written to ${OUTPUT_PATH}"
fi
