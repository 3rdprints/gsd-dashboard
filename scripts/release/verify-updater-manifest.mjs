#!/usr/bin/env node
import assert from "node:assert/strict";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

const REQUIRED_PLATFORMS = ["darwin-universal", "windows-x86_64", "linux-x86_64"];

function fail(message) {
  throw new Error(message);
}

function parseManifest(source, path) {
  try {
    return JSON.parse(source);
  } catch (error) {
    fail(`updater manifest is not valid JSON at ${path}: ${error.message}`);
  }
}

function isHttpsUrl(value) {
  if (typeof value !== "string") {
    return false;
  }
  try {
    return new URL(value).protocol === "https:";
  } catch {
    return false;
  }
}

function isInlineSignature(value) {
  if (typeof value !== "string" || value.trim() === "") {
    return false;
  }
  if (isHttpsUrl(value)) {
    return false;
  }
  return !/\.sig(?:[?#].*)?$/i.test(value.trim());
}

function isIsoUtcTimestamp(value) {
  if (typeof value !== "string") {
    return false;
  }
  const trimmed = value.trim();
  if (!/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d{1,3})?Z$/.test(trimmed)) {
    return false;
  }
  return !Number.isNaN(Date.parse(trimmed));
}

/**
 * Validates a generated updater manifest before release publication.
 */
export function validateUpdaterManifest(manifest) {
  if (typeof manifest?.version !== "string" || manifest.version.trim() === "") {
    fail("updater manifest requires a nonempty version");
  }

  if (!isIsoUtcTimestamp(manifest.pub_date)) {
    fail("updater manifest requires pub_date to be a valid UTC ISO-8601 timestamp");
  }

  const platforms = manifest.platforms;
  if (!platforms || typeof platforms !== "object" || Array.isArray(platforms)) {
    fail("updater manifest requires platforms object");
  }

  for (const platform of REQUIRED_PLATFORMS) {
    const entry = platforms[platform];
    if (!entry || typeof entry !== "object" || Array.isArray(entry)) {
      fail(`updater manifest missing platform ${platform}`);
    }
    if (!isHttpsUrl(entry.url)) {
      fail(`updater manifest ${platform}.url must be HTTPS`);
    }
    if (!isInlineSignature(entry.signature)) {
      fail(`updater manifest ${platform}.signature must be an inline nonempty signature, not a URL or .sig path`);
    }
  }
}

async function validatePath(path) {
  const source = await readFile(path, "utf8");
  validateUpdaterManifest(parseManifest(source, path));
}

function validManifestFixture() {
  return {
    version: "0.1.1",
    pub_date: "2026-05-03T00:00:00Z",
    platforms: {
      "darwin-universal": {
        url: "https://smacdonald.github.io/gsd-dashboard/downloads/GSD-Dashboard_0.1.1_universal.dmg",
        signature: "darwin-inline-signature"
      },
      "windows-x86_64": {
        url: "https://smacdonald.github.io/gsd-dashboard/downloads/GSD-Dashboard_0.1.1_x64_en-US.msi",
        signature: "windows-inline-signature"
      },
      "linux-x86_64": {
        url: "https://smacdonald.github.io/gsd-dashboard/downloads/gsd-dashboard_0.1.1_amd64.AppImage.tar.gz",
        signature: "linux-inline-signature"
      }
    }
  };
}

function invalidManifestFixture() {
  const manifest = validManifestFixture();
  manifest.platforms["windows-x86_64"].signature = "https://smacdonald.github.io/gsd-dashboard/downloads/GSD-Dashboard.sig";
  manifest.platforms["linux-x86_64"].signature = "downloads/linux.sig";
  return manifest;
}

function invalidPubDateFixture() {
  const manifest = validManifestFixture();
  manifest.pub_date = "not-a-date";
  return manifest;
}

/**
 * Runs fixture-based validation for this release helper.
 */
export async function runSelfTest() {
  const tempDir = mkdtempSync(join(tmpdir(), "gsd-updater-manifest-"));
  try {
    const validPath = join(tempDir, "latest-valid.json");
    const invalidPath = join(tempDir, "latest-invalid.json");
    const invalidPubDatePath = join(tempDir, "latest-invalid-pub-date.json");
    writeFileSync(validPath, `${JSON.stringify(validManifestFixture(), null, 2)}\n`);
    writeFileSync(invalidPath, `${JSON.stringify(invalidManifestFixture(), null, 2)}\n`);
    writeFileSync(invalidPubDatePath, `${JSON.stringify(invalidPubDateFixture(), null, 2)}\n`);

    await validatePath(validPath);
    await assert.rejects(() => validatePath(invalidPath), /signature/);
    await assert.rejects(() => validatePath(invalidPubDatePath), /pub_date to be a valid UTC ISO-8601 timestamp/);
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

  const manifestPath = args.find((arg) => !arg.startsWith("--"));
  if (!manifestPath) {
    fail("usage: node scripts/release/verify-updater-manifest.mjs <manifest-path>");
  }
  await validatePath(manifestPath);
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
});
