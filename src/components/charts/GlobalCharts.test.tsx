import "@testing-library/jest-dom/vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import { BrowserRouter } from "react-router-dom";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("recharts", () => ({
  BarChart: ({ children, data }: { children: ReactNode; data: unknown[] }) => (
    <div data-testid="bar-chart" data-points={data.length}>
      {children}
    </div>
  ),
  Bar: ({
    dataKey,
    fill,
    fillOpacity,
    stackId,
    name
  }: {
    dataKey: string;
    fill: string;
    fillOpacity?: number;
    stackId?: string;
    name?: string;
  }) => (
    <div
      data-testid={`bar-${dataKey}`}
      data-fill={fill}
      data-fill-opacity={fillOpacity}
      data-name={name}
      data-stack-id={stackId}
    />
  ),
  CartesianGrid: () => <div data-testid="cartesian-grid" />,
  Legend: () => <div data-testid="legend" />,
  ResponsiveContainer: ({ children, height }: { children: ReactNode; height: number }) => (
    <div data-testid="responsive-container" data-height={height}>
      {children}
    </div>
  ),
  Tooltip: () => <div data-testid="tooltip" />,
  XAxis: ({ dataKey, ticks }: { dataKey: string; ticks?: number[] }) => (
    <div data-testid={`x-axis-${dataKey}`} data-ticks={ticks?.join(",")} />
  ),
  YAxis: () => <div data-testid="y-axis" />
}));

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
      const tableArgs = tableCall?.[1] as { filters: Record<string, unknown> } | undefined;
      const chartArgs = chartCall?.[1] as { filters: Record<string, unknown> } | undefined;

      expect(tableArgs).toMatchObject({ filters: { source: "codex", tokensMin: 200, unmatchedOnly: true } });
      expect(chartArgs).toMatchObject({ filters: tableArgs?.filters });
    });
  });

  it("renders source chart as Claude and Codex stacked bars", async () => {
    const { StackedSourcesChart } = await import("./StackedSourcesChart");

    render(<StackedSourcesChart data={[{ date: "2026-04-27", claude: 2, codex: 3 }]} />);

    expect(screen.getByTestId("responsive-container")).toHaveAttribute("data-height", "200");
    expect(screen.getByTestId("bar-claude")).toHaveAttribute("data-stack-id", "src");
    expect(screen.getByTestId("bar-claude")).toHaveAttribute("data-fill", "#2563EB");
    expect(screen.getByTestId("bar-codex")).toHaveAttribute("data-stack-id", "src");
    expect(screen.getByTestId("bar-codex")).toHaveAttribute("data-fill", "#7C3AED");
  });

  it("renders project chart as up to five project stacks plus other with escaped legend text", async () => {
    const { StackedProjectsChart } = await import("./StackedProjectsChart");
    const unsafeName = "<img src=x onerror=alert(1)>";

    render(
      <StackedProjectsChart
        data={[
          { date: "2026-04-27", projectId: "one", projectName: unsafeName, tokens: 100 },
          { date: "2026-04-27", projectId: "two", projectName: "Two", tokens: 90 },
          { date: "2026-04-27", projectId: "three", projectName: "Three", tokens: 80 },
          { date: "2026-04-27", projectId: "four", projectName: "Four", tokens: 70 },
          { date: "2026-04-27", projectId: "five", projectName: "Five", tokens: 60 },
          { date: "2026-04-27", projectId: null, projectName: "Other", tokens: 50 }
        ]}
      />
    );

    expect(screen.getByTestId("bar-project0")).toHaveAttribute("data-stack-id", "projects");
    expect(screen.getByTestId("bar-project4")).toHaveAttribute("data-fill", "#DC2626");
    expect(screen.getByTestId("bar-other")).toHaveAttribute("data-stack-id", "projects");
    expect(screen.getByTestId("bar-other")).toHaveAttribute("data-fill", "#9CA3AF");
    expect(screen.getByText(unsafeName)).toBeInTheDocument();
    expect(document.querySelector("img")).toBeNull();
  });

  it("normalizes time-of-day chart to 24 buckets with 4-hour ticks", async () => {
    const { TimeOfDayHistogram } = await import("./TimeOfDayHistogram");

    render(<TimeOfDayHistogram data={[{ hour: 3, count: 4 }]} />);

    expect(screen.getByTestId("bar-chart")).toHaveAttribute("data-points", "24");
    expect(screen.getByTestId("x-axis-hour")).toHaveAttribute("data-ticks", "0,4,8,12,16,20");
    expect(screen.getByText("3:00")).toBeInTheDocument();
  });

  it("normalizes day-of-week chart to seven Mon-first buckets", async () => {
    const { DayOfWeekChart } = await import("./DayOfWeekChart");

    render(<DayOfWeekChart data={[{ day: 0, count: 2 }, { day: 1, count: 4 }]} />);

    expect(screen.getByTestId("bar-chart")).toHaveAttribute("data-points", "7");
    expect(screen.getByText("Mon")).toBeInTheDocument();
    expect(screen.getByText("Sun")).toBeInTheDocument();
  });

  it("renders the shared empty chart state", async () => {
    const { ChartCard } = await import("./ChartCard");

    render(
      <ChartCard title="Sessions by source" subtitle="Daily Claude and Codex session volume" empty>
        <div />
      </ChartCard>
    );

    expect(screen.getByText("No data for this period.")).toBeInTheDocument();
  });
});
