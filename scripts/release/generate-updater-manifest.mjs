#!/usr/bin/env node
import { mkdir, readdir, readFile, writeFile } from "node:fs/promises";
import { basename, join } from "node:path";
import { fileURLToPath } from "node:url";

const PLATFORM_ARTIFACTS = {
  "darwin-universal": /\.app\.tar\.gz$/i,
  "windows-x86_64": /\.msi$/i,
  "linux-x86_64": /\.AppImage$/i
};

function fail(message) {
  throw new Error(message);
}

function parseArgs(argv) {
  const args = {};
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (!arg.startsWith("--")) {
      fail(`unexpected argument: ${arg}`);
    }
    const key = arg.slice(2);
    const value = argv[index + 1];
    if (!value || value.startsWith("--")) {
      fail(`missing value for --${key}`);
    }
    args[key] = value;
    index += 1;
  }
  return args;
}

function requireArg(args, key) {
  const value = args[key];
  if (typeof value !== "string" || value.trim() === "") {
    fail(`--${key} is required`);
  }
  return value;
}

function normalizeUrlBase(value) {
  return value.endsWith("/") ? value : `${value}/`;
}

function assetUrl(urlBase, filename) {
  return new URL(encodeURI(filename), normalizeUrlBase(urlBase)).toString();
}

async function findArtifactEntries(artifactDir) {
  const filenames = await readdir(artifactDir);
  const entries = {};

  for (const [platform, pattern] of Object.entries(PLATFORM_ARTIFACTS)) {
    const filename = filenames.find((candidate) => pattern.test(candidate));
    if (!filename) {
      fail(`missing ${platform} update artifact matching ${pattern}`);
    }

    const signaturePath = join(artifactDir, `${filename}.sig`);
    let signature;
    try {
      signature = (await readFile(signaturePath, "utf8")).trim();
    } catch (error) {
      fail(`missing signature file for ${filename}: ${error.message}`);
    }
    if (!signature) {
      fail(`signature file for ${filename} is empty`);
    }

    entries[platform] = { filename, signature };
  }

  return entries;
}

export async function generateUpdaterManifest({
  version,
  releaseUrlBase,
  artifactDir
}) {
  const entries = await findArtifactEntries(artifactDir);
  const platforms = {};

  for (const [platform, entry] of Object.entries(entries)) {
    platforms[platform] = {
      url: assetUrl(releaseUrlBase, entry.filename),
      signature: entry.signature
    };
  }

  return {
    version,
    notes: `GSD Dashboard ${version}`,
    pub_date: new Date().toISOString(),
    platforms
  };
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  const version = requireArg(args, "version");
  const releaseUrlBase = requireArg(args, "release-url-base");
  const artifactDir = requireArg(args, "artifact-dir");
  const out = requireArg(args, "out");

  const manifest = await generateUpdaterManifest({ version, releaseUrlBase, artifactDir });
  await mkdir(join(out, ".."), { recursive: true });
  await writeFile(out, `${JSON.stringify(manifest, null, 2)}\n`);
  console.log(`wrote latest.json to ${basename(out) === "latest.json" ? out : basename(out)}`);
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  main().catch((error) => {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  });
}
