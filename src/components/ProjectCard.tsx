import { MouseEvent, useState } from "react";
import { ClipboardCopy } from "lucide-react";
import { Link, useNavigate } from "react-router-dom";

import { copyNextCommand } from "../lib/actions";
import type { PortfolioProjectCard } from "../lib/types";

type ProjectCardProps = {
  project: PortfolioProjectCard;
};

export function ProjectCard({ project }: ProjectCardProps) {
  const navigate = useNavigate();
  const [copied, setCopied] = useState(false);
  const phaseLabel =
    project.currentPhaseNumber && project.currentPhaseName
      ? `Phase ${project.currentPhaseNumber}: ${project.currentPhaseName}`
      : "Phase not available";

  async function handleCopy(event: MouseEvent<HTMLButtonElement>) {
    event.preventDefault();
    event.stopPropagation();
    setCopied(true);
    await copyNextCommand(project.nextCommand);
    window.setTimeout(() => setCopied(false), 1600);
  }

  return (
    <article className="project-card">
      <Link
        className="project-card-link"
        to={`/project/${project.id}`}
        aria-label={project.name}
        onClick={(event) => {
          event.preventDefault();
          navigate(`/project/${project.id}`);
        }}
      >
        <div className="project-card-header">
          <div>
            <h2 title={project.name}>{project.name}</h2>
            <p>{project.currentMilestoneName ?? "Milestone not available"}</p>
          </div>
          {project.parseError ? <span className="parse-badge">Parse error</span> : null}
        </div>

        <div className="milestone-progress-row">
          <div className="scan-progress-track" aria-hidden="true">
            <div
              className="scan-progress-fill"
              style={{ width: `${Math.round(project.milestoneProgressPct)}%` }}
            />
          </div>
          <span>{Math.round(project.milestoneProgressPct)}%</span>
        </div>

        <div className="project-card-meta">
          <span>{phaseLabel}</span>
          <span>{formatRelativeActivity(project.lastActivityAt ?? project.lastScannedAt)}</span>
        </div>
      </Link>

      <button className="card-copy-action" type="button" onClick={handleCopy}>
        <ClipboardCopy aria-hidden="true" size={16} strokeWidth={2} />
        {copied ? "Copied" : "Copy next command"}
      </button>
    </article>
  );
}

function formatRelativeActivity(timestampSeconds: number | null) {
  if (!timestampSeconds) {
    return "No activity yet";
  }

  const elapsedSeconds = Math.max(0, Math.floor(Date.now() / 1000) - timestampSeconds);
  const elapsedMinutes = Math.floor(elapsedSeconds / 60);
  const elapsedHours = Math.floor(elapsedMinutes / 60);
  const elapsedDays = Math.floor(elapsedHours / 24);

  if (elapsedDays > 0) return `${elapsedDays}d ago`;
  if (elapsedHours > 0) return `${elapsedHours}h ago`;
  if (elapsedMinutes > 0) return `${elapsedMinutes}m ago`;
  return "Just now";
}
