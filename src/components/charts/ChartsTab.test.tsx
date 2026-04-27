import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import "@testing-library/jest-dom/vitest";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

const { getProjectChartDataMock } = vi.hoisted(() => ({
  getProjectChartDataMock: vi.fn()
}));

vi.mock("../../lib/ipc", async (importOriginal) => {
  const actual = await importOriginal<typeof import("../../lib/ipc")>();
  return {
    ...actual,
    getProjectChartData: getProjectChartDataMock
  };
});

describe("ChartsTab", () => {
  it("renders loading and empty states inside stable chart cards", async () => {
    const { ChartCard } = await import("./ChartCard");

    const { rerender } = render(
      <ChartCard title="Sessions per day" subtitle="Last 30 days" loading>
        <div />
      </ChartCard>
    );

    expect(screen.getByLabelText("Loading Sessions per day")).toHaveClass("chart-card-body");

    rerender(
      <ChartCard title="Sessions per day" subtitle="Last 30 days" empty>
        <div />
      </ChartCard>
    );
    expect(screen.getByText("No data for this period.")).toBeInTheDocument();
  });

  it("uses one range selector for all project charts and refetches by range", async () => {
    getProjectChartDataMock.mockResolvedValue({
      sessionsPerDay: [{ date: "2026-04-27", count: 2 }],
      tokensPerDay: [{ date: "2026-04-27", tokens: 4200 }],
      averageDurationPerDay: [{ date: "2026-04-27", averageDurationMs: 180000 }],
      milestoneVelocity: [{ week: "2026-W17", completedPlans: 3 }]
    });
    const { ProjectChartsTab } = await import("../ProjectDetail/ProjectChartsTab");

    renderWithQueryClient(<ProjectChartsTab projectId="gsd-dashboard" />);

    expect(await screen.findByRole("button", { name: "30d" })).toHaveAttribute("aria-pressed", "true");
    expect(screen.getByText("Sessions per day")).toBeInTheDocument();
    expect(screen.getByText("Tokens per day")).toBeInTheDocument();
    expect(screen.getByText("Average duration")).toBeInTheDocument();
    expect(screen.getByText("Milestone velocity")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "90d" }));
    expect(screen.getByRole("button", { name: "90d" })).toHaveAttribute("aria-pressed", "true");
    await waitFor(() => {
      expect(getProjectChartDataMock).toHaveBeenCalledWith("gsd-dashboard", "90d");
    });
  });
});

function renderWithQueryClient(children: React.ReactElement) {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false }
    }
  });

  render(<QueryClientProvider client={queryClient}>{children}</QueryClientProvider>);
}
