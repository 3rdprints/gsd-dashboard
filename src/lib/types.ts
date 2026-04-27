export type TrayBarSort = "name" | "progress" | "recent_activity";
export type GlobalSessionsDefaultRange = "7d" | "30d" | "90d" | "all";
export type ProjectSessionSortKey =
  | "startedAt"
  | "source"
  | "durationMs"
  | "messageCount"
  | "tokensIn"
  | "tokensOut"
  | "tokenTotal";
export type SortDirection = "asc" | "desc";
export type ProjectChartRange = "7d" | "30d" | "90d" | "all";

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
  globalSessionsDefaultRange: GlobalSessionsDefaultRange;
};

export type SettingsInput = {
  scanRoots: string[];
  hiddenProjectIds: string[];
  autostartEnabled: boolean;
  trayBarMaxProjects: number;
  trayBarSort: TrayBarSort;
  globalSessionsDefaultRange: GlobalSessionsDefaultRange;
};

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

export type HeatmapDay = {
  date: string;
  sessionCount: number;
  tokenTotal: number;
  topProjectId: string | null;
  topProjectName: string | null;
};

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

export type ProjectMilestonePhase = {
  number: string;
  name: string | null;
  isCurrent: boolean;
  completedAt: number | null;
  completedPlanCount: number;
  totalPlanCount: number;
};

export type ProjectMilestone = {
  name: string | null;
  progressPct: number;
  phaseCount: number;
  completedPhaseCount: number;
  phases: ProjectMilestonePhase[];
};

export type ProjectPlanItem = {
  planPath: string;
  ord: number;
  text: string;
  checked: boolean;
  lineNo: number;
};

export type ProjectPhasePanel = {
  phaseNumber: string | null;
  phaseName: string | null;
  planPath: string | null;
  statePath: string;
  stateExcerpt: string | null;
  completedItemCount: number;
  totalItemCount: number;
  items: ProjectPlanItem[];
};

export type ProjectSessionRow = {
  id: string;
  projectId?: string | null;
  projectName?: string | null;
  source: "claude" | "codex";
  sourcePath: string;
  startedAt: number | null;
  endedAt: number | null;
  durationMs: number | null;
  messageCount: number;
  tokensIn: number;
  tokensOut: number;
  tokenTotal: number;
  model: string | null;
};

export type ProjectSessionsPage = {
  rows: ProjectSessionRow[];
  total: number;
  page: number;
  pageSize: number;
};

export type GlobalSessionFilters = {
  source?: "claude" | "codex";
  projectId?: string;
  startedAfter?: number;
  startedBefore?: number;
  durationMinMs?: number;
  durationMaxMs?: number;
  tokensMin?: number;
  tokensMax?: number;
  unmatchedOnly?: boolean;
};

export type GlobalSessionsPage = {
  rows: ProjectSessionRow[];
  total: number;
  page: number;
  pageSize: number;
};

export type GlobalSessionsBySourceDay = {
  date: string;
  claude: number;
  codex: number;
};

export type GlobalTokensByProjectDay = {
  date: string;
  projectId: string | null;
  projectName: string;
  tokens: number;
};

export type GlobalHistogramBucket = {
  hour: number;
  count: number;
};

export type GlobalDayOfWeekBucket = {
  day: number;
  count: number;
};

export type GlobalChartData = {
  sessionsPerDayBySource: GlobalSessionsBySourceDay[];
  tokensPerDayByProject: GlobalTokensByProjectDay[];
  timeOfDayHistogram: GlobalHistogramBucket[];
  dayOfWeekDistribution: GlobalDayOfWeekBucket[];
};

export type ProjectDailyCount = {
  date: string;
  count: number;
};

export type ProjectDailyTokens = {
  date: string;
  tokens: number;
};

export type ProjectDailyAverageDuration = {
  date: string;
  averageDurationMs: number;
};

export type ProjectMilestoneVelocity = {
  week: string;
  completedPlans: number;
};

export type ProjectChartData = {
  sessionsPerDay: ProjectDailyCount[];
  tokensPerDay: ProjectDailyTokens[];
  averageDurationPerDay: ProjectDailyAverageDuration[];
  milestoneVelocity: ProjectMilestoneVelocity[];
};

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

export type SessionIndexClearSummary = {
  sessionsCleared?: number;
  sessions_cleared?: number;
  indexStatesCleared?: number;
  index_states_cleared?: number;
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
