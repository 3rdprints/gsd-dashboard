import { useEffect, useRef, useState } from "react";
import { Loader2, Search } from "lucide-react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { PortfolioHeaderStats } from "../components/PortfolioHeaderStats";
import { ProjectCard } from "../components/ProjectCard";
import { RightRail } from "../components/RightRail";
import { ActivityHeatmap } from "../components/charts/ActivityHeatmap";
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

export function PortfolioPage() {
  const queryClient = useQueryClient();
  const [scanState, setScanState] = useState(initialScanState);
  const [sessionIndexState, setSessionIndexState] = useState(initialSessionIndexState);
  const initialScanStarted = useRef(false);
  const isScanning = scanState.status === "scanning";
  const isIndexingSessions = sessionIndexState.status === "indexing";
  const bootStatus = useQuery({ queryKey: bootStatusQueryKey, queryFn: getBootStatus });
  const settings = useQuery({ queryKey: settingsQueryKey, queryFn: getSettings });
  const portfolio = useQuery({ queryKey: portfolioQueryKey, queryFn: getPortfolio });
  const portfolioHeatmap = useQuery({
    queryKey: portfolioHeatmapQueryKey,
    queryFn: () => getPortfolioHeatmap(90)
  });
  const saveSettings = useMutation(createSaveSettingsMutationOptions(queryClient));
  const scanProjectsMutation = useMutation({
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
  const indexSessionsMutation = useMutation({
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
        queryClient.invalidateQueries({ queryKey: portfolioHeatmapQueryKey })
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

  useEffect(() => {
    if (initialScanStarted.current || !bootStatus.data || !settings.data) {
      return;
    }

    initialScanStarted.current = true;
    void runScan();
  }, [bootStatus.data, settings.data]);

  function runScan() {
    scanProjectsMutation.mutate();
  }

  async function handleHideProject(projectId: string) {
    if (!settings.data) return;

    const nextHiddenProjectIds = settings.data.hiddenProjectIds.includes(projectId)
      ? settings.data.hiddenProjectIds
      : [...settings.data.hiddenProjectIds, projectId];

    await saveSettings.mutateAsync({
      ...settings.data,
      hiddenProjectIds: nextHiddenProjectIds
    });
  }

  function runSessionIndex() {
    indexSessionsMutation.mutate();
  }

  return (
    <div className="page-stack">
      <div className="app-header">
        <header>
          <h1>Portfolio</h1>
          <p>{portfolio.data ? `${portfolio.data.projects.length} visible projects` : "Loading projects"}</p>
        </header>
        <div className="header-actions">
          <button className="scan-cta" type="button" onClick={runScan} disabled={isScanning}>
            {isScanning ? (
              <Loader2 aria-hidden="true" size={16} strokeWidth={2} />
            ) : (
              <Search aria-hidden="true" size={16} strokeWidth={2} />
            )}
            Scan Projects
          </button>
          <button
            className="scan-cta"
            type="button"
            onClick={runSessionIndex}
            disabled={isIndexingSessions}
          >
            {isIndexingSessions ? (
              <Loader2 aria-hidden="true" size={16} strokeWidth={2} />
            ) : (
              <Search aria-hidden="true" size={16} strokeWidth={2} />
            )}
            Index Sessions
          </button>
        </div>
      </div>

      <PortfolioHeaderStats
        stats={
          portfolio.data?.stats ?? {
            projectsTracked: 0,
            activeMilestones: 0,
            sessionsToday: 0,
            tokensToday: 0
          }
        }
      />

      <div className="portfolio-activity-row">
        <div className="portfolio-status-stack">
          <ScanProgressPanel state={scanState} />
          {sessionIndexState.status !== "ready" ? (
            <SessionIndexProgressPanel state={sessionIndexState} />
          ) : null}
        </div>

        {portfolioHeatmap.isLoading ? (
          <div
            className="chart-card activity-heatmap-card"
            aria-label="Loading activity heatmap"
          >
            <div className="heatmap-skeleton" />
          </div>
        ) : (
          <ActivityHeatmap days={portfolioHeatmap.data ?? []} />
        )}
      </div>

      <div className="portfolio-layout">
        <section className="project-grid" aria-label="Projects">
          {portfolio.isLoading ? (
            <>
              <div className="project-card-skeleton" />
              <div className="project-card-skeleton" />
            </>
          ) : portfolio.data && portfolio.data.projects.length > 0 ? (
            portfolio.data.projects.map((project) => (
              <ProjectCard
                key={project.id}
                project={project}
                onHideProject={handleHideProject}
                hideDisabled={!settings.data || saveSettings.isPending}
              />
            ))
          ) : (
            <div className="empty-state">
              <h2>No projects found</h2>
              <p>
                Add a scan root or rebuild the cache to discover projects with `.planning/`
                directories.
              </p>
            </div>
          )}
        </section>

        <RightRail
          hiddenProjects={portfolio.data?.hiddenProjects ?? []}
          unmatchedSessions={
            portfolio.data?.unmatchedSessions ?? {
              count: 0,
              label: "No unmatched sessions",
              claudeCount: 0,
              codexCount: 0,
              recent: []
            }
          }
        />
      </div>
    </div>
  );
}
