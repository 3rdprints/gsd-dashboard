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
  SortDirection
} from "./types";

export function getBootStatus(): Promise<BootStatus> {
  return invoke<BootStatus>("get_boot_status");
}

export function getSettings(): Promise<AppSettings> {
  return invoke<AppSettings>("get_settings");
}

export function saveSettings(input: SettingsInput): Promise<AppSettings> {
  return invoke<AppSettings>("save_settings", { input });
}

export function scanProjects(onEvent: (event: ScanEvent) => void): Promise<ScanSummary> {
  const onEventChannel = new Channel<ScanEvent>();
  onEventChannel.onmessage = onEvent;

  return invoke<ScanSummary>("scan_projects", { onEvent: onEventChannel });
}

export function getPortfolio(): Promise<PortfolioDto> {
  return invoke<PortfolioDto>("get_portfolio");
}

export function getPortfolioHeatmap(days: number): Promise<HeatmapDay[]> {
  return invoke<HeatmapDay[]>("get_portfolio_heatmap", { days });
}

export function getProject(projectId: string): Promise<ProjectDetail> {
  return invoke<ProjectDetail>("get_project", { projectId });
}

export function getProjectMilestones(projectId: string): Promise<ProjectMilestone[]> {
  return invoke<ProjectMilestone[]>("get_project_milestones", { projectId });
}

export function getProjectPhasePanel(projectId: string): Promise<ProjectPhasePanel> {
  return invoke<ProjectPhasePanel>("get_project_phase_panel", { projectId });
}

export function listProjectSessions(
  projectId: string,
  sort: ProjectSessionSortKey,
  direction: SortDirection,
  page: number,
  pageSize: number
): Promise<ProjectSessionsPage> {
  return invoke<ProjectSessionsPage>("list_project_sessions", { projectId, sort, direction, page, page_size: pageSize });
}

export function listGlobalSessions(
  filters: GlobalSessionFilters,
  page: number,
  pageSize: number
): Promise<GlobalSessionsPage> {
  return invoke<GlobalSessionsPage>("list_global_sessions", { filters, page, page_size: pageSize });
}

export function getGlobalChartData(filters: GlobalSessionFilters): Promise<GlobalChartData> {
  return invoke<GlobalChartData>("get_global_chart_data", { filters });
}

export function getProjectChartData(projectId: string, range: ProjectChartRange): Promise<ProjectChartData> {
  return invoke<ProjectChartData>("get_project_chart_data", { projectId, range });
}

export function rebuildCache(onEvent: (event: ScanEvent) => void): Promise<ScanSummary> {
  const onEventChannel = new Channel<ScanEvent>();
  onEventChannel.onmessage = onEvent;

  return invoke<ScanSummary>("rebuild_cache", { onEvent: onEventChannel });
}

export function indexSessions(onEvent: (event: SessionIndexEvent) => void): Promise<SessionIndexSummary> {
  const onEventChannel = new Channel<SessionIndexEvent>();
  onEventChannel.onmessage = onEvent;

  return invoke<SessionIndexSummary>("index_sessions", { onEvent: onEventChannel });
}

export function clearSessionIndex(): Promise<SessionIndexClearSummary> {
  return invoke<SessionIndexClearSummary>("clear_session_index");
}
