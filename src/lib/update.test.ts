import { beforeEach, describe, expect, it, vi } from "vitest";

import { checkForUpdate, installAndRestart } from "./update";

const checkMock = vi.fn();
const relaunchMock = vi.fn();

vi.mock("@tauri-apps/plugin-updater", () => ({
  check: checkMock
}));

vi.mock("@tauri-apps/plugin-process", () => ({
  relaunch: relaunchMock
}));

describe("update wrapper", () => {
  beforeEach(() => {
    delete (window as Window & { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
    checkMock.mockReset();
    relaunchMock.mockReset();
  });

  it("returns unsupported when Tauri internals are absent", async () => {
    await expect(checkForUpdate()).resolves.toEqual({ state: "unsupported" });
    expect(checkMock).not.toHaveBeenCalled();
  });

  it("returns a nonblocking error state when update checks fail", async () => {
    Object.defineProperty(window, "__TAURI_INTERNALS__", {
      configurable: true,
      value: {}
    });
    checkMock.mockRejectedValue(new Error("network unavailable"));

    await expect(checkForUpdate()).resolves.toEqual({
      state: "error",
      message:
        "Update check failed. The dashboard will keep running on this version; check your network or try again later."
    });
  });

  it("installs before relaunching and ignores missing updates", async () => {
    const update = {
      downloadAndInstall: vi.fn().mockResolvedValue(undefined)
    };

    await installAndRestart(null);
    expect(update.downloadAndInstall).not.toHaveBeenCalled();
    expect(relaunchMock).not.toHaveBeenCalled();

    await installAndRestart(update);

    expect(update.downloadAndInstall).toHaveBeenCalledTimes(1);
    expect(relaunchMock).toHaveBeenCalledTimes(1);
    expect(update.downloadAndInstall.mock.invocationCallOrder[0]).toBeLessThan(
      relaunchMock.mock.invocationCallOrder[0]
    );
  });
});
