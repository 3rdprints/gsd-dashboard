import { readFileSync } from "node:fs";
import { resolve } from "node:path";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import "@testing-library/jest-dom/vitest";
import { render, screen } from "@testing-library/react";
import type React from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { App } from "./App";
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

describe("Phase 1 shell", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockImplementation((command: string) => {
      if (command === "get_boot_status") {
        return Promise.resolve({
          appDataDir: "/tmp/gsd-dashboard",
          cachePath: "/tmp/gsd-dashboard/cache.db",
          cacheReady: true,
          walEnabled: true,
          migrationsApplied: 1,
          settingsInitialized: true
        });
      }

      if (command === "get_settings") {
        return Promise.resolve({
          scanRoots: ["~/Documents"],
          hiddenProjectIds: [],
          autostartEnabled: false,
          trayBarMaxProjects: 8,
          trayBarSort: "recent_activity"
        });
      }

      return Promise.reject(new Error(`Unexpected command: ${command}`));
    });
  });

  it("renders boot, cache, settings, and empty dashboard states", async () => {
    renderWithQueryClient(<App />);

    expect(await screen.findByRole("heading", { name: "GSD Dashboard" })).toBeInTheDocument();
    expect(await screen.findByText("Cache ready")).toBeInTheDocument();
    expect(screen.getByText("Migrations applied")).toBeInTheDocument();
    expect(screen.getByText("Settings saved")).toBeInTheDocument();
    expect(screen.getByText("Default scan root")).toBeInTheDocument();
    expect(screen.getByText("No projects scanned yet")).toBeInTheDocument();
    expect(
      screen.getByText(
        "GSD Dashboard is initialized with ~/Documents as the default scan root. Project discovery starts in the next phase."
      )
    ).toBeInTheDocument();
  });

  it("displays the first-run default scan root exactly", async () => {
    renderWithQueryClient(<App />);

    const rootInput = await screen.findByLabelText("Default scan root");

    expect(rootInput).toHaveValue("~/Documents");
  });

  it("does not render Phase 3 dashboard controls or data surfaces", async () => {
    renderWithQueryClient(<App />);

    await screen.findByText("No projects scanned yet");

    expect(screen.queryByText("Rebuild cache")).not.toBeInTheDocument();
    expect(screen.queryByText("Scan now")).not.toBeInTheDocument();
    expect(screen.queryByText(/session/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/chart/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/tray/i)).not.toBeInTheDocument();
    expect(screen.queryByText("Hidden projects")).not.toBeInTheDocument();
  });
});

function renderWithQueryClient(ui: React.ReactElement) {
  const testQueryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false }
    }
  });

  render(<QueryClientProvider client={testQueryClient}>{ui}</QueryClientProvider>);

  return testQueryClient;
}
