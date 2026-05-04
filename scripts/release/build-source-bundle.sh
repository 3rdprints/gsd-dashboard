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

mkdir -p "${BUNDLE_ROOT}"

npm ci
npm run build

(cd src-tauri && cargo vendor "${TEMP_DIR}/vendor")

git archive HEAD | tar -x -C "${BUNDLE_ROOT}"
rm -rf "${BUNDLE_ROOT}/src-tauri/dist"
cp -R src-tauri/dist "${BUNDLE_ROOT}/src-tauri/dist"
mkdir -p "${BUNDLE_ROOT}/vendor"
cp -R "${TEMP_DIR}/vendor/." "${BUNDLE_ROOT}/vendor/"

if [ "${check_mode}" = true ]; then
  test -f "${BUNDLE_ROOT}/docs/distribution/BUILD.md"
  test -f "${BUNDLE_ROOT}/package.json"
  test -f "${BUNDLE_ROOT}/package-lock.json"
  test -f "${BUNDLE_ROOT}/src-tauri/Cargo.toml"
  test -f "${BUNDLE_ROOT}/src-tauri/Cargo.lock"
  test -d "${BUNDLE_ROOT}/vendor"
  echo "source bundle check passed"
else
  mkdir -p "${OUTPUT_DIR}"
  tar -C "${TEMP_DIR}" -czf "${OUTPUT_PATH}" "gsd-dashboard-source-${VERSION}"
  cp "${OUTPUT_PATH}" "${OUTPUT_DIR}/gsd-dashboard-source-latest.tar.gz"
  echo "source bundle written to ${OUTPUT_PATH}"
fi
