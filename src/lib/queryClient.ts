import type { QueryClient } from "@tanstack/react-query";
import { QueryClient as TanStackQueryClient } from "@tanstack/react-query";

import { saveSettings } from "./ipc";
import type { AppSettings, SettingsInput } from "./types";

export const bootStatusQueryKey = ["bootStatus"] as const;
export const settingsQueryKey = ["settings"] as const;
export const portfolioQueryKey = ["portfolio"] as const;
export const projectQueryKey = (id: string) => ["project", id] as const;
export const projectMilestonesQueryKey = (id: string) => ["project", id, "milestones"] as const;
export const projectPhasePanelQueryKey = (id: string) => ["project", id, "phasePanel"] as const;

export const queryClient = new TanStackQueryClient();

export function createSaveSettingsMutationOptions(client: QueryClient) {
  return {
    mutationFn: (input: SettingsInput) => saveSettings(input),
    onSuccess: async (_settings: AppSettings) => {
      await Promise.all([
        client.invalidateQueries({ queryKey: settingsQueryKey }),
        client.invalidateQueries({ queryKey: portfolioQueryKey }),
        client.invalidateQueries({
          predicate: (query) => query.queryKey[0] === "project"
        })
      ]);
    }
  };
}
