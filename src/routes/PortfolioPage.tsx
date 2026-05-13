import { useEffect, useRef, useState } from "react";
import { Loader2, Search } from "lucide-react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { PortfolioHeaderStats } from "../components/PortfolioHeaderStats";
import { ProjectCard } from "../components/ProjectCard";
import { RightRail } from "../components/RightRail";
import { ActivityHeatmap } from "../components/charts/ActivityHeatmap";
import { Button } from "../components/ui/button";
import {
  completeScanState,
  initialScanState,
  reduceScanEvent,
  ScanProgressPanel
} from "../components/ScanProgressPanel";
import {
  completeSessionIndexState,
  initialSessionIndexState,
  reduceSessionIndexEvent,
  SessionIndexProgressPanel
} from "../components/SessionIndexProgressPanel";
import { getBootStatus, getPortfolio, getPortfolioHeatmap, getSettings, indexSessions, scanProjects } from "../lib/ipc";
import {
  bootStatusQueryKey,
  createSaveSettingsMutationOptions,
  portfolioHeatmapQueryKey,
  portfolioQueryKey,
  settingsQueryKey
} from "../lib/queryClient";
import type { PortfolioDto, SettingsInput } from "../lib/types";

const DEFAULT_PORTFOLIO_STATS: PortfolioDto["stats"] = {
  projectsTracked: 0,
  activeMilestones: 0,
  sessionsToday: 0,
  tokensToday: 0
};

const DEFAULT_UNMATCHED_SESSIONS: PortfolioDto["unmatchedSessions"] = {
  count: 0,
  label: "No unmatched sessions",
  claudeCount: 0,
  codexCount: 0,
  recent: []
};

const getVisibleProjectCount = (portfolio: PortfolioDto | undefined) => {
  if (!portfolio) {
    return null;
  }

  return portfolio.projects.length;
};

const getPortfolioStats = (portfolio: PortfolioDto | undefined) =>
  portfolio ? portfolio.stats : DEFAULT_PORTFOLIO_STATS;

const getPortfolioProjects = (portfolio: PortfolioDto | undefined) => (portfolio ? portfolio.projects : []);

const getHiddenProjects = (portfolio: PortfolioDto | undefined) => (portfolio ? portfolio.hiddenProjects : []);

const getUnmatchedSessions = (portfolio: PortfolioDto | undefined) =>
  portfolio ? portfolio.unmatchedSessions : DEFAULT_UNMATCHED_SESSIONS;

type PortfolioProjectsProps = {
  hideDisabled: boolean;
  isLoading: boolean;
  isScanning: boolean;
  onHideProject: (projectId: string) => void;
  projects: PortfolioDto["projects"];
  runScan: () => void;
};

type PortfolioHeaderProps = {
  isIndexingSessions: boolean;
  isScanning: boolean;
  onRunScan: () => void;
  onRunSessionIndex: () => void;
  visibleProjectCount: number | null;
};

type PortfolioActivityRowProps = {
  heatmapDays: Awaited<ReturnType<typeof getPortfolioHeatmap>> | undefined;
  isHeatmapLoading: boolean;
  scanState: typeof initialScanState;
  sessionIndexState: typeof initialSessionIndexState;
};

type ScanState = typeof initialScanState;
type SessionIndexState = typeof initialSessionIndexState;
type SetScanState = (value: ScanState | ((current: ScanState) => ScanState)) => void;
type SetSessionIndexState = (
  value: SessionIndexState | ((current: SessionIndexState) => SessionIndexState)
) => void;

const PortfolioHeatmapLoading = () => (
  <div className="chart-card activity-heatmap-card" aria-label="Loading activity heatmap">
    <div className="chart-card-header">
      <div>
        <h2 className="chart-card-title">Activity heatmap</h2>
        <p className="chart-card-subtitle">Loading 90 days of session activity</p>
      </div>
    </div>
    <div className="heatmap-skeleton labeled-skeleton">
      <span>Activity loading</span>
    </div>
  </div>
);

