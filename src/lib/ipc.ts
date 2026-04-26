import { Channel, invoke } from "@tauri-apps/api/core";

import type {
  AppSettings,
  BootStatus,
  PortfolioDto,
  ProjectDetail,
  ScanEvent,
  ScanSummary,
  SessionIndexEvent,
  SessionIndexSummary,
  SettingsInput
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

export function getProject(projectId: string): Promise<ProjectDetail> {
  return invoke<ProjectDetail>("get_project", { projectId });
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
