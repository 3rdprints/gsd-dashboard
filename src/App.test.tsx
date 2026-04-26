import { readFileSync } from "node:fs";
import { resolve } from "node:path";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import "@testing-library/jest-dom/vitest";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import type React from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { App } from "./App";
import type { PortfolioDto, ProjectDetail, ScanEvent, SettingsInput } from "./lib/types";

const { channelInstances, invokeMock, openUrlMock, revealItemInDirMock, writeTextMock } = vi.hoisted(() => ({
  channelInstances: [] as Array<{ onmessage: ((event: unknown) => void) | null }>,
  invokeMock: vi.fn(),
  openUrlMock: vi.fn(),
  revealItemInDirMock: vi.fn(),
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
  openUrl: openUrlMock,
  revealItemInDir: revealItemInDirMock
}));

const defaultSettings: SettingsInput = {
  scanRoots: ["~/Documents"],
  hiddenProjectIds: ["listingguru"],
  autostartEnabled: false,
  trayBarMaxProjects: 8,
  trayBarSort: "recent_activity"
};

const portfolio: PortfolioDto = {
  stats: {
    projectsTracked: 2,
    activeMilestones: 2,
    sessionsToday: 0,
    tokensToday: 0
  },
  projects: [
    {
      id: "gsd-dashboard",
      name: "GSD Dashboard",
      rootPath: "/Users/smacdonald/homegit/gsd-dashboard",
      planningPath: "/Users/smacdonald/homegit/gsd-dashboard/.planning",
      currentMilestoneName: "v1.0 MVP",
      currentPhaseNumber: "03",
      currentPhaseName: "Portfolio Vertical Slice",
      milestoneProgressPct: 42,
      nextCommand: "/gsd-execute-phase 3",
      parseError: null,
      lastActivityAt: 1_777_132_245,
      lastScannedAt: 1_777_132_245
    },
    {
      id: "deckpilot",
      name: "DeckPilot",
      rootPath: "/Users/smacdonald/homegit/deckpilot",
      planningPath: "/Users/smacdonald/homegit/deckpilot/.planning",
      currentMilestoneName: "Launch",
      currentPhaseNumber: "02",
      currentPhaseName: "Parser",
      milestoneProgressPct: 75,
      nextCommand: "/gsd-next",
      parseError: "ROADMAP frontmatter could not be parsed",
      lastActivityAt: null,
      lastScannedAt: 1_777_132_200
    }
  ],
  hiddenProjects: [
    {
      id: "listingguru",
      name: "ListingGuru",
      rootPath: "/Users/smacdonald/homegit/listingguru"
    }
  ],
  unmatchedSessions: {
    count: 0,
    label: "Available after session indexing"
  }
};

const projectDetail: ProjectDetail = portfolio.projects[0];

describe("IPC plumbing", () => {
  beforeEach(() => {
    channelInstances.length = 0;
    invokeMock.mockReset();
  });

  it("calls the exact command names for boot, settings, portfolio, detail, scan, and rebuild", async () => {
    const { getBootStatus, getPortfolio, getProject, getSettings, rebuildCache, saveSettings, scanProjects } =
      await import("./lib/ipc");

    invokeMock.mockResolvedValue({});

    await getBootStatus();
    await getSettings();
    await saveSettings(defaultSettings);
    await getPortfolio();
    await getProject("gsd-dashboard");
    await scanProjects(vi.fn());
    await rebuildCache(vi.fn());

    expect(invokeMock).toHaveBeenNthCalledWith(1, "get_boot_status");
    expect(invokeMock).toHaveBeenNthCalledWith(2, "get_settings");
    expect(invokeMock).toHaveBeenNthCalledWith(3, "save_settings", { input: defaultSettings });
    expect(invokeMock).toHaveBeenNthCalledWith(4, "get_portfolio");
    expect(invokeMock).toHaveBeenNthCalledWith(5, "get_project", { projectId: "gsd-dashboard" });
    expect(invokeMock).toHaveBeenNthCalledWith(6, "scan_projects", {
      onEvent: channelInstances[0]
    });
    expect(invokeMock).toHaveBeenNthCalledWith(7, "rebuild_cache", {
      onEvent: channelInstances[1]
    });
  });

  it("provides the query client at the app root", () => {
    const mainSource = readFileSync(resolve("src/main.tsx"), "utf8");

    expect(mainSource).toContain("QueryClientProvider");
    expect(mainSource).toContain("queryClient");
  });

  it("invalidates settings, portfolio, and project queries only after a successful settings save", async () => {
    const { createSaveSettingsMutationOptions, portfolioQueryKey, settingsQueryKey } = await import(
      "./lib/queryClient"
    );
    const queryClient = new QueryClient();
    const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");

    invokeMock.mockRejectedValueOnce({ kind: "store", message: "save failed" });
    await expect(createSaveSettingsMutationOptions(queryClient).mutationFn(defaultSettings)).rejects.toEqual({
      kind: "store",
      message: "save failed"
    });
    expect(invalidateSpy).not.toHaveBeenCalled();

    invokeMock.mockResolvedValueOnce(defaultSettings);
    await createSaveSettingsMutationOptions(queryClient).mutationFn(defaultSettings);
    await createSaveSettingsMutationOptions(queryClient).onSuccess?.(defaultSettings);

    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: settingsQueryKey });
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: portfolioQueryKey });
    expect(invalidateSpy).toHaveBeenCalledWith({ predicate: expect.any(Function) });
  });
});

