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

export interface ScanSummary {
  discoveredCount: number;
  parsedCount: number;
  errorCount: number;
}

export type ScanEvent =
  | {
      event: "started";
      data: {
        rootCount: number;
      };
    }
  | {
      event: "rootStarted";
      data: {
        rootPath: string;
      };
    }
  | {
      event: "projectFound";
      data: {
        projectId: string;
        projectName: string;
        rootPath: string;
      };
    }
  | {
      event: "projectParsed";
      data: {
        projectId: string;
        projectName: string;
      };
    }
  | {
      event: "projectParseError";
      data: {
        projectId: string;
        projectName: string;
        filePath: string;
        message: string;
      };
    }
  | {
      event: "finished";
      data: ScanSummary;
    };

export type AppError = {
  kind: "store" | "settings" | "io" | "invalidScanRoot";
  message: string;
  path?: string;
  reason?: string;
};
