import { CheckSquare, Square } from "lucide-react";

import { openProjectInVsCode } from "../../lib/actions";
import type { ProjectMilestone, ProjectPhasePanel } from "../../lib/types";
import { Button } from "../ui/button";
import { MilestoneTimeline } from "./MilestoneTimeline";
import { StateExcerpt } from "./StateExcerpt";

type OverviewTabProps = {
  milestones: ProjectMilestone[];
  phasePanel: ProjectPhasePanel | null;
  loading: boolean;
  error: boolean;
};

export function OverviewTab({ milestones, phasePanel, loading, error }: OverviewTabProps) {
  if (loading) {
    return (
      <div className="overview-grid">
        <div className="chart-card">Loading overview</div>
        <div className="chart-card">Loading current position</div>
      </div>
    );
  }

  if (error || !phasePanel) {
    return (
      <div className="chart-card" role="alert">
        Project overview could not be loaded.
      </div>
    );
  }

  const phaseLabel =
    phasePanel.phaseNumber && phasePanel.phaseName
      ? `Phase ${phasePanel.phaseNumber}: ${phasePanel.phaseName}`
      : "Current Phase";

  return (
    <div className="overview-grid">
      <div className="overview-main-column">
        <MilestoneTimeline milestones={milestones} />
        <section className="chart-card">
          <div className="chart-card-header">
            <div>
              <h2 className="chart-card-title">Current Phase</h2>
              <p className="chart-card-subtitle">{phaseLabel}</p>
            </div>
            <span className="plan-count-badge">
              {phasePanel.completedItemCount} of {phasePanel.totalItemCount} plans complete
            </span>
          </div>
          <ul className="plan-checklist">
            {phasePanel.items.map((item) => (
              <li
                key={`${item.planPath}-${item.ord}`}
                className={`plan-item ${item.checked ? "checked" : ""}`}
              >
                {item.checked ? (
                  <CheckSquare className="plan-item-check" aria-hidden="true" size={16} strokeWidth={2} />
                ) : (
                  <Square className="plan-item-check" aria-hidden="true" size={16} strokeWidth={2} />
                )}
                <span>{item.text}</span>
              </li>
            ))}
          </ul>
          {phasePanel.planPath ? (
            <Button
              type="button"
              className="overview-path-action"
              variant="outline"
              onClick={() => void openProjectInVsCode(phasePanel.planPath ?? "")}
            >
              Open PLAN.md
            </Button>
          ) : null}
        </section>
      </div>
      <StateExcerpt statePath={phasePanel.statePath} excerpt={phasePanel.stateExcerpt} />
    </div>
  );
}
