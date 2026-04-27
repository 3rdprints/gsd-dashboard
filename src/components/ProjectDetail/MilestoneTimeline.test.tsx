import "@testing-library/jest-dom/vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { MilestoneTimeline } from "./MilestoneTimeline";
import type { ProjectMilestone } from "../../lib/types";

describe("MilestoneTimeline", () => {
  const milestones: ProjectMilestone[] = [
    {
      name: "v1.0 MVP",
      progressPct: 55,
      phaseCount: 3,
      completedPhaseCount: 1,
      phases: [
        {
          number: "04",
          name: "Session Indexer",
          isCurrent: false,
          completedAt: 1_777_000_000,
          completedPlanCount: 4,
          totalPlanCount: 4
        },
        {
          number: "05",
          name: "Project Detail",
          isCurrent: true,
          completedAt: null,
          completedPlanCount: 2,
          totalPlanCount: 4
        },
        {
          number: "06",
          name: "Tray",
          isCurrent: false,
          completedAt: null,
          completedPlanCount: 0,
          totalPlanCount: 3
        }
      ]
    },
    {
      name: "v1.1 Polish",
      progressPct: 0,
      phaseCount: 1,
      completedPhaseCount: 0,
      phases: [
        {
          number: "07",
          name: "Live Updates",
          isCurrent: false,
          completedAt: null,
          completedPlanCount: 0,
          totalPlanCount: 2
        }
      ]
    }
  ];

  it("starts with the active milestone expanded", () => {
    render(<MilestoneTimeline milestones={milestones} />);

    expect(screen.getByRole("button", { name: /v1.0 MVP/ })).toHaveAttribute("aria-expanded", "true");
    expect(screen.getByText("Phase 05: Project Detail")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /v1.1 Polish/ })).toHaveAttribute("aria-expanded", "false");
    expect(screen.queryByText("Phase 07: Live Updates")).not.toBeInTheDocument();
  });

  it("toggles rows and renders one timeline segment per phase", () => {
    render(<MilestoneTimeline milestones={milestones} />);

    expect(document.querySelectorAll(".timeline-segment")).toHaveLength(4);
    fireEvent.click(screen.getByRole("button", { name: /v1.0 MVP/ }));
    expect(screen.queryByText("Phase 05: Project Detail")).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: /v1.1 Polish/ }));
    expect(screen.getByText("Phase 07: Live Updates")).toBeInTheDocument();
  });
});
