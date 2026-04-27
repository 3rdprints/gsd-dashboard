import { useState } from "react";
import { ClipboardCopy, ExternalLink, FolderOpen } from "lucide-react";
import { Link, useParams } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";

import { OverviewTab } from "../components/ProjectDetail/OverviewTab";
import { ProjectSessionsTab } from "../components/ProjectDetail/ProjectSessionsTab";
import { copyNextCommand, openProjectInFinder, openProjectInVsCode } from "../lib/actions";
import { getProject, getProjectMilestones, getProjectPhasePanel } from "../lib/ipc";
import { projectMilestonesQueryKey, projectPhasePanelQueryKey, projectQueryKey } from "../lib/queryClient";
import "./ProjectDetailPage.css";

type ProjectDetailTab = "overview" | "sessions" | "charts";

const detailTabs: Array<{ id: ProjectDetailTab; label: string }> = [
  { id: "overview", label: "Overview" },
  { id: "sessions", label: "Sessions" },
  { id: "charts", label: "Charts" }
];

export function ProjectDetailPage() {
  const { id } = useParams<{ id: string }>();
  const [actionError, setActionError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<ProjectDetailTab>("overview");
  const project = useQuery({
    queryKey: projectQueryKey(id ?? ""),
    queryFn: () => getProject(id ?? ""),
    enabled: Boolean(id)
  });
  const milestones = useQuery({
    queryKey: projectMilestonesQueryKey(id ?? ""),
    queryFn: () => getProjectMilestones(id ?? ""),
    enabled: Boolean(id)
  });
  const phasePanel = useQuery({
    queryKey: projectPhasePanelQueryKey(id ?? ""),
    queryFn: () => getProjectPhasePanel(id ?? ""),
    enabled: Boolean(id)
  });

  async function runAction(action: () => Promise<void>) {
    setActionError(null);
    try {
      await action();
    } catch {
      setActionError("Action failed. Check the configured project path and try again.");
    }
  }

  function selectTabByOffset(offset: number) {
    const currentIndex = detailTabs.findIndex((tab) => tab.id === activeTab);
    const nextIndex = (currentIndex + offset + detailTabs.length) % detailTabs.length;
    setActiveTab(detailTabs[nextIndex].id);
  }

  function handleTabKeyDown(event: React.KeyboardEvent<HTMLButtonElement>) {
    if (event.key === "ArrowLeft") {
      event.preventDefault();
      selectTabByOffset(-1);
    } else if (event.key === "ArrowRight") {
      event.preventDefault();
      selectTabByOffset(1);
    } else if (event.key === "Home") {
      event.preventDefault();
      setActiveTab(detailTabs[0].id);
    } else if (event.key === "End") {
      event.preventDefault();
      setActiveTab(detailTabs[detailTabs.length - 1].id);
    }
  }

  if (!id) {
    return (
      <section className="settings-panel">
        <h1>Project not found</h1>
        <Link to="/">Back to Portfolio</Link>
      </section>
    );
  }

  if (project.isLoading) {
    return <section className="settings-panel">Loading project</section>;
  }

  if (project.isError) {
    return (
      <section className="settings-panel" role="alert">
        <h1>Project failed to load</h1>
        <p>Check the project cache and try again.</p>
        <Link to="/">Back to Portfolio</Link>
      </section>
    );
  }

  if (!project.data) {
    return (
      <section className="settings-panel">
        <h1>Project not found</h1>
        <Link to="/">Back to Portfolio</Link>
      </section>
    );
  }

  const phaseLabel =
    project.data.currentPhaseNumber && project.data.currentPhaseName
      ? `Phase ${project.data.currentPhaseNumber}: ${project.data.currentPhaseName}`
      : "Phase not available";
  const progressPct = Math.max(0, Math.min(100, Math.round(project.data.milestoneProgressPct)));

  return (
    <div className="page-stack">
      <Link className="back-link" to="/">
        Portfolio
      </Link>
      <section className="detail-panel">
        <div className="detail-header">
          <div>
            <h1>{project.data.name}</h1>
            <p>{project.data.rootPath}</p>
          </div>
          <div className="detail-actions">
            <button
              type="button"
              onClick={() => runAction(() => openProjectInFinder(project.data.rootPath))}
            >
              <FolderOpen aria-hidden="true" size={16} strokeWidth={2} />
              Open in Finder
            </button>
            <button
              type="button"
              onClick={() => runAction(() => openProjectInVsCode(project.data.rootPath))}
            >
              <ExternalLink aria-hidden="true" size={16} strokeWidth={2} />
              Open in VS Code
            </button>
            <button
              type="button"
              onClick={() => runAction(() => copyNextCommand(project.data.nextCommand))}
            >
              <ClipboardCopy aria-hidden="true" size={16} strokeWidth={2} />
              Copy next command
            </button>
          </div>
        </div>

        {actionError ? (
          <div className="parse-error-alert" role="alert">
            <p>{actionError}</p>
          </div>
        ) : null}

        <div className="detail-summary">
          <div>
            <p className="label-text">Current milestone</p>
            <h2>{project.data.currentMilestoneName ?? "Milestone not available"}</h2>
          </div>
          <div>
            <p className="label-text">Current phase</p>
            <h2>{phaseLabel}</h2>
          </div>
          <div>
            <p className="label-text">Progress</p>
            <div className="milestone-progress-row">
              <div className="scan-progress-track" aria-hidden="true">
                <div
                  className="scan-progress-fill"
                  style={{ width: `${progressPct}%` }}
                />
              </div>
              <span>{progressPct}%</span>
            </div>
          </div>
        </div>

        <div className="tab-nav" role="tablist" aria-label="Project detail sections">
          {detailTabs.map((tab) => {
            const selected = activeTab === tab.id;
            return (
              <button
                key={tab.id}
                type="button"
                id={`project-tab-${tab.id}-tab`}
                className="tab-btn"
                role="tab"
                aria-selected={selected}
                aria-controls={`project-tab-${tab.id}`}
                tabIndex={selected ? 0 : -1}
                onClick={() => setActiveTab(tab.id)}
                onKeyDown={handleTabKeyDown}
              >
                {tab.label}
              </button>
            );
          })}
        </div>

        {detailTabs.map((tab) => (
          <section
            key={tab.id}
            id={`project-tab-${tab.id}`}
            className="tab-panel"
            role="tabpanel"
            aria-labelledby={`project-tab-${tab.id}-tab`}
            hidden={activeTab !== tab.id}
          >
            {tab.id === "overview" ? (
              <OverviewTab
                milestones={milestones.data ?? []}
                phasePanel={phasePanel.data ?? null}
                loading={milestones.isLoading || phasePanel.isLoading}
                error={milestones.isError || phasePanel.isError}
              />
            ) : null}
            {tab.id === "sessions" ? <ProjectSessionsTab projectId={id} /> : null}
            {tab.id === "charts" ? <div className="chart-card">Charts</div> : null}
          </section>
        ))}
      </section>
    </div>
  );
}
