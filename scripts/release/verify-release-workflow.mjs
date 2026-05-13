#!/usr/bin/env node
import assert from "node:assert/strict";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

const DEFAULT_WORKFLOW_PATH = ".github/workflows/release.yml";
const REQUIRED_TAG_PATTERN = "v*.*.*";
const REQUIRED_OS_VALUES = [
  "blacksmith-6vcpu-macos-latest",
  "blacksmith-2vcpu-windows-2025",
  "blacksmith-2vcpu-ubuntu-2404"
];
const REQUIRED_PERMISSIONS = ["contents: write", "pages: write", "id-token: write"];
const REQUIRED_BASE_URL = "https://3rdprints.github.io/gsd-dashboard";

function fail(message) {
  throw new Error(message);
}

function requireIncludes(source, needle, description) {
  if (!source.includes(needle)) {
    fail(`release workflow missing ${description}: ${needle}`);
  }
}

function requireRegex(source, pattern, description) {
  if (!pattern.test(source)) {
    fail(`release workflow missing ${description}`);
  }
}

function requireCanonicalBaseUrl(source) {
  const match = source.match(/(?:GSD_DASHBOARD_BASE_URL|base_url)=["']?\$\{GSD_DASHBOARD_BASE_URL:-([^}]+)\}/);
  if (!match) {
    fail("release workflow missing GSD_DASHBOARD_BASE_URL defaulting behavior");
  }

  let url;
  try {
    url = new URL(match[1]);
  } catch {
    fail(`release workflow GSD_DASHBOARD_BASE_URL default is not a valid URL: ${match[1]}`);
  }

  if (url.protocol !== "https:" || url.hostname !== "3rdprints.github.io") {
    fail(`release workflow GSD_DASHBOARD_BASE_URL default must use ${REQUIRED_BASE_URL}`);
  }

  if (url.pathname.replace(/\/$/, "") !== "/gsd-dashboard") {
    fail(`release workflow GSD_DASHBOARD_BASE_URL default must use ${REQUIRED_BASE_URL}`);
  }
}

function topLevelBlock(source, blockName) {
  const lines = source.split("\n");
  const start = lines.findIndex((line) => line === `${blockName}:`);
  if (start === -1) {
    return "";
  }

  const block = [lines[start]];
  for (const line of lines.slice(start + 1)) {
    if (/^\S/.test(line)) {
      break;
    }
    block.push(line);
  }
  return block.join("\n");
}

function firstIndentedBlock(source, blockName) {
  const lines = source.split("\n");
  const start = lines.findIndex((line) => line.trim() === `${blockName}:`);
  if (start === -1) {
    return "";
  }

  const indent = lines[start].match(/^\s*/)?.[0].length ?? 0;
  const block = [lines[start]];
  for (const line of lines.slice(start + 1)) {
    if (line.trim() !== "" && (line.match(/^\s*/)?.[0].length ?? 0) <= indent) {
      break;
    }
    block.push(line);
  }
  return block.join("\n");
}

/**
 * Validates release workflow permissions, matrix, and publishing gates.
 */
export function validateReleaseWorkflow(source) {
  requireRegex(source, /push:\s*(?:\n[\s\S]*?)tags:\s*(?:\n[\s\S]*?-\s*["']?v\*\.\*\.\*["']?|\[[^\]]*["']?v\*\.\*\.\*["']?[^\]]*\])/m, `push.tags ${REQUIRED_TAG_PATTERN}`);

  const permissionsBlock = topLevelBlock(source, "permissions");
  for (const permission of REQUIRED_PERMISSIONS) {
    requireIncludes(permissionsBlock, permission, `least-privilege permission ${permission}`);
  }

  const matrixBlock = firstIndentedBlock(source, "matrix");
  for (const osValue of REQUIRED_OS_VALUES) {
    requireIncludes(matrixBlock, osValue, `matrix OS ${osValue}`);
  }

  requireIncludes(source, "rustup target add aarch64-apple-darwin x86_64-apple-darwin", "macOS universal Rust targets");
  requireIncludes(source, "--target universal-apple-darwin --bundles app,dmg", "macOS universal app and DMG build command");
  requireRegex(source, /lipo\s+-archs|universal[\w.-]*\.dmg|\.dmg[\s\S]*universal/i, "universal DMG assertion");
  requireIncludes(source, "TAURI_SIGNING_PRIVATE_KEY", "updater signing secret gate");
  requireRegex(source, /unsigned[\s\S]{0,120}(artifact|installer|build|caveat)|artifact[\s\S]{0,120}unsigned/i, "unsigned artifact caveat text");
  requireCanonicalBaseUrl(source);
  requireRegex(source, /npm ci[\s\S]{0,160}npm run release:verify-tauri-config[\s\S]{0,160}npm run build[\s\S]{0,800}Generate updater manifest/, "release config install and smoke gate before updater manifest");
  requireIncludes(source, "site-dist/releases/latest/download", "Pages latest-download compatibility directory");
  requireIncludes(source, "GSD-Dashboard.dmg", "stable macOS DMG alias");
  requireIncludes(source, "GSD-Dashboard.msi", "stable Windows MSI alias");
  requireIncludes(source, "GSD-Dashboard.exe", "stable Windows EXE alias");
  requireIncludes(source, "gsd-dashboard.deb", "stable Linux DEB alias");
  requireIncludes(source, "gsd-dashboard.rpm", "stable Linux RPM alias");
  requireIncludes(source, "gsd-dashboard.AppImage", "stable Linux AppImage alias");
  requireIncludes(source, "site-dist/releases/latest/download/GSD-Dashboard.dmg", "macOS release alias upload");
  requireIncludes(source, "site-dist/releases/latest/download/GSD-Dashboard.msi", "Windows MSI release alias upload");
  requireIncludes(source, "site-dist/releases/latest/download/GSD-Dashboard.exe", "Windows EXE release alias upload");
  requireIncludes(source, "site-dist/releases/latest/download/gsd-dashboard.deb", "Linux DEB release alias upload");
  requireIncludes(source, "site-dist/releases/latest/download/gsd-dashboard.rpm", "Linux RPM release alias upload");
  requireIncludes(source, "site-dist/releases/latest/download/gsd-dashboard.AppImage", "Linux release alias upload");
  requireIncludes(source, "actions/upload-pages-artifact", "Pages artifact upload action");
  requireIncludes(source, "actions/deploy-pages", "Pages deploy action");
}

async function validatePath(path) {
  const source = await readFile(path, "utf8");
  validateReleaseWorkflow(source);
}

function validWorkflowFixture() {
  return `name: release

on:
  push:
    tags:
      - "v*.*.*"

permissions:
  contents: write
  pages: write
  id-token: write

jobs:
  build:
    strategy:
      matrix:
        os:
          - blacksmith-6vcpu-macos-latest
          - blacksmith-2vcpu-windows-2025
          - blacksmith-2vcpu-ubuntu-2404
    runs-on: \${{ matrix.os }}
    steps:
      - uses: actions/checkout@v6
      - name: Configure macOS universal targets
        run: rustup target add aarch64-apple-darwin x86_64-apple-darwin
      - name: Build macOS universal app and DMG
        run: npm run tauri build -- --target universal-apple-darwin --bundles app,dmg
      - name: Assert universal DMG
        run: |
          lipo -archs src-tauri/target/universal-apple-darwin/release/bundle/macos/GSD\\ Dashboard.app/Contents/MacOS/gsd-dashboard
          test -f "src-tauri/target/universal-apple-darwin/release/bundle/dmg/GSD Dashboard universal.dmg"
      - name: Gate updater signing
        env:
          TAURI_SIGNING_PRIVATE_KEY: \${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
        run: test -n "$TAURI_SIGNING_PRIVATE_KEY"
      - name: Document unsigned artifact caveat
        run: echo "Unsigned installer artifacts are published only with explicit caveat text."
      - name: Set base URL default
        run: echo "GSD_DASHBOARD_BASE_URL=\${GSD_DASHBOARD_BASE_URL:-https://3rdprints.github.io/gsd-dashboard}" >> "$GITHUB_ENV"
      - name: Verify release config and build smoke
        run: |
          npm ci
          npm run release:verify-tauri-config
          npm run build
      - name: Generate updater manifest
        run: node scripts/release/generate-updater-manifest.mjs
      - name: Create stable Pages download aliases
        run: |
          mkdir -p site-dist/releases/latest/download
          cp GSD.Dashboard_0.1.2_universal.dmg site-dist/downloads/GSD-Dashboard.dmg
          cp GSD.Dashboard_0.1.2_x64_en-US.msi site-dist/downloads/GSD-Dashboard.msi
          cp GSD.Dashboard_0.1.2_x64-setup.exe site-dist/downloads/GSD-Dashboard.exe
          cp GSD.Dashboard_0.1.2_amd64.deb site-dist/downloads/gsd-dashboard.deb
          cp GSD.Dashboard-0.1.2-1.x86_64.rpm site-dist/downloads/gsd-dashboard.rpm
          cp GSD.Dashboard_0.1.2_amd64.AppImage site-dist/downloads/gsd-dashboard.AppImage
          cp site-dist/downloads/GSD-Dashboard.dmg site-dist/releases/latest/download/GSD-Dashboard.dmg
          gh release upload "$GITHUB_REF_NAME" \
            site-dist/releases/latest/download/GSD-Dashboard.dmg \
            site-dist/releases/latest/download/GSD-Dashboard.msi \
            site-dist/releases/latest/download/GSD-Dashboard.exe \
            site-dist/releases/latest/download/gsd-dashboard.deb \
            site-dist/releases/latest/download/gsd-dashboard.rpm \
            site-dist/releases/latest/download/gsd-dashboard.AppImage \
            --clobber
      - uses: actions/upload-pages-artifact@v4
      - uses: actions/deploy-pages@v4
`;
}

function invalidWorkflowFixture() {
  return `name: release
on:
  push:
    branches: [main]
permissions:
  contents: read
jobs: {}
`;
}

/**
 * Runs fixture-based validation for this release helper.
 */
export async function runSelfTest() {
  const tempDir = mkdtempSync(join(tmpdir(), "gsd-release-workflow-"));
  try {
    const validPath = join(tempDir, "valid.yml");
    const invalidPath = join(tempDir, "invalid.yml");
    writeFileSync(validPath, validWorkflowFixture());
    writeFileSync(invalidPath, invalidWorkflowFixture());

    await validatePath(validPath);
    await assert.rejects(() => validatePath(invalidPath), /missing/);
  } finally {
    rmSync(tempDir, { force: true, recursive: true });
  }
}

async function main() {
  const args = process.argv.slice(2);
  const knownFlags = new Set(["--self-test", "--matrix"]);
  const unknownFlags = args.filter((arg) => arg.startsWith("--") && !knownFlags.has(arg));
  if (unknownFlags.length > 0) {
    fail(`unknown option(s): ${unknownFlags.join(", ")}`);
  }

  if (args.includes("--self-test")) {
    await runSelfTest();
    return;
  }

  const workflowPath = args.find((arg) => !arg.startsWith("--")) ?? DEFAULT_WORKFLOW_PATH;
  await validatePath(workflowPath);

  if (args.includes("--matrix")) {
    console.log(`release workflow matrix validated: ${REQUIRED_OS_VALUES.join(", ")}`);
  }
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
});
