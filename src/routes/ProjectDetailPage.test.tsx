import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import "@testing-library/jest-dom/vitest";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import type React from "react";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { describe, expect, it, vi } from "vitest";

import { ProjectDetailPage } from "./ProjectDetailPage";
import type { ProjectDetail } from "../lib/types";

const { getProjectMock, invokeMock } = vi.hoisted(() => ({
  getProjectMock: vi.fn(),
  invokeMock: vi.fn()
}));

vi.mock("../lib/ipc", async (importOriginal) => {
  const actual = await importOriginal<typeof import("../lib/ipc")>();
  return {
    ...actual,
    getProject: getProjectMock
  };
});

vi.mock("@tauri-apps/api/core", () => ({
  Channel: class TestChannel<T> {
    onmessage: ((event: T) => void) | null = null;
  },
  invoke: invokeMock
}));

vi.mock("../lib/actions", () => ({
  copyNextCommand: vi.fn(() => Promise.resolve()),
  openProjectInFinder: vi.fn(() => Promise.resolve()),
  openProjectInVsCode: vi.fn(() => Promise.resolve())
}));

const projectDetail: ProjectDetail = {
  id: "gsd-dashboard",
  name: "GSD Dashboard",
  rootPath: "/Users/smacdonald/homegit/gsd-dashboard",
  planningPath: "/Users/smacdonald/homegit/gsd-dashboard/.planning",
  currentMilestoneName: "v1.0 MVP",
  currentPhaseNumber: "05",
  currentPhaseName: "Project Detail",
  milestoneProgressPct: 42,
  nextCommand: "/gsd-next",
  parseError: null,
  lastActivityAt: null,
  lastScannedAt: 1_777_301_924,
  sessionSparkline7d: [],
  sessionsLast7d: 0
};

describe("ProjectDetailPage tab shell", () => {
  it("keeps the shared header above accessible in-page tabs", async () => {
    getProjectMock.mockResolvedValue(projectDetail);

    renderProjectDetail();

    expect(await screen.findByRole("heading", { name: "GSD Dashboard" })).toBeInTheDocument();
    const tablist = screen.getByRole("tablist", { name: "Project detail sections" });
    expect(tablist).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "Overview" })).toHaveAttribute("aria-selected", "true");
    expect(screen.getByRole("tab", { name: "Sessions" })).toHaveAttribute("aria-controls", "project-tab-sessions");
    expect(screen.getByRole("tabpanel", { name: "Overview" })).toHaveAttribute("id", "project-tab-overview");
  });

  it("supports arrow, Home, and End keyboard tab navigation", async () => {
    getProjectMock.mockResolvedValue(projectDetail);

    renderProjectDetail();

    const overviewTab = await screen.findByRole("tab", { name: "Overview" });
    overviewTab.focus();
    fireEvent.keyDown(overviewTab, { key: "ArrowRight" });
    expect(screen.getByRole("tab", { name: "Sessions" })).toHaveAttribute("aria-selected", "true");
    fireEvent.keyDown(screen.getByRole("tab", { name: "Sessions" }), { key: "End" });
    expect(screen.getByRole("tab", { name: "Charts" })).toHaveAttribute("aria-selected", "true");
    fireEvent.keyDown(screen.getByRole("tab", { name: "Charts" }), { key: "Home" });
    expect(screen.getByRole("tab", { name: "Overview" })).toHaveAttribute("aria-selected", "true");
  });
});

describe("Project Detail IPC contracts", () => {
  it("exposes typed milestone and phase panel wrappers with stable query keys", async () => {
    const ipc = await import("../lib/ipc");
    const queryClient = await import("../lib/queryClient");
    invokeMock.mockResolvedValue({});

    expect(typeof (ipc as Record<string, unknown>).getProjectMilestones).toBe("function");
    expect(typeof (ipc as Record<string, unknown>).getProjectPhasePanel).toBe("function");
    expect(typeof (queryClient as Record<string, unknown>).projectMilestonesQueryKey).toBe("function");
    expect(typeof (queryClient as Record<string, unknown>).projectPhasePanelQueryKey).toBe("function");

    await (ipc as { getProjectMilestones: (id: string) => Promise<unknown> }).getProjectMilestones("gsd-dashboard");
    await (ipc as { getProjectPhasePanel: (id: string) => Promise<unknown> }).getProjectPhasePanel("gsd-dashboard");

    expect(invokeMock).toHaveBeenCalledWith("get_project_milestones", { projectId: "gsd-dashboard" });
    expect(invokeMock).toHaveBeenCalledWith("get_project_phase_panel", { projectId: "gsd-dashboard" });
  });

  it("exposes typed project sessions and chart wrappers with stable query keys", async () => {
    const ipc = await import("../lib/ipc");
    const queryClient = await import("../lib/queryClient");
    invokeMock.mockResolvedValue({});

    expect(typeof (ipc as Record<string, unknown>).listProjectSessions).toBe("function");
    expect(typeof (ipc as Record<string, unknown>).getProjectChartData).toBe("function");
    expect(typeof (queryClient as Record<string, unknown>).projectSessionsQueryKey).toBe("function");
    expect(typeof (queryClient as Record<string, unknown>).projectChartsQueryKey).toBe("function");

    await (ipc as {
      listProjectSessions: (
        id: string,
        sort: string,
        direction: string,
        page: number,
        pageSize: number
      ) => Promise<unknown>;
    }).listProjectSessions("gsd-dashboard", "startedAt", "desc", 2, 50);
    await (ipc as { getProjectChartData: (id: string, range: string) => Promise<unknown> }).getProjectChartData(
      "gsd-dashboard",
      "30d"
    );

    expect(invokeMock).toHaveBeenCalledWith("list_project_sessions", {
      projectId: "gsd-dashboard",
      sort: "startedAt",
      direction: "desc",
      page: 2,
      pageSize: 50
    });
    expect(invokeMock).toHaveBeenCalledWith("get_project_chart_data", {
      projectId: "gsd-dashboard",
      range: "30d"
    });
    expect((queryClient as { projectSessionsQueryKey: (...args: unknown[]) => readonly unknown[] }).projectSessionsQueryKey(
      "gsd-dashboard",
      "startedAt",
      "desc",
      2,
      50
    )).toEqual(["project", "gsd-dashboard", "sessions", "startedAt", "desc", 2, 50]);
    expect((queryClient as { projectChartsQueryKey: (...args: unknown[]) => readonly unknown[] }).projectChartsQueryKey(
      "gsd-dashboard",
      "30d"
    )).toEqual(["project", "gsd-dashboard", "charts", "30d"]);
  });
});

function renderProjectDetail() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false }
    }
  });

  render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter initialEntries={["/project/gsd-dashboard"]}>
        <Routes>
          <Route path="/project/:id" element={<ProjectDetailPage />} />
        </Routes>
      </MemoryRouter>
    </QueryClientProvider>
  );
}
