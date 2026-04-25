import { useState } from "react";
import { ChevronDown, ChevronRight } from "lucide-react";

import type { HiddenProject, UnmatchedSessions } from "../lib/types";

type RightRailProps = {
  hiddenProjects: HiddenProject[];
  unmatchedSessions: UnmatchedSessions;
};

export function RightRail({ hiddenProjects, unmatchedSessions }: RightRailProps) {
  return (
    <aside className="right-rail" aria-label="Portfolio side panel">
      <RailSection title="Hidden projects">
        {hiddenProjects.length > 0 ? (
          <ul className="rail-list">
            {hiddenProjects.map((project) => (
              <li key={project.id}>
                <span>{project.name}</span>
                <span>{project.rootPath}</span>
              </li>
            ))}
          </ul>
        ) : (
          <p>No hidden projects</p>
        )}
      </RailSection>

      <RailSection title="Unmatched sessions">
        <p>{unmatchedSessions.label}</p>
        <p>{unmatchedSessions.count} unmatched</p>
      </RailSection>
    </aside>
  );
}

function RailSection({ title, children }: { title: string; children: React.ReactNode }) {
  const [open, setOpen] = useState(true);

  return (
    <section className="rail-section">
      <button
        className="rail-toggle"
        type="button"
        onClick={() => setOpen((current) => !current)}
        aria-expanded={open}
      >
        {open ? (
          <ChevronDown aria-hidden="true" size={16} strokeWidth={2} />
        ) : (
          <ChevronRight aria-hidden="true" size={16} strokeWidth={2} />
        )}
        {title}
      </button>
      {open ? <div className="rail-section-body">{children}</div> : null}
    </section>
  );
}
