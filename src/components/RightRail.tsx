import { useState } from "react";
import { ChevronDown, ChevronRight } from "lucide-react";

import { Button } from "./ui/button";
import type { HiddenProject, UnmatchedSessions } from "../lib/types";

type RightRailProps = {
  hiddenProjects: HiddenProject[];
  unmatchedSessions: UnmatchedSessions;
};

export function RightRail({ hiddenProjects, unmatchedSessions }: RightRailProps) {
  return (
    <aside className="right-rail" aria-label="Portfolio side panel">
      <RailSection title="Hidden projects" defaultOpen>
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
        <p>{unmatchedSessions.count === 0 ? "No unmatched sessions" : unmatchedSessions.label}</p>
        {unmatchedSessions.count > 0 ? (
          <>
            <div className="unmatched-source-mix">
              <span>Claude Code {unmatchedSessions.claudeCount}</span>
              <span>Codex {unmatchedSessions.codexCount}</span>
            </div>
            <ul className="rail-list unmatched-session-list">
              {unmatchedSessions.recent.map((session) => (
                <li key={session.id}>
                  <span>{session.source === "claude" ? "Claude Code" : "Codex"}</span>
                  <span>{session.sourcePath}</span>
                </li>
              ))}
            </ul>
          </>
        ) : null}
      </RailSection>
    </aside>
  );
}

function RailSection({
  title,
  children,
  defaultOpen = false
}: {
  title: string;
  children: React.ReactNode;
  defaultOpen?: boolean;
}) {
  const [open, setOpen] = useState(defaultOpen);

  return (
    <section className="rail-section">
      <Button
        className="rail-toggle"
        type="button"
        onClick={() => setOpen((current) => !current)}
        aria-expanded={open}
        variant="ghost"
      >
        {open ? (
          <ChevronDown aria-hidden="true" size={16} strokeWidth={2} />
        ) : (
          <ChevronRight aria-hidden="true" size={16} strokeWidth={2} />
        )}
        {title}
      </Button>
      {open ? <div className="rail-section-body">{children}</div> : null}
    </section>
  );
}
