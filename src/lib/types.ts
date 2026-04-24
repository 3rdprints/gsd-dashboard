export type TrayBarSort = "name" | "progress" | "recent_activity";

export type BootStatus = {
  appDataDir: string;
  cachePath: string;
  cacheReady: boolean;
  walEnabled: boolean;
  migrationsApplied: number;
  settingsInitialized: boolean;
};

export type AppSettings = {
  scanRoots: string[];
  hiddenProjectIds: string[];
  autostartEnabled: boolean;
  trayBarMaxProjects: number;
  trayBarSort: TrayBarSort;
};

export type SettingsInput = AppSettings;

export type AppError = {
  kind: "store" | "settings" | "io" | "invalidScanRoot";
  message: string;
  path?: string;
  reason?: string;
};
