import { ChevronDown, ChevronRight } from "lucide-react";
import { useState } from "react";

import type { ProjectMilestone } from "../../lib/types";
import { Button } from "../ui/button";

type MilestoneTimelineProps = {
  milestones: ProjectMilestone[];
};

export function MilestoneTimeline({ milestones }: MilestoneTimelineProps) {
  const [expanded, setExpanded] = useState<Set<number>>(() => {
    const activeIndex = milestones.findIndex((milestone) =>
      milestone.phases.some((phase) => phase.isCurrent)
    );
    return new Set([activeIndex >= 0 ? activeIndex : 0]);
  });

  if (milestones.length === 0) {
    return (
      <section className="chart-card">
        <h2 className="chart-card-title">Milestone Timeline</h2>
        <p className="muted-copy">No milestone data available.</p>
      </section>
    );
  }

  function toggle(index: number) {
    setExpanded((current) => {
      const next = new Set(current);
      if (next.has(index)) {
        next.delete(index);
      } else {
        next.add(index);
      }
      return next;
    });
  }

  return (
    <section className="chart-card">
      <div className="chart-card-header">
        <div>
          <h2 className="chart-card-title">Milestone Timeline</h2>
          <p className="chart-card-subtitle">Progress by phase</p>
        </div>
      </div>
      <div className="timeline-accordion">
        {milestones.map((milestone, index) => {
          const milestoneName = milestone.name ?? "Milestone not available";
          const progressPct = Math.round(Math.max(0, Math.min(100, milestone.progressPct)));
          const expandedId = `milestone-${index}-phases`;
          const isExpanded = expanded.has(index);

          return (
            <div key={`${milestoneName}-${index}`}>
              <Button
                type="button"
                className="timeline-milestone-row"
                aria-expanded={isExpanded}
                aria-controls={expandedId}
                onClick={() => toggle(index)}
                variant="ghost"
              >
                <span>{milestoneName} - {progressPct}%</span>
                <span className="timeline-track" aria-hidden="true">
                  {milestone.phases.map((phase) => (
                    <span
                      key={phase.number}
                      className={`timeline-segment ${segmentClass(phase)}`}
                    />
                  ))}
                </span>
                <span>{progressPct}%</span>
                {isExpanded ? (
                  <ChevronDown aria-hidden="true" size={16} strokeWidth={2} />
                ) : (
                  <ChevronRight aria-hidden="true" size={16} strokeWidth={2} />
                )}
              </Button>
              {isExpanded ? (
                <ul id={expandedId} className="timeline-phase-list">
                  {milestone.phases.map((phase) => (
                    <li key={phase.number}>
                      Phase {phase.number}: {phase.name ?? "Phase not available"}
                    </li>
                  ))}
                </ul>
              ) : null}
            </div>
          );
        })}
      </div>
    </section>
  );
}

function segmentClass(phase: ProjectMilestone["phases"][number]) {
  if (phase.completedAt !== null || (phase.totalPlanCount > 0 && phase.completedPlanCount >= phase.totalPlanCount)) {
    return "completed";
  }
  if (phase.isCurrent) {
    return "current";
  }
  return "future";
}
