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

export type PortfolioStats = {
  projectsTracked: number;
  activeMilestones: number;
  sessionsToday: number;
  tokensToday: number;
};

export type PortfolioProjectCard = {
  id: string;
  name: string;
  rootPath: string;
  planningPath: string;
  currentMilestoneName: string | null;
  currentPhaseNumber: string | null;
  currentPhaseName: string | null;
  milestoneProgressPct: number;
  nextCommand: string;
  parseError: string | null;
  lastActivityAt: number | null;
  lastScannedAt: number;
  sessionSparkline7d: SessionSparklineDay[];
  sessionsLast7d: number;
};

export type SessionSparklineDay = { date: string; count: number };

export type HiddenProject = {
  id: string;
  name: string;
  rootPath: string;
};

export type UnmatchedSessions = {
  count: number;
  label: string;
  claudeCount: number;
  codexCount: number;
  recent: RecentUnmatchedSession[];
};

export type RecentUnmatchedSession = {
  id: string;
  source: "claude" | "codex";
  sourcePath: string;
  startedAt: number | null;
};

export type PortfolioDto = {
  stats: PortfolioStats;
  projects: PortfolioProjectCard[];
  hiddenProjects: HiddenProject[];
  unmatchedSessions: UnmatchedSessions;
};

export type ProjectDetail = PortfolioProjectCard;

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

export type SessionIndexSummary = {
  rootCount?: number;
  root_count?: number;
  filesProcessed?: number;
  files_processed?: number;
  sessionsPersisted?: number;
  sessions_persisted?: number;
  unmatchedCount?: number;
  unmatched_count?: number;
  errorCount?: number;
  error_count?: number;
};

export type SessionIndexEvent =
  | {
      event: "started";
      data: {
        rootCount: number;
      };
    }
  | {
      event: "sourceStarted";
      data: {
        source: "claude" | "codex";
        rootPath: string;
      };
    }
  | {
      event: "fileIndexed";
      data: {
        source: "claude" | "codex";
        sourcePath: string;
        sessionsPersisted: number;
        livePartial: boolean;
      };
    }
  | {
      event: "fileIndexError";
      data: {
        source: "claude" | "codex";
        sourcePath: string;
        message: string;
      };
    }
  | {
      event: "finished";
      data: SessionIndexSummary;
    };

export type AppError = {
  kind: "store" | "settings" | "io" | "invalidScanRoot";
  message: string;
  path?: string;
  reason?: string;
};
