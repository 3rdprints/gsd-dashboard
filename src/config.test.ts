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

  it("keeps the main Tauri window hidden until startup decides visibility", () => {
    const tauriConfig = JSON.parse(readFileSync(resolve("src-tauri/tauri.conf.json"), "utf8"));
    const mainWindow = tauriConfig.app.windows.find(
      (windowConfig: { label?: string }) => windowConfig.label === "main"
    );

    expect(mainWindow).toMatchObject({ visible: false });
  });

  it("allows Finder reveal and scopes VS Code file URLs", () => {
    const capability = JSON.parse(
      readFileSync(resolve("src-tauri/capabilities/default.json"), "utf8")
    );

    expect(capability.permissions).toContain("opener:allow-reveal-item-in-dir");
    expect(capability.permissions).toContainEqual({
      identifier: "opener:allow-open-url",
      allow: [{ url: "vscode://file/*" }]
    });
  });
});
