import { useState } from "react";
import { ClipboardCopy, ExternalLink, FolderOpen } from "lucide-react";
import { Link, useParams } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";

import { copyNextCommand, openProjectInFinder, openProjectInVsCode } from "../lib/actions";
import { getProject } from "../lib/ipc";
import { projectQueryKey } from "../lib/queryClient";

export function ProjectDetailPage() {
  const { id } = useParams<{ id: string }>();
  const [actionError, setActionError] = useState<string | null>(null);
  const project = useQuery({
    queryKey: projectQueryKey(id ?? ""),
    queryFn: () => getProject(id ?? ""),
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
      </section>
    </div>
  );
}
