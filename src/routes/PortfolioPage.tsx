import { useEffect, useRef, useState } from "react";
import { Loader2, Search } from "lucide-react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { PortfolioHeaderStats } from "../components/PortfolioHeaderStats";
import { ProjectCard } from "../components/ProjectCard";
import { RightRail } from "../components/RightRail";
import {
  completeScanState,
  initialScanState,
  reduceScanEvent,
  ScanProgressPanel
} from "../components/ScanProgressPanel";
import { getBootStatus, getPortfolio, getSettings, scanProjects } from "../lib/ipc";
import {
  bootStatusQueryKey,
  createSaveSettingsMutationOptions,
  portfolioQueryKey,
  settingsQueryKey
} from "../lib/queryClient";

export function PortfolioPage() {
  const queryClient = useQueryClient();
  const [scanState, setScanState] = useState(initialScanState);
  const initialScanStarted = useRef(false);
  const isScanning = scanState.status === "scanning";
  const bootStatus = useQuery({ queryKey: bootStatusQueryKey, queryFn: getBootStatus });
  const settings = useQuery({ queryKey: settingsQueryKey, queryFn: getSettings });
  const portfolio = useQuery({ queryKey: portfolioQueryKey, queryFn: getPortfolio });
  const saveSettings = useMutation(createSaveSettingsMutationOptions(queryClient));

  useEffect(() => {
    if (initialScanStarted.current || !bootStatus.data || !settings.data) {
      return;
    }

    initialScanStarted.current = true;
    void runScan();
  }, [bootStatus.data, settings.data]);

  async function runScan() {
    setScanState({
      ...initialScanState,
      status: "scanning",
      progressText: "Walking scan roots"
    });

    try {
      const summary = await scanProjects((event) => {
        setScanState((current) => reduceScanEvent(current, event));
      });
      setScanState((current) => completeScanState(current, summary));
      await queryClient.invalidateQueries({ queryKey: portfolioQueryKey });
    } catch {
      setScanState((current) => ({
        ...current,
        status: "failed",
        progressText: "Scan failed"
      }));
    }
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

  return (
    <div className="page-stack">
      <div className="app-header">
        <header>
          <h1>Portfolio</h1>
          <p>{portfolio.data ? `${portfolio.data.projects.length} visible projects` : "Loading projects"}</p>
        </header>
        <button className="scan-cta" type="button" onClick={runScan} disabled={isScanning}>
          {isScanning ? (
            <Loader2 aria-hidden="true" size={16} strokeWidth={2} />
          ) : (
            <Search aria-hidden="true" size={16} strokeWidth={2} />
          )}
          Scan Projects
        </button>
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

      <ScanProgressPanel state={scanState} />

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
              label: "Available after session indexing"
            }
          }
        />
      </div>
    </div>
  );
}
