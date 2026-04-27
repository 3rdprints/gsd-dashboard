import { describe, expect, it, vi } from "vitest";

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn()
}));

vi.mock("@tauri-apps/api/core", () => ({
  Channel: class TestChannel<T> {
    onmessage: ((event: T) => void) | null = null;
  },
  invoke: invokeMock
}));

describe("portfolio heatmap IPC contracts", () => {
  it("exposes a typed heatmap wrapper and stable query key", async () => {
    invokeMock.mockResolvedValue([
      {
        date: "2026-04-27",
        sessionCount: 2,
        tokenTotal: 1200,
        topProjectName: "GSD Dashboard"
      }
    ]);

    const ipc = await import("./ipc");
    const queryClient = await import("./queryClient");
    const typesModule = await import("./types");

    expect(typeof (ipc as Record<string, unknown>).getPortfolioHeatmap).toBe("function");
    expect(typeof (queryClient as Record<string, unknown>).portfolioHeatmapQueryKey).toBe("function");
    expect(typesModule).toBeDefined();

    await (ipc as { getPortfolioHeatmap: (days: number) => Promise<unknown> }).getPortfolioHeatmap(90);

    expect(invokeMock).toHaveBeenCalledWith("get_portfolio_heatmap", { days: 90 });
    expect((queryClient as { portfolioHeatmapQueryKey: readonly unknown[] }).portfolioHeatmapQueryKey).toEqual([
      "portfolioHeatmap",
      90
    ]);
  });
});
