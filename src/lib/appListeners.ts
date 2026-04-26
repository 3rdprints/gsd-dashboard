import { listen } from "@tauri-apps/api/event";

import { portfolioQueryKey, queryClient, settingsQueryKey } from "./queryClient";

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

  return () => {
    void unlistenSettingsChanged.then((unlisten) => unlisten());
  };
}
