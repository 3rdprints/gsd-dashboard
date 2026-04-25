import { readFileSync } from "node:fs";
import { resolve } from "node:path";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import "@testing-library/jest-dom/vitest";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import type React from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { App } from "./App";
import type { ScanEvent, SettingsInput } from "./lib/types";

const { channelInstances, invokeMock, openPathMock, openUrlMock, writeTextMock } = vi.hoisted(() => ({
  channelInstances: [] as Array<{ onmessage: ((event: unknown) => void) | null }>,
  invokeMock: vi.fn(),
  openPathMock: vi.fn(),
  openUrlMock: vi.fn(),
  writeTextMock: vi.fn()
}));

vi.mock("@tauri-apps/api/core", () => ({
  Channel: class TestChannel<T> {
    onmessage: ((event: T) => void) | null = null;

    constructor() {
      channelInstances.push(this as { onmessage: ((event: unknown) => void) | null });
    }
  },
  invoke: invokeMock
}));

vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({
  writeText: writeTextMock
}));

vi.mock("@tauri-apps/plugin-opener", () => ({
  openPath: openPathMock,
  openUrl: openUrlMock
}));

const defaultSettings: SettingsInput = {
  scanRoots: ["~/Documents"],
  hiddenProjectIds: [],
  autostartEnabled: false,
  trayBarMaxProjects: 8,
  trayBarSort: "recent_activity"
};

