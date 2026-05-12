import type { QueryClient } from "@tanstack/react-query";
import { QueryClient as TanStackQueryClient } from "@tanstack/react-query";

import { saveSettings } from "./ipc";
import type {
  AppSettings,
  GlobalSessionFilters,
  ProjectChartRange,
  ProjectSessionSortKey,
  SettingsInput,
  SortDirection
} from "./types";

/** Query key for the boot status cache entry. */
export const bootStatusQueryKey = ["bootStatus"] as const;
/** Query key for the current desktop app version cache entry. */
export const currentVersionQueryKey = ["currentVersion"] as const;
/** Query key for the settings cache entry. */
export const settingsQueryKey = ["settings"] as const;
/**
 * Builds the watcher status query key for TanStack Query cache entries.
 */
export const watcherStatusQueryKey = () => ["watcherStatus"] as const;
/** Query key for the portfolio cache entry. */
export const portfolioQueryKey = ["portfolio"] as const;
/**
 * Builds the portfolio heatmap query key for TanStack Query cache entries.
 */
export const portfolioHeatmapQueryKey = (days: number) => ["portfolioHeatmap", days] as const;
/**
 * Builds the project query key for TanStack Query cache entries.
 */
export const projectQueryKey = (id: string) => ["project", id] as const;
/**
 * Builds the project milestones query key for TanStack Query cache entries.
 */
export const projectMilestonesQueryKey = (id: string) => ["project", id, "milestones"] as const;
/**
 * Builds the project phase panel query key for TanStack Query cache entries.
 */
export const projectPhasePanelQueryKey = (id: string) => ["project", id, "phasePanel"] as const;
/**
 * Builds the project sessions query key for TanStack Query cache entries.
 */
export const projectSessionsQueryKey = (id: string, sort: ProjectSessionSortKey, direction: SortDirection, page: number, pageSize: number) => ["project", id, "sessions", sort, direction, page, pageSize] as const;
/**
 * Builds the project charts query key for TanStack Query cache entries.
 */
export const projectChartsQueryKey = (id: string, range: ProjectChartRange) => ["project", id, "charts", range] as const;
/** Builds the global sessions query key for TanStack Query cache entries. */
export const globalSessionsQueryKey = (
  filters: GlobalSessionFilters,
  sort: ProjectSessionSortKey,
  direction: SortDirection,
  page: number,
  pageSize: number
) => ["globalSessions", filters, sort, direction, page, pageSize] as const;
/**
 * Builds the global charts query key for TanStack Query cache entries.
 */
export const globalChartsQueryKey = (filters: GlobalSessionFilters) => ["globalCharts", filters] as const;

/** Shared TanStack Query client instance for the application. */
export const queryClient = new TanStackQueryClient();

/**
 * Provides the exported create save settings mutation options function.
 */
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
