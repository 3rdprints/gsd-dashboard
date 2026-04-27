import "@testing-library/jest-dom/vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import { BrowserRouter } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";

describe("GlobalCharts", () => {
  beforeEach(() => {
    vi.resetModules();
  });

  it("exposes the getGlobalChartData IPC wrapper and stable query key", async () => {
    const invoke = vi.fn().mockResolvedValue({
      sessionsBySource: [],
      tokensByProject: [],
      timeOfDay: [],
      dayOfWeek: []
    });
    vi.doMock("@tauri-apps/api/core", () => ({
      Channel: class TestChannel<T> {
        onmessage: ((event: T) => void) | null = null;
      },
      invoke
    }));

    const { getGlobalChartData } = await import("../../lib/ipc");
    const { globalChartsQueryKey } = await import("../../lib/queryClient");
    const filters = { source: "claude" as const, unmatchedOnly: true };

    await getGlobalChartData(filters);

    expect(invoke).toHaveBeenCalledWith("get_global_chart_data", { filters });
    expect(globalChartsQueryKey(filters)).toEqual(["globalCharts", filters]);
  });

  it("loads global charts with the same active filters as the table", async () => {
    const invoke = vi.fn((command: string, args?: Record<string, unknown>) => {
      if (command === "get_settings") {
        return Promise.resolve({
          scanRoots: [],
          hiddenProjectIds: [],
          autostartEnabled: false,
          trayBarMaxProjects: 6,
          trayBarSort: "name",
          globalSessionsDefaultRange: "7d"
        });
      }
      if (command === "get_portfolio") {
        return Promise.resolve({
          stats: { projectsTracked: 0, activeMilestones: 0, sessionsToday: 0, tokensToday: 0 },
          projects: [],
          hiddenProjects: [],
          unmatchedSessions: { count: 0, label: "0 unmatched", claudeCount: 0, codexCount: 0, recent: [] }
        });
      }
      if (command === "list_global_sessions") {
        return Promise.resolve({ rows: [], total: 0, page: 1, pageSize: 100 });
      }
      if (command === "get_global_chart_data") {
        return Promise.resolve({
          sessionsBySource: [],
          tokensByProject: [],
          timeOfDay: [],
          dayOfWeek: []
        });
      }
      return Promise.resolve(args ?? null);
    });
    vi.doMock("@tauri-apps/api/core", () => ({
      Channel: class TestChannel<T> {
        onmessage: ((event: T) => void) | null = null;
      },
      invoke
    }));
    const { GlobalSessionsPage } = await import("../../routes/GlobalSessionsPage");

    window.history.replaceState(null, "", "/sessions?source=codex&unmatched=true&tmin=200");
    render(
      <QueryClientProvider client={new QueryClient()}>
        <BrowserRouter>
          <GlobalSessionsPage />
        </BrowserRouter>
      </QueryClientProvider>
    );

    expect(await screen.findByText("Sessions by source")).toBeInTheDocument();
    await waitFor(() => {
      const tableCall = invoke.mock.calls.find(([command]) => command === "list_global_sessions");
      const chartCall = invoke.mock.calls.find(([command]) => command === "get_global_chart_data");

      expect(tableCall?.[1]).toMatchObject({ filters: { source: "codex", tokensMin: 200, unmatchedOnly: true } });
      expect(chartCall?.[1]).toMatchObject({ filters: tableCall?.[1].filters });
    });
  });
});