describe("Phase 1 IPC plumbing", () => {
  beforeEach(() => {
    channelInstances.length = 0;
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

describe("Phase 2 scan IPC plumbing", () => {
  beforeEach(() => {
    channelInstances.length = 0;
    invokeMock.mockReset();
  });

  it("calls the exact scan command name with a typed event channel", async () => {
    const { scanProjects } = await import("./lib/ipc");
    const onEvent = vi.fn();
    const summary = {
      discoveredCount: 1,
      parsedCount: 1,
      errorCount: 0
    };

    invokeMock.mockResolvedValue(summary);

    await expect(scanProjects(onEvent)).resolves.toEqual(summary);

    expect(channelInstances).toHaveLength(1);
    expect(invokeMock).toHaveBeenCalledWith("scan_projects", {
      onEvent: channelInstances[0]
    });

    const event: ScanEvent = {
      event: "projectFound",
      data: {
        projectId: "deckpilot-web",
        projectName: "DeckPilot",
        rootPath: "/Users/smacdonald/homegit/deckpilot-web"
      }
    };
    channelInstances[0].onmessage?.(event);

    expect(onEvent).toHaveBeenCalledWith(event);
  });

  it("keeps scan event fixtures metadata-only without raw planning document bodies", () => {
    const events: ScanEvent[] = [
      { event: "started", data: { rootCount: 1 } },
      { event: "rootStarted", data: { rootPath: "/Users/smacdonald/homegit" } },
      {
        event: "projectFound",
        data: {
          projectId: "deckpilot-web",
          projectName: "DeckPilot",
          rootPath: "/Users/smacdonald/homegit/deckpilot-web"
        }
      },
      {
        event: "projectParsed",
        data: {
          projectId: "deckpilot-web",
          projectName: "DeckPilot"
        }
      },
      {
        event: "projectParseError",
        data: {
          projectId: "listingguru",
          projectName: "ListingGuru",
          filePath: ".planning/ROADMAP.md",
          message: "frontmatter could not be parsed"
        }
      },
      {
        event: "finished",
        data: {
          discoveredCount: 2,
          parsedCount: 1,
          errorCount: 1
        }
      }
    ];

    const serializedEvents = JSON.stringify(events);

    expect(serializedEvents).not.toContain("# Roadmap");
    expect(serializedEvents).not.toContain("<task");
    expect(serializedEvents).toContain("projectParseError");
    expect(serializedEvents).toContain("finished");
  });
});

describe("Phase 3 safe action wrappers", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    openPathMock.mockReset();
    openUrlMock.mockReset();
    writeTextMock.mockReset();
  });

  it("copyNextCommand writes clipboard text without backend invoke", async () => {
    const { copyNextCommand } = await import("./lib/actions");

    writeTextMock.mockResolvedValueOnce(undefined);

    await copyNextCommand("/gsd-next");

    expect(writeTextMock).toHaveBeenCalledWith("/gsd-next");
    expect(invokeMock).not.toHaveBeenCalled();
  });

  it("openProjectInFinder opens the project path directly", async () => {
    const { openProjectInFinder } = await import("./lib/actions");
    const rootPath = "/Users/smacdonald/homegit/gsd-dashboard";

    openPathMock.mockResolvedValueOnce(undefined);

    await openProjectInFinder(rootPath);

    expect(openPathMock).toHaveBeenCalledWith(rootPath);
  });

  it("openProjectInVsCode_preserves_path_separators", async () => {
    const { openProjectInVsCode } = await import("./lib/actions");
    const rootPath = "/Users/smacdonald/homegit/gsd-dashboard";

    openUrlMock.mockResolvedValueOnce(undefined);

    await openProjectInVsCode(rootPath);

    expect(openUrlMock).toHaveBeenCalledWith(
      "vscode://file//Users/smacdonald/homegit/gsd-dashboard"
    );
  });
});

describe("Phase 1 shell", () => {
  beforeEach(() => {
    resetMocks();
    mockBaseCommands(1);
  });

  it("renders boot, cache, settings, and empty dashboard states", async () => {
    renderWithQueryClient(<App />);

    expect(await screen.findByRole("heading", { name: "GSD Dashboard" })).toBeInTheDocument();
    expect(await screen.findByText("Cache ready")).toBeInTheDocument();
    expect(screen.getByText("Migrations applied")).toBeInTheDocument();
    expect(screen.getByText("Settings saved")).toBeInTheDocument();
    expect(screen.getAllByText("Default scan root").length).toBeGreaterThan(0);
    expect(screen.getAllByText("No projects scanned yet").length).toBeGreaterThan(0);
    expect(
      screen.getByText(
        "GSD Dashboard is ready to scan your configured roots. Start a scan to discover projects with `.planning/` directories."
      )
    ).toBeInTheDocument();
  });

  it("displays the first-run default scan root exactly", async () => {
    renderWithQueryClient(<App />);

    const rootInput = await screen.findByLabelText("Default scan root");

    await waitFor(() => expect(rootInput).toHaveValue("~/Documents"));
  });

  it("does not render Phase 3 dashboard controls or data surfaces", async () => {
    renderWithQueryClient(<App />);

    await screen.findByRole("heading", { name: "No projects scanned yet" });

    expect(screen.queryByText("Rebuild cache")).not.toBeInTheDocument();
    expect(screen.queryByText("Scan now")).not.toBeInTheDocument();
    expect(screen.queryByText(/session/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/chart/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/tray/i)).not.toBeInTheDocument();
    expect(screen.queryByText("Hidden projects")).not.toBeInTheDocument();
  });

  it("shows the backend broad-root error for slash and keeps the rejected value", async () => {
    mockInvalidScanRoot("/");
    renderWithQueryClient(<App />);
    const rootInput = await screen.findByLabelText("Default scan root");
    await waitFor(() => expect(rootInput).toHaveValue("~/Documents"));

    fireEvent.change(rootInput, { target: { value: "/" } });
    fireEvent.click(screen.getByRole("button", { name: "Save Settings" }));

    expect(
      await screen.findByText(
        "This scan root is too broad. Choose a specific folder inside your home directory, such as ~/Documents or a project workspace."
      )
    ).toBeInTheDocument();
    expect(screen.getByText("Rejected path: /")).toBeInTheDocument();
    expect(rootInput).toHaveValue("/");
    expect(screen.queryByText("Settings saved")).not.toBeInTheDocument();
  });

  it("shows the backend broad-root error for the bare home path", async () => {
    const homePath = "/Users/smacdonald";

    mockInvalidScanRoot(homePath);
    renderWithQueryClient(<App />);
    const rootInput = await screen.findByLabelText("Default scan root");
    await waitFor(() => expect(rootInput).toHaveValue("~/Documents"));

    fireEvent.change(rootInput, { target: { value: homePath } });
    fireEvent.click(screen.getByRole("button", { name: "Save Settings" }));

    expect(
      await screen.findByText(
        "This scan root is too broad. Choose a specific folder inside your home directory, such as ~/Documents or a project workspace."
      )
    ).toBeInTheDocument();
    expect(screen.getByText(`Rejected path: ${homePath}`)).toBeInTheDocument();
    expect(rootInput).toHaveValue(homePath);
    expect(screen.queryByText("Settings saved")).not.toBeInTheDocument();
  });
});

describe("Phase 2 scan status shell", () => {
  beforeEach(() => {
    resetMocks();
    mockBaseCommands(2, true);
  });

  it("renders the ready scan CTA and Phase 2 empty-state copy", async () => {
    renderWithQueryClient(<App />);

    expect(await screen.findByRole("button", { name: /Scan Projects/ })).toBeInTheDocument();
    expect(screen.getAllByText("Ready to scan").length).toBeGreaterThan(0);
    expect(
      screen.getByText(
        "GSD Dashboard is ready to scan your configured roots. Start a scan to discover projects with `.planning/` directories."
      )
    ).toBeInTheDocument();
  });

  it("disables the scan CTA and announces active scan progress", async () => {
    let resolveScan: ((summary: { discoveredCount: number; parsedCount: number; errorCount: number }) => void) | null =
      null;
    invokeMock.mockImplementation((command: string) => {
      const baseResponse = baseCommandResponse(command, 2);
      if (baseResponse) return baseResponse;

      if (command === "scan_projects") {
        return new Promise((resolve) => {
          resolveScan = resolve;
        });
      }

      return Promise.reject(new Error(`Unexpected command: ${command}`));
    });
    renderWithQueryClient(<App />);
    const scanButton = await screen.findByRole("button", { name: /Scan Projects/ });

    fireEvent.click(scanButton);
    act(() => {
      channelInstances[0].onmessage?.({ event: "started", data: { rootCount: 1 } });
    });

    expect(scanButton).toBeDisabled();
    expect((await screen.findAllByText("Scanning projects")).length).toBeGreaterThan(0);
    expect(screen.getByText("Walking scan roots")).toHaveAttribute("aria-live", "polite");

    act(() => {
      resolveScan?.({ discoveredCount: 0, parsedCount: 0, errorCount: 0 });
    });
  });

  it("shows completed scan counts without adding Phase 3 surfaces", async () => {
    renderWithQueryClient(<App />);
    const scanButton = await screen.findByRole("button", { name: /Scan Projects/ });

    fireEvent.click(scanButton);
    act(() => {
      channelInstances[0].onmessage?.({
        event: "finished",
        data: { discoveredCount: 3, parsedCount: 3, errorCount: 0 }
      });
    });

    expect((await screen.findAllByText("Scan complete")).length).toBeGreaterThan(0);
    expect(screen.getByText("3 projects discovered")).toBeInTheDocument();
    expect(screen.queryByText("Project Detail")).not.toBeInTheDocument();
    expect(screen.queryByText("Rebuild cache")).not.toBeInTheDocument();
    expect(screen.queryByText("Copy next command")).not.toBeInTheDocument();
  });

  it("normalizes snake_case scan summary counts from backend payloads", async () => {
    renderWithQueryClient(<App />);
    const scanButton = await screen.findByRole("button", { name: /Scan Projects/ });

    fireEvent.click(scanButton);
    act(() => {
      channelInstances[0].onmessage?.({
        event: "finished",
        data: { discovered_count: 9, parsed_count: 9, error_count: 0 }
      });
    });

    expect((await screen.findAllByText("Scan complete")).length).toBeGreaterThan(0);
    expect(screen.getByText("9 projects discovered")).toBeInTheDocument();
    expect(screen.queryByText(/NaN projects discovered/)).not.toBeInTheDocument();
  });

  it("renders compact parse-error status when scanning continues after errors", async () => {
    renderWithQueryClient(<App />);
    const scanButton = await screen.findByRole("button", { name: /Scan Projects/ });

    fireEvent.click(scanButton);
    act(() => {
      channelInstances[0].onmessage?.({
        event: "projectParseError",
        data: {
          projectId: "listingguru",
          projectName: "ListingGuru",
          filePath: ".planning/ROADMAP.md",
          message: "frontmatter could not be parsed"
        }
      });
      channelInstances[0].onmessage?.({
        event: "finished",
        data: { discoveredCount: 2, parsedCount: 1, errorCount: 1 }
      });
    });

    expect((await screen.findAllByText("Scan completed with parse errors")).length).toBeGreaterThan(
      0
    );
    expect(screen.getByText("2 projects discovered · 1 parse errors")).toBeInTheDocument();
    expect(screen.getByRole("alert")).toHaveTextContent(
      "Some planning files could not be parsed. Scanning continued; open the scan details to review the affected project and file."
    );
    expect(screen.getByText("ListingGuru · .planning/ROADMAP.md")).toBeInTheDocument();
  });

  it("shows scan command failures separately from parse errors", async () => {
    invokeMock.mockImplementation((command: string) => {
      const baseResponse = baseCommandResponse(command, 2);
      if (baseResponse) return baseResponse;

      if (command === "scan_projects") {
        return Promise.reject({ kind: "invalidScanRoot", message: "scan root is invalid" });
      }

      return Promise.reject(new Error(`Unexpected command: ${command}`));
    });
    renderWithQueryClient(<App />);
    const scanButton = await screen.findByRole("button", { name: /Scan Projects/ });

    fireEvent.click(scanButton);

    expect((await screen.findAllByText("Scan failed")).length).toBeGreaterThan(0);
    expect(screen.getByRole("alert")).toHaveTextContent(
      "Scan could not start. Check that the configured scan root exists and is allowed."
    );
    expect(screen.queryByText("0 projects discovered · 1 parse errors")).not.toBeInTheDocument();
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

function resetMocks() {
  channelInstances.length = 0;
  invokeMock.mockReset();
}

function mockBaseCommands(migrationsApplied: number, includeScan = false) {
  invokeMock.mockImplementation((command: string) => {
    const baseResponse = baseCommandResponse(command, migrationsApplied);
    if (baseResponse) return baseResponse;

    if (includeScan && command === "scan_projects") {
      return Promise.resolve({ discoveredCount: 0, parsedCount: 0, errorCount: 0 });
    }

    return Promise.reject(new Error(`Unexpected command: ${command}`));
  });
}

function baseCommandResponse(command: string, migrationsApplied: number) {
  if (command === "get_boot_status") {
    return Promise.resolve({
      appDataDir: "/tmp/gsd-dashboard",
      cachePath: "/tmp/gsd-dashboard/cache.db",
      cacheReady: true,
      walEnabled: true,
      migrationsApplied,
      settingsInitialized: true
    });
  }

  if (command === "get_settings") {
    return Promise.resolve(defaultSettings);
  }

  return null;
}

function mockInvalidScanRoot(path: string) {
  const message =
    "This scan root is too broad. Choose a specific folder inside your home directory, such as ~/Documents or a project workspace.";

  invokeMock.mockImplementation((command: string) => {
    const baseResponse = baseCommandResponse(command, 1);
    if (baseResponse) return baseResponse;
    if (command === "save_settings") {
      return Promise.reject({ kind: "invalidScanRoot", message, path, reason: message });
    }

    return Promise.reject(new Error(`Unexpected command: ${command}`));
  });
}
