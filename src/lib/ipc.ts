import { invoke } from "@tauri-apps/api/core";

import type { AppSettings, BootStatus, SettingsInput } from "./types";

export function getBootStatus(): Promise<BootStatus> {
  return invoke<BootStatus>("get_boot_status");
}

export function getSettings(): Promise<AppSettings> {
  return invoke<AppSettings>("get_settings");
}

export function saveSettings(input: SettingsInput): Promise<AppSettings> {
  return invoke<AppSettings>("save_settings", { input });
}