const PortfolioHeader = ({
  isIndexingSessions,
  isScanning,
  onRunScan,
  onRunSessionIndex,
  visibleProjectCount
}: PortfolioHeaderProps) => (
  <div className="app-header">
    <header>
      <h1>Portfolio</h1>
      <p>{visibleProjectCount === null ? "Loading projects" : `${visibleProjectCount} visible projects`}</p>
    </header>
    <div className="header-actions">
      <Button className="scan-cta" type="button" onClick={onRunScan} disabled={isScanning}>
        {isScanning ? (
          <Loader2 aria-hidden="true" size={16} strokeWidth={2} />
        ) : (
          <Search aria-hidden="true" size={16} strokeWidth={2} />
        )}
        Scan Projects
      </Button>
      <Button
        className="scan-cta"
        type="button"
        onClick={onRunSessionIndex}
        disabled={isIndexingSessions}
      >
        {isIndexingSessions ? (
          <Loader2 aria-hidden="true" size={16} strokeWidth={2} />
        ) : (
          <Search aria-hidden="true" size={16} strokeWidth={2} />
        )}
        Index Sessions
      </Button>
    </div>
  </div>
);

const PortfolioActivityRow = ({
  heatmapDays,
  isHeatmapLoading,
  scanState,
  sessionIndexState
}: PortfolioActivityRowProps) => (
  <div className="portfolio-activity-row">
    <div className="portfolio-status-stack">
      <ScanProgressPanel state={scanState} />
      {sessionIndexState.status !== "ready" ? (
        <SessionIndexProgressPanel state={sessionIndexState} />
      ) : null}
    </div>

    {isHeatmapLoading ? <PortfolioHeatmapLoading /> : <ActivityHeatmap days={heatmapDays ?? []} />}
  </div>
);

const ProjectLoadingCards = () => (
  <>
    <div className="project-card-skeleton" aria-label="Loading project">
      <p className="label-text">Project</p>
      <h2>Loading project status</h2>
      <p>Current milestone, phase, and activity will appear here.</p>
      <div className="skeleton-line" />
    </div>
    <div className="project-card-skeleton" aria-label="Loading project">
      <p className="label-text">Project</p>
      <h2>Loading project status</h2>
      <p>Session trend and next action will appear here.</p>
      <div className="skeleton-line" />
    </div>
  </>
);

const PortfolioEmptyState = ({ isScanning, runScan }: Pick<PortfolioProjectsProps, "isScanning" | "runScan">) => (
  <div className="empty-state">
    <h2>No projects found</h2>
    <p>Add a scan root or rebuild the cache to discover projects with `.planning/` directories.</p>
    <div className="empty-state-actions">
      <Button type="button" onClick={runScan} disabled={isScanning}>
        {isScanning ? (
          <Loader2 aria-hidden="true" size={16} strokeWidth={2} />
        ) : (
          <Search aria-hidden="true" size={16} strokeWidth={2} />
        )}
        Scan Projects
      </Button>
    </div>
  </div>
);

const PortfolioProjects = ({
  hideDisabled,
  isLoading,
  isScanning,
  onHideProject,
  projects,
  runScan
}: PortfolioProjectsProps) => {
  if (isLoading) {
    return <ProjectLoadingCards />;
  }

  if (projects.length === 0) {
    return <PortfolioEmptyState isScanning={isScanning} runScan={runScan} />;
  }

  return projects.map((project) => (
    <ProjectCard
      key={project.id}
      project={project}
      onHideProject={onHideProject}
      hideDisabled={hideDisabled}
    />
  ));
};

const getNextHiddenProjectIds = (settings: SettingsInput, projectId: string) =>
  settings.hiddenProjectIds.includes(projectId)
    ? settings.hiddenProjectIds
    : [...settings.hiddenProjectIds, projectId];

const getHideProjectSettings = (settings: SettingsInput | undefined, projectId: string) =>
  settings
    ? {
        ...settings,
        hiddenProjectIds: getNextHiddenProjectIds(settings, projectId)
      }
    : undefined;

const useScanProjectsMutation = (queryClient: ReturnType<typeof useQueryClient>, setScanState: SetScanState) =>
  useMutation({
    mutationFn: () =>
      scanProjects((event) => {
        setScanState((current) => reduceScanEvent(current, event));
      }),
    onMutate: () => {
      setScanState({
        ...initialScanState,
        status: "scanning",
        progressText: "Walking scan roots"
      });
    },
    onSuccess: async (summary) => {
      setScanState((current) => completeScanState(current, summary));
      await queryClient.invalidateQueries({ queryKey: portfolioQueryKey });
    },
    onError: () => {
      setScanState((current) => ({
        ...current,
        status: "failed",
        progressText: "Scan failed"
      }));
    }
  });