describe("portfolio vertical slice", () => {
  beforeEach(() => {
    resetMocks();
    mockCommands();
    window.history.pushState({}, "", "/");
  });

  it("renders project cards with portfolio stats", async () => {
    renderWithQueryClient(<App />);

    expect(await screen.findByRole("heading", { name: "Portfolio" })).toBeInTheDocument();
    expect(document.querySelectorAll(".project-card-skeleton")).toHaveLength(2);
    expect(screen.getByText("Projects tracked")).toBeInTheDocument();
    expect(screen.getByText("Active milestones")).toBeInTheDocument();
    expect(screen.getByText("Sessions today")).toBeInTheDocument();
    expect(screen.getByText("Tokens today")).toBeInTheDocument();
    expect(await screen.findByRole("link", { name: /GSD Dashboard/ })).toBeInTheDocument();
    expect(screen.getByText("v1.0 MVP")).toBeInTheDocument();
    expect(screen.getByText("Phase 03: Portfolio Vertical Slice")).toBeInTheDocument();
    expect(screen.getByText("42%")).toBeInTheDocument();
    expect(screen.getByText("Parse error")).toBeInTheDocument();
  });

  it("renders No projects found empty state when portfolio has no cards", async () => {
    mockCommands({
      ...portfolio,
      stats: { projectsTracked: 0, activeMilestones: 0, sessionsToday: 0, tokensToday: 0 },
      projects: [],
      hiddenProjects: []
    });
    renderWithQueryClient(<App />);

    expect(await screen.findByRole("heading", { name: "No projects found" })).toBeInTheDocument();
    expect(
      screen.getByText("Add a scan root or rebuild the cache to discover projects with `.planning/` directories.")
    ).toBeInTheDocument();
  });

  it("copies from a card without navigating and shows Copied feedback", async () => {
    let resolveCopy: (() => void) | null = null;
    writeTextMock.mockReturnValueOnce(new Promise<void>((resolve) => { resolveCopy = resolve; }));
    renderWithQueryClient(<App />);

    await screen.findByRole("link", { name: /GSD Dashboard/ });
    const copyButtons = await screen.findAllByRole("button", { name: "Copy next command" });
    fireEvent.click(copyButtons[0]);
    expect(screen.queryByText("Copied")).not.toBeInTheDocument();
    await act(async () => {
      resolveCopy?.();
      await Promise.resolve();
      await Promise.resolve();
    });

    expect(writeTextMock).toHaveBeenCalledWith("/gsd-execute-phase 3");
    expect(await screen.findByText("Copied")).toBeInTheDocument();
    expect(window.location.pathname).toBe("/");
  });

  it("hides a visible project and removes it after portfolio refetch", async () => {
    let hiddenSaved = false;
    invokeMock.mockImplementation((command: string, args?: Record<string, unknown>) => {
      if (command === "get_boot_status") {
        return Promise.resolve({ appDataDir: "/tmp", cachePath: "/tmp/cache.db", cacheReady: true, walEnabled: true, migrationsApplied: 3, settingsInitialized: true });
      }
      if (command === "get_settings") return Promise.resolve(defaultSettings);
      if (command === "get_portfolio") {
        return Promise.resolve(
          hiddenSaved
            ? { ...portfolio, projects: portfolio.projects.filter((project) => project.id !== "gsd-dashboard"), hiddenProjects: [...portfolio.hiddenProjects, { id: "gsd-dashboard", name: "GSD Dashboard", rootPath: portfolio.projects[0].rootPath }] }
            : portfolio
        );
      }
      if (command === "save_settings") {
        expect(args).toEqual({
          input: { ...defaultSettings, hiddenProjectIds: ["listingguru", "gsd-dashboard"] }
        });
        hiddenSaved = true;
        return Promise.resolve((args as { input: SettingsInput }).input);
      }
      if (command === "scan_projects") return Promise.resolve({ discoveredCount: 2, parsedCount: 2, errorCount: 0 });
      return Promise.reject(new Error(`Unexpected command: ${command}`));
    });
    renderWithQueryClient(<App />);

    expect(await screen.findByRole("link", { name: /GSD Dashboard/ })).toBeInTheDocument();
    fireEvent.click((await screen.findAllByRole("button", { name: "Hide Project" }))[0]);

    await waitFor(() => expect(screen.queryByRole("link", { name: /GSD Dashboard/ })).not.toBeInTheDocument());
  });

  it("does not show Copied when clipboard failed", async () => {
    writeTextMock.mockRejectedValueOnce(new Error("clipboard failed"));
    renderWithQueryClient(<App />);

    await screen.findByRole("link", { name: /GSD Dashboard/ });
    fireEvent.click((await screen.findAllByRole("button", { name: "Copy next command" }))[0]);

    await waitFor(() => expect(writeTextMock).toHaveBeenCalled());
    expect(screen.queryByText("Copied")).not.toBeInTheDocument();
    expect(window.location.pathname).toBe("/");
  });

  it("links from a card to project detail and calls get_project on the detail route", async () => {
    renderWithQueryClient(<App />);

    const projectLink = await screen.findByRole("link", { name: /GSD Dashboard/ });
    expect(projectLink).toHaveAttribute("href", "/project/gsd-dashboard");

    cleanup();
    window.history.pushState({}, "", "/project/gsd-dashboard");
    renderWithQueryClient(<App />);

    expect(await screen.findByRole("heading", { name: "GSD Dashboard" })).toBeInTheDocument();
    await waitFor(() =>
      expect(invokeMock).toHaveBeenCalledWith("get_project", { projectId: "gsd-dashboard" })
    );
    expect(screen.getByText("/Users/smacdonald/homegit/gsd-dashboard")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Open in Finder" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Open in VS Code" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Copy next command" })).toBeInTheDocument();
  });

  it("detail_opener_failure_renders_inline_error", async () => {
    window.history.pushState({}, "", "/project/gsd-dashboard");
    revealItemInDirMock.mockRejectedValueOnce(new Error("open failed"));
    renderWithQueryClient(<App />);

    fireEvent.click(await screen.findByRole("button", { name: "Open in Finder" }));

    expect(
      await screen.findByText("Action failed. Check the configured project path and try again.")
    ).toBeInTheDocument();
  });

  it("renders right rail placeholders", async () => {
    renderWithQueryClient(<App />);

    expect(await screen.findByText("ListingGuru")).toBeInTheDocument();
    expect(screen.getByText("Hidden projects")).toBeInTheDocument();
    expect(screen.getAllByText("ListingGuru").length).toBeGreaterThan(0);
    expect(screen.getByText("Unmatched sessions")).toBeInTheDocument();
    expect(screen.getByText("Available after session indexing")).toBeInTheDocument();
  });
});

describe("settings vertical slice", () => {
  beforeEach(() => {
    resetMocks();
    mockCommands();
    window.history.pushState({}, "", "/settings");
  });

  it("renders settings sections and disabled indexing toggles", async () => {
    renderWithQueryClient(<App />);

    expect(await screen.findByRole("heading", { name: "Settings" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Scan roots" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Hidden projects" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Rebuild Cache" })).toBeDisabled();
    expect(screen.getByLabelText("Index tool usage")).toBeDisabled();
    expect(screen.getByLabelText("Index message content")).toBeDisabled();
  });

  it("adds and removes scan roots before saving settings", async () => {
    renderWithQueryClient(<App />);

    const rootInputs = await screen.findAllByRole("textbox");
    fireEvent.change(rootInputs[0], { target: { value: "~/homegit" } });
    fireEvent.click(screen.getByRole("button", { name: "Add Root" }));

    const updatedInputs = await screen.findAllByRole("textbox");
    fireEvent.change(updatedInputs[1], { target: { value: "~/Documents/clients" } });
    fireEvent.click(screen.getAllByRole("button", { name: "Remove Root" })[0]);
    fireEvent.click(screen.getByRole("button", { name: "Save Settings" }));

    await waitFor(() =>
      expect(invokeMock).toHaveBeenCalledWith("save_settings", {
        input: {
          ...defaultSettings,
          scanRoots: ["~/Documents/clients"]
        }
      })
    );
  });

  it("unhides hidden projects through settings save", async () => {
    renderWithQueryClient(<App />);

    fireEvent.click(await screen.findByRole("button", { name: "Unhide Project" }));

    await waitFor(() =>
      expect(invokeMock).toHaveBeenCalledWith("save_settings", {
        input: {
          ...defaultSettings,
          hiddenProjectIds: []
        }
      })
    );
  });

  it("settings rebuild confirmation states source planning files will not be changed", async () => {
    renderWithQueryClient(<App />);

    expect(
      await screen.findByText(
        "Rebuild cache: This clears the derived project cache and runs a full rescan. Source `.planning/` files will not be changed."
      )
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Rebuild Cache" })).toBeDisabled();

    fireEvent.click(screen.getByLabelText("Confirm rebuild cache"));
    fireEvent.click(screen.getByRole("button", { name: "Rebuild Cache" }));

    await waitFor(() => expect(invokeMock).toHaveBeenCalledWith("rebuild_cache", expect.any(Object)));
  });

  it("disables duplicate rebuild while rebuild_cache is in progress", async () => {
    mockCommands(portfolio, projectDetail, true);
    renderWithQueryClient(<App />);

    fireEvent.click(await screen.findByLabelText("Confirm rebuild cache"));
    fireEvent.click(screen.getByRole("button", { name: "Rebuild Cache" }));

    expect(screen.getByRole("button", { name: "Rebuild Cache" })).toBeDisabled();
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
  openUrlMock.mockReset();
  revealItemInDirMock.mockReset();
  writeTextMock.mockReset();
}

function mockCommands(
  portfolioResponse: PortfolioDto = portfolio,
  projectResponse: ProjectDetail = projectDetail,
  holdRebuild = false
) {
  invokeMock.mockImplementation((command: string, args?: Record<string, unknown>) => {
    if (command === "get_boot_status") {
      return Promise.resolve({
        appDataDir: "/tmp/gsd-dashboard",
        cachePath: "/tmp/gsd-dashboard/cache.db",
        cacheReady: true,
        walEnabled: true,
        migrationsApplied: 3,
        settingsInitialized: true
      });
    }

    if (command === "get_settings") {
      return Promise.resolve(defaultSettings);
    }

    if (command === "save_settings") {
      return Promise.resolve((args as { input: SettingsInput }).input);
    }

    if (command === "get_portfolio") {
      return Promise.resolve(portfolioResponse);
    }

    if (command === "get_project") {
      return Promise.resolve(projectResponse);
    }

    if (command === "scan_projects" || command === "rebuild_cache") {
      if (command === "rebuild_cache" && holdRebuild) {
        return new Promise(() => {});
      }
      const event = (args as { onEvent: { onmessage: ((event: ScanEvent) => void) | null } })
        .onEvent;
      act(() => {
        event.onmessage?.({
          event: "finished",
          data: { discoveredCount: 2, parsedCount: 2, errorCount: 0 }
        });
      });
      return Promise.resolve({ discoveredCount: 2, parsedCount: 2, errorCount: 0 });
    }

    return Promise.reject(new Error(`Unexpected command: ${command}`));
  });
}
