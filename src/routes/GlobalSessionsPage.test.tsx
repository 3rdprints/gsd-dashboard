import "@testing-library/jest-dom/vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
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
});
