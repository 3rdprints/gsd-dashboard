#!/usr/bin/env node
import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { execFile } from "node:child_process";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { promisify } from "node:util";

import { generateUpdaterManifest } from "./generate-updater-manifest.mjs";

const execFileAsync = promisify(execFile);

async function withFixture(testFn) {
    const tempDir = await mkdtemp(join(tmpdir(), "gsd-generate-manifest-"));
  try {
    const artifactDir = join(tempDir, "artifacts");
    await mkdir(artifactDir);

    const artifacts = [
      ["GSD Dashboard_0.1.1_universal.dmg", "darwin-signature"],
      ["GSD Dashboard_0.1.1_x64_en-US.msi", "windows-signature"],
      ["gsd-dashboard_0.1.1_amd64.AppImage.tar.gz", "linux-signature"]
    ];
    for (const [filename, signature] of artifacts) {
      await writeFile(join(artifactDir, filename), "fixture artifact");
      await writeFile(join(artifactDir, `${filename}.sig`), `${signature}\n`);
    }

    await testFn({ tempDir, artifactDir });
  } finally {
    await rm(tempDir, { force: true, recursive: true });
  }
}

describe("generate-updater-manifest.mjs", () => {
  it("inlines signature file contents for all updater platforms", async () => {
    await withFixture(async ({ artifactDir }) => {
      const manifest = await generateUpdaterManifest({
        version: "0.1.1",
        releaseUrlBase: "https://horknfbr.github.io/gsd-dashboard/downloads/",
        artifactDir
      });

      assert.equal(manifest.version, "0.1.1");
      assert.equal(manifest.platforms["darwin-universal"].signature, "darwin-signature");
      assert.equal(manifest.platforms["windows-x86_64"].signature, "windows-signature");
      assert.equal(manifest.platforms["linux-x86_64"].signature, "linux-signature");
      assert.match(manifest.platforms["darwin-universal"].url, /\.dmg$/);
      assert.match(manifest.platforms["windows-x86_64"].url, /\.msi$/);
      assert.match(manifest.platforms["linux-x86_64"].url, /\.AppImage\.tar\.gz$/);
    });
  });

  it("fails when a required signature is missing", async () => {
    await withFixture(async ({ artifactDir }) => {
      await rm(join(artifactDir, "GSD Dashboard_0.1.1_x64_en-US.msi.sig"));

      await assert.rejects(
        () => generateUpdaterManifest({
          version: "0.1.1",
          releaseUrlBase: "https://horknfbr.github.io/gsd-dashboard/downloads/",
          artifactDir
        }),
        /missing signature file/
      );
    });
  });

  it("writes latest.json that passes verify-updater-manifest.mjs", async () => {
    await withFixture(async ({ tempDir, artifactDir }) => {
      const out = join(tempDir, "latest.json");
      await execFileAsync("node", [
        "scripts/release/generate-updater-manifest.mjs",
        "--version",
        "0.1.1",
        "--release-url-base",
        "https://horknfbr.github.io/gsd-dashboard/downloads/",
        "--artifact-dir",
        artifactDir,
        "--out",
        out
      ]);

      const manifest = JSON.parse(await readFile(out, "utf8"));
      assert.equal(manifest.platforms["linux-x86_64"].signature, "linux-signature");
      await execFileAsync("node", ["scripts/release/verify-updater-manifest.mjs", out]);
    });
  });
});
