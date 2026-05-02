import { MouseEvent, useState } from "react";
import { flushSync } from "react-dom";
import { ClipboardCopy, EyeOff } from "lucide-react";
import { Link } from "react-router-dom";
import { Bar, BarChart, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";

import { Button } from "./ui/button";
import { copyNextCommand } from "../lib/actions";
import type { PortfolioProjectCard } from "../lib/types";
import "./ProjectCard.css";

type ProjectCardProps = {
  project: PortfolioProjectCard;
  onHideProject: (projectId: string) => Promise<void> | void;
  hideDisabled?: boolean;
};

export function ProjectCard({ project, onHideProject, hideDisabled = false }: ProjectCardProps) {
  const [copied, setCopied] = useState(false);
  const [hideError, setHideError] = useState(false);
  const phaseLabel =
    project.currentPhaseNumber && project.currentPhaseName
      ? `Phase ${project.currentPhaseNumber}: ${project.currentPhaseName}`
      : "Phase not available";
  const progressPct = Math.max(0, Math.min(100, Math.round(project.milestoneProgressPct)));

  async function handleCopy(event: MouseEvent<HTMLButtonElement>) {
    event.preventDefault();
    event.stopPropagation();

    try {
      await copyNextCommand(project.nextCommand);
      flushSync(() => setCopied(true));
      window.setTimeout(() => setCopied(false), 1600);
    } catch {
      setCopied(false);
    }
  }

  async function handleHide(event: MouseEvent<HTMLButtonElement>) {
    event.preventDefault();
    event.stopPropagation();
    setHideError(false);
    try {
      await onHideProject(project.id);
    } catch {
      setHideError(true);
    }
  }

  return (
    <article className="project-card">
      <Link
        className="project-card-link"
        to={`/project/${project.id}`}
        aria-label={project.name}
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
              style={{ width: `${progressPct}%` }}
            />
          </div>
          <span>{progressPct}%</span>
        </div>

        <div className="project-module-strip" aria-label={`${project.name} metrics`}>
          <MetricModule label="Milestone %" value={progressPct.toString()} />
          <MetricModule label="Sessions 7d" value={project.sessionsLast7d.toString()} />
          <MetricModule
            label="Activity"
            value={formatRelativeActivity(project.lastActivityAt ?? project.lastScannedAt)}
          />
        </div>

        <SessionSparkline project={project} />

        <div className="project-card-meta">
          <span>{phaseLabel}</span>
          <span>{formatRelativeActivity(project.lastActivityAt ?? project.lastScannedAt)}</span>
        </div>
      </Link>

      <Button className="card-copy-action" type="button" onClick={handleCopy} variant="outline">
        <ClipboardCopy aria-hidden="true" size={16} strokeWidth={2} />
        {copied ? "Copied" : "Copy next command"}
      </Button>
      <Button type="button" onClick={handleHide} disabled={hideDisabled} variant="outline">
        <EyeOff aria-hidden="true" size={16} strokeWidth={2} />
        Hide Project
      </Button>
      {hideError ? (
        <p className="project-card-error" role="alert">
          Could not hide project
        </p>
      ) : null}
    </article>
  );
}

function MetricModule({ label, value }: { label: string; value: string }) {
  return (
    <div className="project-module">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function SessionSparkline({ project }: { project: PortfolioProjectCard }) {
  const maxCount = Math.max(1, ...project.sessionSparkline7d.map((day) => day.count));
  const accessibleText = `${project.sessionsLast7d} sessions in the last 7 days`;
  const sparklineData = project.sessionSparkline7d.map((day) => ({
    date: day.date,
    count: day.count
  }));

  return (
    <div className="session-sparkline-row">
      <div>
        <p className="session-sparkline-label">
          {project.sessionsLast7d > 0 ? "7d sessions" : "No sessions in 7d"}
        </p>
        <p className="sr-only">{accessibleText}</p>
      </div>
      <div className="session-sparkline" aria-label={accessibleText}>
        <ResponsiveContainer width="100%" height="100%" minWidth={1} minHeight={1}>
          <BarChart data={sparklineData} margin={{ top: 0, right: 0, bottom: 0, left: 0 }}>
            <XAxis dataKey="date" hide />
            <YAxis hide domain={[0, maxCount]} />
            <Tooltip cursor={false} content={() => null} />
            <Bar
              dataKey="count"
              fill="#2563EB"
              isAnimationActive={false}
              minPointSize={4}
              radius={[2, 2, 0, 0]}
            />
          </BarChart>
        </ResponsiveContainer>
      </div>
    </div>
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
