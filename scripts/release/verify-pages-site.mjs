#!/usr/bin/env node
import assert from "node:assert/strict";
import { existsSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

const SOURCE_SITE_DIR = "docs/public";
const BUILT_SITE_DIR = "site-dist";

const REQUIRED_HTML_FRAGMENTS = [
  "GSD Dashboard",
  "Download for macOS",
  "cargo install gsd-dashboard",
  "/updates/latest.json",
  "Updater manifest",
  ".dmg",
  ".msi",
  ".deb",
  ".rpm",
  ".AppImage"
];

const REQUIRED_INSTALL_FRAGMENTS = [
  "--yes",
  "uname -s",
  "uname -m",
  "Install `${artifact}` for `${os}/${arch}`?",
  "curl -fsSL",
  "GSD_DASHBOARD_BASE_URL"
];

function fail(message) {
  throw new Error(message);
}

function resolveDefaultPaths() {
  const siteDir = existsSync(BUILT_SITE_DIR) ? BUILT_SITE_DIR : SOURCE_SITE_DIR;
  return {
    htmlPath: join(siteDir, "index.html"),
    installPath: join(siteDir, "install.sh")
  };
}

function requireFragments(source, fragments, label) {
  const missingFragments = fragments.filter((fragment) => !source.includes(fragment));
  if (missingFragments.length > 0) {
    fail(`${label} missing required content: ${missingFragments.join(", ")}`);
  }
}

export function validatePagesSite(html, installScript) {
  requireFragments(html, REQUIRED_HTML_FRAGMENTS, "Pages HTML");
  requireFragments(installScript, REQUIRED_INSTALL_FRAGMENTS, "install.sh");
}

async function validatePaths({ htmlPath, installPath }) {
  const [html, installScript] = await Promise.all([
    readFile(htmlPath, "utf8"),
    readFile(installPath, "utf8")
  ]);
  validatePagesSite(html, installScript);
}

function validHtmlFixture() {
  return `<!doctype html>
<html lang="en">
  <head><title>GSD Dashboard</title></head>
  <body>
    <h1>GSD Dashboard</h1>
    <a href="./downloads/GSD-Dashboard.dmg">Download for macOS</a>
    <a href="./downloads/GSD-Dashboard.msi">Windows .msi</a>
    <a href="./downloads/gsd-dashboard.deb">Linux .deb</a>
    <a href="./downloads/gsd-dashboard.rpm">Linux .rpm</a>
    <a href="./downloads/gsd-dashboard.AppImage">Linux .AppImage</a>
    <code>cargo install gsd-dashboard</code>
    <a href="/updates/latest.json">Updater manifest</a>
  </body>
</html>
`;
}

function validInstallFixture() {
  return `#!/bin/sh
set -eu

yes_flag=false
for arg in "$@"; do
  if [ "$arg" = "--yes" ]; then
    yes_flag=true
  fi
done

os="$(uname -s)"
arch="$(uname -m)"
artifact="GSD-Dashboard.dmg"
base_url="\${GSD_DASHBOARD_BASE_URL:-https://smacdonald.github.io/gsd-dashboard}"
printf 'Install \`%s\` for \`%s/%s\`? ' "$artifact" "$os" "$arch"
prompt="Install \`\${artifact}\` for \`\${os}/\${arch}\`?"
curl -fsSL "$base_url/downloads/$artifact" -o "$artifact"
`;
}

function invalidHtmlFixture() {
  return "<!doctype html><title>Downloads</title>";
}

function invalidInstallFixture() {
  return "#!/bin/sh\ncurl http://example.test\n";
}

export async function runSelfTest() {
  const tempDir = mkdtempSync(join(tmpdir(), "gsd-pages-site-"));
  try {
    const validHtmlPath = join(tempDir, "valid.html");
    const validInstallPath = join(tempDir, "valid-install.sh");
    const invalidHtmlPath = join(tempDir, "invalid.html");
    const invalidInstallPath = join(tempDir, "invalid-install.sh");
    writeFileSync(validHtmlPath, validHtmlFixture());
    writeFileSync(validInstallPath, validInstallFixture());
    writeFileSync(invalidHtmlPath, invalidHtmlFixture());
    writeFileSync(invalidInstallPath, invalidInstallFixture());

    await validatePaths({ htmlPath: validHtmlPath, installPath: validInstallPath });
    await assert.rejects(
      () => validatePaths({ htmlPath: invalidHtmlPath, installPath: invalidInstallPath }),
      /missing required content/
    );
  } finally {
    rmSync(tempDir, { force: true, recursive: true });
  }
}

async function main() {
  const args = process.argv.slice(2);
  if (args.includes("--self-test")) {
    await runSelfTest();
    return;
  }

  const positionalArgs = args.filter((arg) => !arg.startsWith("--"));
  const paths = positionalArgs.length >= 2
    ? { htmlPath: positionalArgs[0], installPath: positionalArgs[1] }
    : resolveDefaultPaths();
  await validatePaths(paths);
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
});
