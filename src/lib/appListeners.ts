import { listen } from "@tauri-apps/api/event";

import { portfolioHeatmapQueryKey, portfolioQueryKey, queryClient, settingsQueryKey } from "./queryClient";

export function registerAppListeners() {
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
    await queryClient.invalidateQueries({ queryKey: portfolioHeatmapQueryKey });
  });

  return () => {
    void unlistenSettingsChanged.then((unlisten) => unlisten());
    void unlistenDailyActivityUpdated.then((unlisten) => unlisten());
  };
}
