#!/usr/bin/env bash
set -euo pipefail

# Syncs package.json, tauri.conf.json, and Cargo.toml versions from a git tag.
# Usage: TAG_VERSION=v1.2.3 ./scripts/release/sync-version-from-tag.sh

if [ -z "${TAG_VERSION:-}" ]; then
  echo "TAG_VERSION is required (e.g. v1.2.3)" >&2
  exit 1
fi

VERSION="${TAG_VERSION#v}"
echo "Syncing version to ${VERSION} from tag ${TAG_VERSION}"

VERSION="$VERSION" node -e "
  const fs = require('fs');
  const v = process.env.VERSION;
  for (const f of ['package.json', 'package-lock.json']) {
    const obj = JSON.parse(fs.readFileSync(f, 'utf8'));
    obj.version = v;
    if (f === 'package-lock.json' && obj.packages?.['']) obj.packages[''].version = v;
    fs.writeFileSync(f, JSON.stringify(obj, null, 2) + '\n');
  }
"

VERSION="$VERSION" node -e "
  const fs = require('fs');
  const v = process.env.VERSION;
  const cfg = JSON.parse(fs.readFileSync('src-tauri/tauri.conf.json', 'utf8'));
  cfg.version = v;
  fs.writeFileSync('src-tauri/tauri.conf.json', JSON.stringify(cfg, null, 2) + '\n');
"

sed -i.bak "s/^version = \".*\"/version = \"${VERSION}\"/" src-tauri/Cargo.toml && rm -f src-tauri/Cargo.toml.bak
