import { Channel, invoke } from "@tauri-apps/api/core";

import type { AppSettings, BootStatus, ScanEvent, ScanSummary, SettingsInput } from "./types";

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
