#!/usr/bin/env node
import { readdir, readFile } from "node:fs/promises";
import { join } from "node:path";

const ROOTS = ["src", "scripts"];
const SOURCE_EXTENSIONS = new Set([".ts", ".tsx", ".mjs"]);
const DEFAULT_THRESHOLD = 85;

function thresholdFromArgs(argv) {
  const thresholdIndex = argv.indexOf("--threshold");
  if (thresholdIndex === -1) {
    return DEFAULT_THRESHOLD;
  }

  const value = Number(argv[thresholdIndex + 1]);
  if (!Number.isFinite(value) || value < 0 || value > 100) {
    throw new Error("--threshold must be a number from 0 to 100");
  }
  return value;
}

function extensionFor(path) {
  const match = path.match(/\.[^.]+$/);
  return match ? match[0] : "";
}

async function collectSourceFiles(root) {
  const entries = await readdir(root, { withFileTypes: true });
  const files = [];

  for (const entry of entries) {
    const path = join(root, entry.name);
    if (entry.isDirectory()) {
      if (entry.name === "node_modules" || entry.name === "dist") {
        continue;
      }
      files.push(...await collectSourceFiles(path));
      continue;
    }

    if (entry.isFile() && SOURCE_EXTENSIONS.has(extensionFor(path)) && !path.includes(".test.")) {
      files.push(path);
    }
  }

  return files;
}

function isDocumented(lines, lineIndex) {
  let previous = lineIndex - 1;
  while (previous >= 0 && lines[previous].trim() === "") {
    previous -= 1;
  }
  if (previous < 0 || !lines[previous].includes("*/")) {
    return false;
  }

  for (let cursor = previous; cursor >= 0; cursor -= 1) {
    const text = lines[cursor].trim();
    if (text.includes("/**")) {
      return true;
    }
    if (cursor !== previous && text !== "" && !text.startsWith("*") && !text.startsWith("/*")) {
      return false;
    }
  }

  return false;
}

function publicFunctionName(line) {
  return line.match(/^export\s+(?:async\s+)?function\s+(\w+)/)?.[1] ||
    line.match(/^export\s+const\s+(\w+)\s*=\s*(?:async\s*)?(?:<[^>]+>\s*)?(?:\([^)]*\)|[A-Za-z_$][\w$]*)\s*=>/)?.[1] ||
    null;
}

async function measureFile(path) {
  const lines = (await readFile(path, "utf8")).split("\n");
  const findings = [];

  for (let index = 0; index < lines.length; index += 1) {
    const name = publicFunctionName(lines[index]);
    if (!name) {
      continue;
    }

    findings.push({
      documented: isDocumented(lines, index),
      line: index + 1,
      name,
      path
    });
  }

  return findings;
}

async function main() {
  const threshold = thresholdFromArgs(process.argv.slice(2));
  const files = (await Promise.all(ROOTS.map(collectSourceFiles))).flat();
  const findings = (await Promise.all(files.map(measureFile))).flat();
  const total = findings.length;
  const documented = findings.filter((finding) => finding.documented).length;
  const coverage = total === 0 ? 100 : (documented / total) * 100;

  if (coverage < threshold) {
    const missing = findings
      .filter((finding) => !finding.documented)
      .map((finding) => `${finding.path}:${finding.line} ${finding.name}`)
      .join("\n");
    throw new Error(`Doc-comment coverage ${coverage.toFixed(1)}% is below ${threshold}%.\n${missing}`);
  }

  console.log(`Doc-comment coverage ${coverage.toFixed(1)}% (${documented}/${total})`);
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
});
