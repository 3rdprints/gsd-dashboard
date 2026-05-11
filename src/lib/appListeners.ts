import { listen } from "@tauri-apps/api/event";

import {
  portfolioHeatmapQueryKey,
  portfolioQueryKey,
  projectMilestonesQueryKey,
  projectPhasePanelQueryKey,
  projectQueryKey,
  queryClient,
  settingsQueryKey,
  watcherStatusQueryKey
} from "./queryClient";

type ProjectUpdatedPayload = {
  id?: unknown;
};

type SessionNewPayload = {
  id?: unknown;
  projectId?: unknown;
};

type TrayNavigatePayload =
  | {
      event: "trayNavigate";
      data: {
        route: string;
      };
    }
  | {
      route: string;
    };

/**
 * Registers Tauri event listeners that invalidate cached application data.
 */
export function registerAppListeners() {
  if (!appListenerInternals.hasTauriInternals()) {
    return () => {};
  }

  const unlistenSettingsChanged = listen("settings-changed", async () => {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: settingsQueryKey }),
      queryClient.invalidateQueries({ queryKey: portfolioQueryKey }),
      queryClient.invalidateQueries({
        predicate: (query) => query.queryKey[0] === "project"
      })
    ]);
  });
  const unlistenDailyActivityUpdated = listen("daily_activity_updated", async () => {
    await queryClient.invalidateQueries({ queryKey: portfolioHeatmapQueryKey(90) });
  });
  const unlistenProjectUpdated = listen<ProjectUpdatedPayload>("project:updated", async (event) => {
    if (typeof event.payload.id !== "string") return;

    await Promise.all([
      queryClient.invalidateQueries({ queryKey: portfolioQueryKey }),
      queryClient.invalidateQueries({ queryKey: projectQueryKey(event.payload.id) }),
      queryClient.invalidateQueries({ queryKey: projectMilestonesQueryKey(event.payload.id) }),
      queryClient.invalidateQueries({ queryKey: projectPhasePanelQueryKey(event.payload.id) }),
      queryClient.invalidateQueries({
        predicate: (query) =>
          query.queryKey[0] === "portfolioHeatmap" ||
          (query.queryKey[0] === "project" &&
            query.queryKey[1] === event.payload.id &&
            (query.queryKey[2] === "sessions" || query.queryKey[2] === "charts"))
      })
    ]);
  });
  const unlistenSessionNew = listen<SessionNewPayload>("session:new", async (event) => {
    const projectId = typeof event.payload.projectId === "string" ? event.payload.projectId : null;

    await Promise.all([
      queryClient.invalidateQueries({ queryKey: portfolioQueryKey }),
      queryClient.invalidateQueries({
        predicate: (query) =>
          query.queryKey[0] === "globalSessions" ||
          query.queryKey[0] === "globalCharts" ||
          query.queryKey[0] === "portfolioHeatmap" ||
          (projectId !== null &&
            query.queryKey[0] === "project" &&
            query.queryKey[1] === projectId &&
            (query.queryKey[2] === "sessions" || query.queryKey[2] === "charts"))
      })
    ]);
  });
  const unlistenWatcherStatusChanged = listen("watcher:status-changed", async () => {
    await queryClient.invalidateQueries({ queryKey: watcherStatusQueryKey() });
  });
  const unlistenTrayNavigate = listen<TrayNavigatePayload>("trayNavigate", (event) => {
    const route = getTrayNavigateRoute(event.payload);

    if (route) {
      navigateToTrayRoute(route);
    }
  });

  return () => {
    void unlistenSettingsChanged.then((unlisten) => unlisten());
    void unlistenDailyActivityUpdated.then((unlisten) => unlisten());
    void unlistenProjectUpdated.then((unlisten) => unlisten());
    void unlistenSessionNew.then((unlisten) => unlisten());
    void unlistenWatcherStatusChanged.then((unlisten) => unlisten());
    void unlistenTrayNavigate.then((unlisten) => unlisten());
  };
}

/**
 * Reports whether the current runtime exposes Tauri internals.
 */
export function hasTauriInternals() {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/** Exposed internals for testing Tauri listener registration. */
export const appListenerInternals = {
  hasTauriInternals
};

function getTrayNavigateRoute(payload: TrayNavigatePayload): string | null {
  if ("data" in payload) {
    return payload.data.route;
  }

  return payload.route;
}

function navigateToTrayRoute(route: string) {
  if (!isAllowedTrayRoute(route)) {
    return;
  }

  window.history.pushState(null, "", route);
  window.dispatchEvent(new PopStateEvent("popstate"));
}

function isAllowedTrayRoute(route: string) {
  return route === "/" || route === "/settings" || /^\/project\/[^/]+$/.test(route);
}
