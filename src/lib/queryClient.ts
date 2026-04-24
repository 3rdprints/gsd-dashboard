import type { QueryClient } from "@tanstack/react-query";
import { QueryClient as TanStackQueryClient } from "@tanstack/react-query";

import { saveSettings } from "./ipc";
import type { AppSettings, SettingsInput } from "./types";

export const bootStatusQueryKey = ["bootStatus"] as const;
export const settingsQueryKey = ["settings"] as const;

export const queryClient = new TanStackQueryClient();

export function createSaveSettingsMutationOptions(client: QueryClient) {
  return {
    mutationFn: (input: SettingsInput) => saveSettings(input),
    onSuccess: async (_settings: AppSettings) => {
      await client.invalidateQueries({ queryKey: settingsQueryKey });
    }
  };
}
