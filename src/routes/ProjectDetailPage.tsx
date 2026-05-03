import { useState } from "react";
import { ClipboardCopy, ExternalLink, FolderOpen } from "lucide-react";
import { Link, useParams } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";

import { OverviewTab } from "../components/ProjectDetail/OverviewTab";
import { ProjectChartsTab } from "../components/ProjectDetail/ProjectChartsTab";
import { ProjectSessionsTab } from "../components/ProjectDetail/ProjectSessionsTab";
import { Button } from "../components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "../components/ui/tabs";
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
      <Button asChild className="back-link" variant="outline">
        <Link to="/">Portfolio</Link>
      </Button>
      <section className="detail-panel">
        <div className="detail-header">
          <div>
            <h1>{project.data.name}</h1>
            <p>{project.data.rootPath}</p>
          </div>
          <div className="detail-actions">
            <Button
              type="button"
              variant="outline"
              onClick={() => runAction(() => openProjectInFinder(project.data.rootPath))}
            >
              <FolderOpen aria-hidden="true" size={16} strokeWidth={2} />
              Open in Finder
            </Button>
            <Button
              type="button"
              variant="outline"
              onClick={() => runAction(() => openProjectInVsCode(project.data.rootPath))}
            >
              <ExternalLink aria-hidden="true" size={16} strokeWidth={2} />
              Open in VS Code
            </Button>
            <Button
              type="button"
              variant="outline"
              onClick={() => runAction(() => copyNextCommand(project.data.nextCommand))}
            >
              <ClipboardCopy aria-hidden="true" size={16} strokeWidth={2} />
              Copy next command
            </Button>
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

        <Tabs
          aria-label="Project detail sections"
          value={activeTab}
          onValueChange={(value) => setActiveTab(value as ProjectDetailTab)}
        >
          <TabsList aria-label="Project detail sections" variant="line">
            {detailTabs.map((tab) => (
              <TabsTrigger key={tab.id} value={tab.id}>
                {tab.label}
              </TabsTrigger>
            ))}
          </TabsList>

          <TabsContent value="overview" className="tab-panel">
            {activeTab === "overview" ? (
              <OverviewTab
                milestones={milestones.data ?? []}
                phasePanel={phasePanel.data ?? null}
                loading={milestones.isLoading || phasePanel.isLoading}
                error={milestones.isError || phasePanel.isError}
              />
            ) : null}
          </TabsContent>
          <TabsContent value="sessions" className="tab-panel">
            {activeTab === "sessions" ? <ProjectSessionsTab projectId={id} /> : null}
          </TabsContent>
          <TabsContent value="charts" className="tab-panel">
            {activeTab === "charts" ? <ProjectChartsTab projectId={id} /> : null}
          </TabsContent>
        </Tabs>
      </section>
    </div>
  );
}
