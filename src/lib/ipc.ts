import { Channel, invoke } from "@tauri-apps/api/core";

import type {
  AppSettings,
  BootStatus,
  GlobalChartData,
  GlobalSessionFilters,
  GlobalSessionsPage,
  HeatmapDay,
  PortfolioDto,
  ProjectChartData,
  ProjectChartRange,
  ProjectDetail,
  ProjectMilestone,
  ProjectPhasePanel,
  ProjectSessionsPage,
  ProjectSessionSortKey,
  ScanEvent,
  ScanSummary,
  SessionIndexEvent,
  SessionIndexClearSummary,
  SessionIndexSummary,
  SettingsInput,
  SortDirection,
  WatcherStatus
} from "./types";

/**
 * Invokes the backend command for get boot status.
 */
export function getBootStatus(): Promise<BootStatus> {
  return invoke<BootStatus>("get_boot_status");
}

/**
 * Invokes the backend command for get settings.
 */
export function getSettings(): Promise<AppSettings> {
  return invoke<AppSettings>("get_settings");
}

/**
 * Invokes the backend command for get watcher status.
 */
export function getWatcherStatus(): Promise<WatcherStatus> {
  return invoke<WatcherStatus>("get_watcher_status");
}

/**
 * Invokes the backend command for save settings.
 */
export function saveSettings(input: SettingsInput): Promise<AppSettings> {
  return invoke<AppSettings>("save_settings", { input });
}

/**
 * Invokes the backend command for scan projects.
 */
export function scanProjects(onEvent: (event: ScanEvent) => void): Promise<ScanSummary> {
  const onEventChannel = new Channel<ScanEvent>();
  onEventChannel.onmessage = onEvent;

  return invoke<ScanSummary>("scan_projects", { onEvent: onEventChannel });
}

/**
 * Invokes the backend command for get portfolio.
 */
export function getPortfolio(): Promise<PortfolioDto> {
  return invoke<PortfolioDto>("get_portfolio");
}

/**
 * Invokes the backend command for get portfolio heatmap.
 */
export function getPortfolioHeatmap(days: number): Promise<HeatmapDay[]> {
  return invoke<HeatmapDay[]>("get_portfolio_heatmap", { days });
}

/**
 * Invokes the backend command for get project.
 */
export function getProject(projectId: string): Promise<ProjectDetail> {
  return invoke<ProjectDetail>("get_project", { projectId });
}

/**
 * Invokes the backend command for get project milestones.
 */
export function getProjectMilestones(projectId: string): Promise<ProjectMilestone[]> {
  return invoke<ProjectMilestone[]>("get_project_milestones", { projectId });
}

/**
 * Renders the get project phase panel.
 */
export function getProjectPhasePanel(projectId: string): Promise<ProjectPhasePanel> {
  return invoke<ProjectPhasePanel>("get_project_phase_panel", { projectId });
}

/**
 * Invokes the backend command for list project sessions.
 */
export function listProjectSessions(
  projectId: string,
  sort: ProjectSessionSortKey,
  direction: SortDirection,
  page: number,
  pageSize: number
): Promise<ProjectSessionsPage> {
  return invoke<ProjectSessionsPage>("list_project_sessions", { projectId, sort, direction, page, page_size: pageSize });
}

/**
 * Invokes the backend command for list global sessions.
 */
export function listGlobalSessions(
  filters: GlobalSessionFilters,
  sort: ProjectSessionSortKey,
  direction: SortDirection,
  page: number,
  pageSize: number
): Promise<GlobalSessionsPage> {
  return invoke<GlobalSessionsPage>("list_global_sessions", { filters, sort, direction, page, page_size: pageSize });
}

/**
 * Invokes the backend command for get global chart data.
 */
export function getGlobalChartData(filters: GlobalSessionFilters): Promise<GlobalChartData> {
  return invoke<GlobalChartData>("get_global_chart_data", { filters });
}

/**
 * Invokes the backend command for get project chart data.
 */
export function getProjectChartData(projectId: string, range: ProjectChartRange): Promise<ProjectChartData> {
  return invoke<ProjectChartData>("get_project_chart_data", { projectId, range });
}

/**
 * Invokes the backend command for rebuild cache.
 */
export function rebuildCache(onEvent: (event: ScanEvent) => void): Promise<ScanSummary> {
  const onEventChannel = new Channel<ScanEvent>();
  onEventChannel.onmessage = onEvent;

  return invoke<ScanSummary>("rebuild_cache", { onEvent: onEventChannel });
}

/**
 * Invokes the backend command for index sessions.
 */
export function indexSessions(onEvent: (event: SessionIndexEvent) => void): Promise<SessionIndexSummary> {
  const onEventChannel = new Channel<SessionIndexEvent>();
  onEventChannel.onmessage = onEvent;

  return invoke<SessionIndexSummary>("index_sessions", { onEvent: onEventChannel });
}

/**
 * Invokes the backend command for clear session index.
 */
export function clearSessionIndex(): Promise<SessionIndexClearSummary> {
  return invoke<SessionIndexClearSummary>("clear_session_index");
}
