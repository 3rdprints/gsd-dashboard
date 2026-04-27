import "@testing-library/jest-dom/vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { BrowserRouter } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";

describe("GlobalSessionsPage", () => {
  beforeEach(() => {
    vi.resetModules();
  });

  it("adds the Sessions top-level navigation link", async () => {
    vi.doMock("../lib/appListeners", () => ({
      registerAppListeners: () => undefined
    }));
    const { App } = await import("../App");

    render(
      <QueryClientProvider client={new QueryClient()}>
        <App />
      </QueryClientProvider>
    );

    expect(screen.getByRole("link", { name: "Portfolio" })).toHaveAttribute("href", "/");
    expect(screen.getByRole("link", { name: "Sessions" })).toHaveAttribute("href", "/sessions");
    expect(screen.getByRole("link", { name: "Settings" })).toHaveAttribute("href", "/settings");
  });

  it("exposes the listGlobalSessions IPC wrapper and stable query key", async () => {
    const invoke = vi.fn().mockResolvedValue({ rows: [], total: 0, page: 1, pageSize: 100 });
    vi.doMock("@tauri-apps/api/core", () => ({
      Channel: class TestChannel<T> {
        onmessage: ((event: T) => void) | null = null;
      },
      invoke
    }));

    const { listGlobalSessions } = await import("../lib/ipc");
    const { globalSessionsQueryKey } = await import("../lib/queryClient");
    const filters = { source: "claude" as const, unmatchedOnly: true };

    await listGlobalSessions(filters, 2, 100);

    expect(invoke).toHaveBeenCalledWith("list_global_sessions", { filters, page: 2, pageSize: 100 });
    expect(globalSessionsQueryKey(filters, 2, 100)).toEqual(["globalSessions", filters, 2, 100]);
  });

  it("coerces URL filters and rejects invalid numeric and source values", async () => {
    const { DEFAULT_FILTERS, parseFiltersFromUrl, serializeFiltersToUrl } = await import("../lib/sessionFilters");
    const params = new URLSearchParams({
      source: "claude'; DROP TABLE sessions; --",
      dmin: "NaN",
      dmax: "Infinity",
      tmin: "-1",
      tmax: "1200",
      unmatched: "true",
      page: "3"
    });

    const filters = parseFiltersFromUrl(params, DEFAULT_FILTERS({ globalSessionsDefaultRange: "7d" }));

    expect(filters.source).toBeUndefined();
    expect(filters.durationMinMinutes).toBeUndefined();
    expect(filters.durationMaxMinutes).toBeUndefined();
    expect(filters.tokensMin).toBeUndefined();
    expect(filters.tokensMax).toBe(1200);
    expect(filters.unmatchedOnly).toBe(true);
    expect(filters.page).toBe(3);
    expect(serializeFiltersToUrl(filters).get("source")).toBeNull();
  });

  it("updates URL-backed filters with debounced numeric inputs and removable chips", async () => {
    const invoke = vi.fn((command: string) => {
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
          stats: { projectsTracked: 1, activeMilestones: 1, sessionsToday: 0, tokensToday: 0 },
          projects: [
            {
              id: "project-1",
              name: "Dashboard",
              rootPath: "/tmp/dashboard",
              planningPath: "/tmp/dashboard/.planning",
              currentMilestoneName: "v1",
              currentPhaseNumber: "05",
              currentPhaseName: "UI",
              milestoneProgressPct: 50,
              nextCommand: "/gsd-next",
              parseError: null,
              lastActivityAt: null,
              lastScannedAt: 0,
              sessionSparkline7d: [],
              sessionsLast7d: 0
            }
          ],
          hiddenProjects: [],
          unmatchedSessions: { count: 0, label: "0 unmatched", claudeCount: 0, codexCount: 0, recent: [] }
        });
      }
      if (command === "list_global_sessions") {
        return Promise.resolve({ rows: [], total: 0, page: 1, pageSize: 100 });
      }
      if (command === "save_settings") {
        return Promise.resolve({
          scanRoots: [],
          hiddenProjectIds: [],
          autostartEnabled: false,
          trayBarMaxProjects: 6,
          trayBarSort: "name",
          globalSessionsDefaultRange: "30d"
        });
      }
      return Promise.resolve(null);
    });
    vi.doMock("@tauri-apps/api/core", () => ({
      Channel: class TestChannel<T> {
        onmessage: ((event: T) => void) | null = null;
      },
      invoke
    }));
    const { GlobalSessionsPage } = await import("./GlobalSessionsPage");

    window.history.replaceState(null, "", "/sessions");
    render(
      <QueryClientProvider client={new QueryClient()}>
        <BrowserRouter>
          <GlobalSessionsPage />
        </BrowserRouter>
      </QueryClientProvider>
    );

    fireEvent.change(await screen.findByLabelText("Source"), { target: { value: "claude" } });
    expect(await screen.findByText("Source: Claude")).toBeInTheDocument();
    expect(window.location.search).toContain("source=claude");

    fireEvent.change(screen.getByLabelText("Minimum duration"), { target: { value: "5" } });
    expect(window.location.search).not.toContain("dmin=5");
    await new Promise((resolve) => window.setTimeout(resolve, 350));
    expect(window.location.search).toContain("dmin=5");

    fireEvent.click(screen.getByLabelText("Remove source filter"));
    expect(screen.queryByText("Source: Claude")).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Clear all" }));
    await waitFor(() => expect(window.location.search).not.toContain("dmin=5"));
  });
});
