#!/usr/bin/env node
import assert from "node:assert/strict";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

const DEFAULT_CONFIG_PATH = "src-tauri/tauri.conf.json";
const REQUIRED_TARGETS = ["dmg", "msi", "nsis", "deb", "appimage", "rpm"];
const REQUIRED_ENDPOINT_SUFFIX = "/updates/latest.json";

function fail(message) {
  throw new Error(message);
}

function parseConfig(source, path) {
  try {
    return JSON.parse(source);
  } catch (error) {
    fail(`tauri config is not valid JSON at ${path}: ${error.message}`);
  }
}

function hasRequiredTargets(targets) {
  if (targets === "all") {
    return true;
  }
  if (!Array.isArray(targets)) {
    return false;
  }
  const normalizedTargets = new Set(targets.map((target) => String(target).toLowerCase()));
  return REQUIRED_TARGETS.every((target) => normalizedTargets.has(target));
}

/**
 * Validates Tauri bundle and updater configuration invariants.
 */
export function validateTauriConfig(config) {
  if (config?.bundle?.active !== true) {
    fail("tauri config requires bundle.active true");
  }

  if (!hasRequiredTargets(config.bundle.targets)) {
    fail(`tauri config bundle.targets must be "all" or include ${REQUIRED_TARGETS.join(", ")}`);
  }

  if (config.bundle.createUpdaterArtifacts !== true) {
    fail("tauri config requires bundle.createUpdaterArtifacts true");
  }

  const updater = config?.plugins?.updater;
  if (!updater || typeof updater.pubkey !== "string" || updater.pubkey.trim() === "") {
    fail("tauri config requires plugins.updater.pubkey");
  }

  const firstEndpoint = updater.endpoints?.[0];
  if (typeof firstEndpoint !== "string") {
    fail("tauri config requires plugins.updater.endpoints[0]");
  }

  let endpointUrl;
  try {
    endpointUrl = new URL(firstEndpoint);
  } catch {
    fail("tauri config updater endpoint must be a valid HTTPS URL");
  }

  if (endpointUrl.protocol !== "https:" || !firstEndpoint.endsWith(REQUIRED_ENDPOINT_SUFFIX)) {
    fail(`tauri config updater endpoint must be HTTPS and end with ${REQUIRED_ENDPOINT_SUFFIX}`);
  }

  if (updater.windows?.installMode !== "passive") {
    fail("tauri config requires plugins.updater.windows.installMode passive");
  }
}

async function validatePath(path) {
  const source = await readFile(path, "utf8");
  validateTauriConfig(parseConfig(source, path));
}

function validConfigFixture() {
  return {
    bundle: {
      active: true,
      targets: ["dmg", "msi", "nsis", "deb", "appimage", "rpm"],
      createUpdaterArtifacts: true
    },
    plugins: {
      updater: {
        pubkey: "trusted-public-key",
        endpoints: ["https://smacdonald.github.io/gsd-dashboard/updates/latest.json"],
        windows: {
          installMode: "passive"
        }
      }
    }
  };
}

function invalidConfigFixture() {
  return {
    bundle: {
      active: true,
      targets: ["dmg"],
      createUpdaterArtifacts: false
    },
    plugins: {
      updater: {
        pubkey: "",
        endpoints: ["http://example.test/latest.json"],
        windows: {
          installMode: "basicUi"
        }
      }
    }
  };
}

/**
 * Runs fixture-based validation for this release helper.
 */
export async function runSelfTest() {
  const tempDir = mkdtempSync(join(tmpdir(), "gsd-tauri-config-"));
  try {
    const validPath = join(tempDir, "valid.json");
    const invalidPath = join(tempDir, "invalid.json");
    writeFileSync(validPath, `${JSON.stringify(validConfigFixture(), null, 2)}\n`);
    writeFileSync(invalidPath, `${JSON.stringify(invalidConfigFixture(), null, 2)}\n`);

    await validatePath(validPath);
    await assert.rejects(() => validatePath(invalidPath), /requires|must/);
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

  const configPath = args.find((arg) => !arg.startsWith("--")) ?? DEFAULT_CONFIG_PATH;
  await validatePath(configPath);

  if (args.includes("--updater")) {
    console.log("tauri updater config validated");
  }
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
});
