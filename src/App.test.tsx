import { readFileSync } from "node:fs";
import { resolve } from "node:path";

import { QueryClient } from "@tanstack/react-query";
import { beforeEach, describe, expect, it, vi } from "vitest";

import type { SettingsInput } from "./lib/types";

const invokeMock = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock
}));

describe("Phase 1 IPC plumbing", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("calls the exact boot and settings command names", async () => {
    const { getBootStatus, getSettings, saveSettings } = await import("./lib/ipc");
    const settingsInput: SettingsInput = {
      scanRoots: ["~/Documents"],
      hiddenProjectIds: [],
      autostartEnabled: false,
      trayBarMaxProjects: 8,
      trayBarSort: "recent_activity"
    };

    invokeMock.mockResolvedValue({});

    await getBootStatus();
    await getSettings();
    await saveSettings(settingsInput);

    expect(invokeMock).toHaveBeenNthCalledWith(1, "get_boot_status");
    expect(invokeMock).toHaveBeenNthCalledWith(2, "get_settings");
    expect(invokeMock).toHaveBeenNthCalledWith(3, "save_settings", { input: settingsInput });
  });

  it("provides the query client at the app root", () => {
    const mainSource = readFileSync(resolve("src/main.tsx"), "utf8");

    expect(mainSource).toContain("QueryClientProvider");
    expect(mainSource).toContain("queryClient");
  });

  it("invalidates settings only after a successful settings save", async () => {
    const { createSaveSettingsMutationOptions, settingsQueryKey } = await import(
      "./lib/queryClient"
    );
    const queryClient = new QueryClient();
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
    const settingsInput: SettingsInput = {
      scanRoots: ["~/Documents"],
      hiddenProjectIds: [],
      autostartEnabled: false,
      trayBarMaxProjects: 8,
      trayBarSort: "recent_activity"
    };

    invokeMock.mockRejectedValueOnce({ kind: "store", message: "save failed" });
    await expect(createSaveSettingsMutationOptions(queryClient).mutationFn(settingsInput)).rejects
      .toEqual({ kind: "store", message: "save failed" });
    expect(invalidateSpy).not.toHaveBeenCalled();

    invokeMock.mockResolvedValueOnce(settingsInput);
    await createSaveSettingsMutationOptions(queryClient).mutationFn(settingsInput);
    await createSaveSettingsMutationOptions(queryClient).onSuccess?.(settingsInput);

    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: settingsQueryKey });
  });
});