const useIndexSessionsMutation = (
  queryClient: ReturnType<typeof useQueryClient>,
  setSessionIndexState: SetSessionIndexState
) =>
  useMutation({
    mutationFn: () =>
      indexSessions((event) => {
        setSessionIndexState((current) => reduceSessionIndexEvent(current, event));
      }),
    onMutate: () => {
      setSessionIndexState({
        ...initialSessionIndexState,
        status: "indexing",
        progressText: "Indexing sessions"
      });
    },
    onSuccess: async (summary) => {
      setSessionIndexState((current) => completeSessionIndexState(current, summary));
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: portfolioQueryKey }),
        queryClient.invalidateQueries({ queryKey: portfolioHeatmapQueryKey(90) })
      ]);
    },
    onError: () => {
      setSessionIndexState((current) => ({
        ...current,
        status: "failed",
        progressText: "Some session files could not be indexed"
      }));
    }
  });

const useInitialScan = (bootReady: boolean, settingsReady: boolean, runScan: () => void) => {
  const initialScanStarted = useRef(false);

  useEffect(() => {
    if (initialScanStarted.current || !bootReady || !settingsReady) {
      return;
    }

    initialScanStarted.current = true;
    void runScan();
  }, [bootReady, runScan, settingsReady]);
};

/**
 * Renders the portfolio route.
 */
export const PortfolioPage = () => {
  const queryClient = useQueryClient();
  const [scanState, setScanState] = useState(initialScanState);
  const [sessionIndexState, setSessionIndexState] = useState(initialSessionIndexState);
  const isScanning = scanState.status === "scanning";
  const isIndexingSessions = sessionIndexState.status === "indexing";
  const bootStatus = useQuery({ queryKey: bootStatusQueryKey, queryFn: getBootStatus });
  const settings = useQuery({ queryKey: settingsQueryKey, queryFn: getSettings });
  const portfolio = useQuery({ queryKey: portfolioQueryKey, queryFn: getPortfolio });
  const portfolioHeatmap = useQuery({
    queryKey: portfolioHeatmapQueryKey(90),
    queryFn: () => getPortfolioHeatmap(90)
  });
  const saveSettings = useMutation(createSaveSettingsMutationOptions(queryClient));
  const scanProjectsMutation = useScanProjectsMutation(queryClient, setScanState);
  const indexSessionsMutation = useIndexSessionsMutation(queryClient, setSessionIndexState);

  function runScan() {
    scanProjectsMutation.mutate();
  }

  useInitialScan(Boolean(bootStatus.data), Boolean(settings.data), runScan);

  async function handleHideProject(projectId: string) {
    const nextSettings = getHideProjectSettings(settings.data, projectId);
    if (nextSettings) await saveSettings.mutateAsync(nextSettings);
  }

  function runSessionIndex() {
    indexSessionsMutation.mutate();
  }

  const portfolioData = portfolio.data;

  return (
    <div className="page-stack">
      <PortfolioHeader
        isIndexingSessions={isIndexingSessions}
        isScanning={isScanning}
        onRunScan={runScan}
        onRunSessionIndex={runSessionIndex}
        visibleProjectCount={getVisibleProjectCount(portfolioData)}
      />

      <PortfolioHeaderStats
        stats={getPortfolioStats(portfolioData)}
      />

      <PortfolioActivityRow
        heatmapDays={portfolioHeatmap.data}
        isHeatmapLoading={portfolioHeatmap.isLoading}
        scanState={scanState}
        sessionIndexState={sessionIndexState}
      />

      <div className="portfolio-layout">
        <section className="project-grid" aria-label="Projects">
          <PortfolioProjects
            hideDisabled={!settings.data || saveSettings.isPending}
            isLoading={portfolio.isLoading}
            isScanning={isScanning}
            onHideProject={handleHideProject}
            projects={getPortfolioProjects(portfolioData)}
            runScan={runScan}
          />
        </section>

        <RightRail
          hiddenProjects={getHiddenProjects(portfolioData)}
          unmatchedSessions={getUnmatchedSessions(portfolioData)}
        />
      </div>
    </div>
  );
};
