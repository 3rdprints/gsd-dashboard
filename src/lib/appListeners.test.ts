import { beforeEach, describe, expect, it, vi } from "vitest";

import { registerAppListeners } from "./appListeners";
import { queryClient } from "./queryClient";

type ListenerCallback = (event: { payload: unknown }) => void | Promise<void>;

const listeners = new Map<string, ListenerCallback>();

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn((eventName: string, callback: ListenerCallback) => {
    listeners.set(eventName, callback);
    return Promise.resolve(vi.fn());
  })
}));

describe("live update app listeners", () => {
  beforeEach(() => {
    listeners.clear();
    vi.restoreAllMocks();
  });

  it("invalidates project queries from project:updated id payload", async () => {
    const invalidateQueries = vi.spyOn(queryClient, "invalidateQueries").mockResolvedValue();

    registerAppListeners();
    await listeners.get("project:updated")?.({ payload: { id: "project-1" } });

    expect(invalidateQueries).toHaveBeenCalledWith({ queryKey: ["portfolio"] });
    expect(invalidateQueries).toHaveBeenCalledWith({ queryKey: ["project", "project-1"] });
    expect(invalidateQueries).toHaveBeenCalledWith({ queryKey: ["project", "project-1", "milestones"] });
    expect(invalidateQueries).toHaveBeenCalledWith({ queryKey: ["project", "project-1", "phasePanel"] });
    expect(invalidateQueries).toHaveBeenCalledWith({ predicate: expect.any(Function) });
  });

  it("invalidates global and project session queries from session:new tiny payload", async () => {
    const invalidateQueries = vi.spyOn(queryClient, "invalidateQueries").mockResolvedValue();

    registerAppListeners();
    await listeners.get("session:new")?.({ payload: { id: "session-1", projectId: "project-1" } });

    expect(invalidateQueries).toHaveBeenCalledWith({ queryKey: ["portfolio"] });
    expect(invalidateQueries).toHaveBeenCalledWith({ predicate: expect.any(Function) });
  });

  it("invalidates watcher status from watcher:status-changed", async () => {
    const invalidateQueries = vi.spyOn(queryClient, "invalidateQueries").mockResolvedValue();

    registerAppListeners();
    await listeners.get("watcher:status-changed")?.({ payload: null });

    expect(invalidateQueries).toHaveBeenCalledWith({ queryKey: ["watcherStatus"] });
  });
});
