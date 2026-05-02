import "@testing-library/jest-dom/vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { ScanRootsEditor } from "./ScanRootsEditor";
import type { AppSettings, PortfolioDto, SettingsInput } from "../lib/types";

const { getSettingsMock, saveSettingsMock, getPortfolioMock } = vi.hoisted(() => ({
  getSettingsMock: vi.fn<() => Promise<AppSettings>>(),
  saveSettingsMock: vi.fn<(input: SettingsInput) => Promise<AppSettings>>(),
  getPortfolioMock: vi.fn<() => Promise<PortfolioDto>>()
}));

vi.mock("../lib/ipc", async (importOriginal) => {
  const actual = await importOriginal<typeof import("../lib/ipc")>();
  return {
    ...actual,
    getSettings: getSettingsMock,
    saveSettings: saveSettingsMock,
    getPortfolio: getPortfolioMock
  };
});

describe("ScanRootsEditor tray display settings", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    getSettingsMock.mockResolvedValue(baseSettings());
    saveSettingsMock.mockImplementation((input) => Promise.resolve(input));
    getPortfolioMock.mockResolvedValue(basePortfolio());
  });

  it("renders the Tray Display section controls", async () => {
    renderScanRootsEditor();

    expect(await screen.findByRole("heading", { name: "Tray Display" })).toBeInTheDocument();
    expect(screen.getByText("Menu bar")).toBeInTheDocument();
    expect(screen.getByLabelText("Max tray bars")).toHaveValue(8);
    expect(screen.getByLabelText("Recent activity")).toBeChecked();
    expect(screen.getByLabelText("Progress")).toBeInTheDocument();
    expect(screen.getByLabelText("Name")).toBeInTheDocument();
  });

  it("renders_launch_on_login_default_off", async () => {
    renderScanRootsEditor();

    const launchOnLoginToggle = await screen.findByLabelText("Launch on login");

    expect(launchOnLoginToggle).not.toBeChecked();
    expect(screen.getAllByText("Launch on login")).toHaveLength(1);
    expect(
      screen.getByText(
        "Off by default. Enable this to keep the tray dashboard available after sign-in."
      )
    ).toBeInTheDocument();
  });

  it("saves_autostart_enabled_when_toggle_is_checked", async () => {
    renderScanRootsEditor();

    const launchOnLoginToggle = await screen.findByLabelText("Launch on login");
    const saveButton = screen.getByRole("button", { name: "Save Settings" });
    await waitFor(() => expect(saveButton).toBeEnabled());

    fireEvent.click(launchOnLoginToggle);
    fireEvent.click(saveButton);

    await waitFor(() => {
      expect(saveSettingsMock).toHaveBeenCalledWith(
        expect.objectContaining({
          autostartEnabled: true
        })
      );
    });
  });

  it("saves_autostart_disabled_when_toggle_is_not_checked", async () => {
    renderScanRootsEditor();

    await screen.findByLabelText("Launch on login");
    const saveButton = screen.getByRole("button", { name: "Save Settings" });
    await waitFor(() => expect(saveButton).toBeEnabled());

    fireEvent.click(saveButton);

    await waitFor(() => {
      expect(saveSettingsMock).toHaveBeenCalledWith(
        expect.objectContaining({
          autostartEnabled: false
        })
      );
    });
  });

  it("shows_autostart_error_copy_when_save_fails", async () => {
    saveSettingsMock.mockRejectedValueOnce({
      kind: "settings",
      message: "failed to update autostart"
    });
    renderScanRootsEditor();

    const launchOnLoginToggle = await screen.findByLabelText("Launch on login");
    const saveButton = screen.getByRole("button", { name: "Save Settings" });
    await waitFor(() => expect(saveButton).toBeEnabled());

    fireEvent.click(launchOnLoginToggle);
    fireEvent.click(saveButton);

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Launch on login could not be updated. The setting was not changed; try again from the desktop app."
    );
  });

  it("saves max tray bars through the form-level Save Settings action", async () => {
    renderScanRootsEditor();

    const maxTrayBars = await screen.findByLabelText("Max tray bars");
    fireEvent.change(maxTrayBars, { target: { value: "16" } });
    fireEvent.click(screen.getByLabelText("Progress"));
    fireEvent.click(screen.getByRole("button", { name: "Save Settings" }));

    await waitFor(() => {
      expect(saveSettingsMock).toHaveBeenCalledWith(
        expect.objectContaining({
          trayBarMaxProjects: 16,
          trayBarSort: "progress"
        })
      );
    });
  });

  it("sends unchecked tray projects as trayHiddenProjectIds", async () => {
    renderScanRootsEditor();

    expect(await screen.findByLabelText("Dashboard")).toBeChecked();
    expect(screen.queryByLabelText("Hidden Portfolio Project")).not.toBeInTheDocument();

    fireEvent.click(screen.getByLabelText("Dashboard"));
    fireEvent.click(screen.getByRole("button", { name: "Save Settings" }));

    await waitFor(() => {
      expect(saveSettingsMock).toHaveBeenCalledWith(
        expect.objectContaining({
          trayHiddenProjectIds: ["project-visible"]
        })
      );
    });
  });
});

function renderScanRootsEditor() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false }
    }
  });

  render(
    <QueryClientProvider client={queryClient}>
      <ScanRootsEditor />
    </QueryClientProvider>
  );
}

function baseSettings(): AppSettings {
  return {
    scanRoots: ["~/Documents"],
    hiddenProjectIds: ["project-hidden"],
    trayHiddenProjectIds: [],
    autostartEnabled: false,
    trayBarMaxProjects: 8,
    trayBarSort: "recent_activity",
    globalSessionsDefaultRange: "7d"
  };
}

function basePortfolio(): PortfolioDto {
  return {
    stats: { projectsTracked: 1, activeMilestones: 1, sessionsToday: 0, tokensToday: 0 },
    projects: [
      {
        id: "project-visible",
        name: "Dashboard",
        rootPath: "/tmp/dashboard",
        planningPath: "/tmp/dashboard/.planning",
        currentMilestoneName: "v1",
        currentPhaseNumber: "06",
        currentPhaseName: "Tray",
        milestoneProgressPct: 50,
        nextCommand: "/gsd-next",
        parseError: null,
        lastActivityAt: null,
        lastScannedAt: 0,
        sessionSparkline7d: [],
        sessionsLast7d: 0
      }
    ],
    hiddenProjects: [
      {
        id: "project-hidden",
        name: "Hidden Portfolio Project",
        rootPath: "/tmp/hidden"
      }
    ],
    unmatchedSessions: {
      count: 0,
      label: "0 unmatched",
      claudeCount: 0,
      codexCount: 0,
      recent: []
    }
  };
}
