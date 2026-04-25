import { readFileSync } from "node:fs";
import { resolve } from "node:path";

import { describe, expect, it } from "vitest";

describe("development config", () => {
  it("keeps the Vite dev server port aligned with the Tauri dev URL", () => {
    const packageJson = JSON.parse(readFileSync(resolve("package.json"), "utf8"));
    const tauriConfig = JSON.parse(readFileSync(resolve("src-tauri/tauri.conf.json"), "utf8"));
    const devUrl = new URL(tauriConfig.build.devUrl);

    expect(packageJson.scripts.dev).toContain(`--port ${devUrl.port}`);
    expect(packageJson.scripts.dev).toContain("--strictPort");
  });
});
