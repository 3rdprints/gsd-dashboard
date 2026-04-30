import { useState } from "react";
import { Link } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";

import { listProjectSessions } from "../../lib/ipc";
import { projectSessionsQueryKey } from "../../lib/queryClient";
import type { ProjectSessionSortKey, SortDirection } from "../../lib/types";
import { SessionsTable } from "../sessions/SessionsTable";

const pageSize = 50;

export type ProjectSessionsTabProps = {
  projectId: string;
};

export function ProjectSessionsTab({ projectId }: ProjectSessionsTabProps) {
  const [sort, setSort] = useState<ProjectSessionSortKey>("startedAt");
  const [direction, setDirection] = useState<SortDirection>("desc");
  const [page, setPage] = useState(1);
  const sessions = useQuery({
    queryKey: projectSessionsQueryKey(projectId, sort, direction, page, pageSize),
    queryFn: () => listProjectSessions(projectId, sort, direction, page, pageSize)
  });

  function handleSortChange(nextSort: ProjectSessionSortKey, nextDirection: SortDirection) {
    setSort(nextSort);
    setDirection(nextDirection);
    setPage(1);
  }

  if (sessions.isLoading) {
    return (
      <section className="chart-card">
        <div className="table-skeleton" aria-label="Loading sessions" />
      </section>
    );
  }

  if (sessions.isError) {
    return (
      <section className="chart-card" role="alert">
        <h2 className="chart-card-title">Sessions could not be loaded</h2>
        <p className="chart-card-subtitle">Rebuild the cache or re-index sessions and try again.</p>
      </section>
    );
  }

  const pageData = sessions.data ?? { rows: [], total: 0, page, pageSize };

  return (
    <section className="chart-card">
      <div className="chart-card-header">
        <div>
          <h2 className="chart-card-title">Sessions</h2>
          <p className="chart-card-subtitle">Sessions attributed to this project.</p>
        </div>
        <Link className="secondary-link" to={`/sessions?project=${encodeURIComponent(projectId)}`}>
          See all sessions
        </Link>
      </div>
      {pageData.total === 0 ? (
        <div className="empty-state">
          <h3>No sessions for this project</h3>
          <p>Sessions attributed to this project will appear here after indexing.</p>
        </div>
      ) : (
        <SessionsTable
          rows={pageData.rows}
          total={pageData.total}
          page={pageData.page}
          pageSize={pageData.pageSize}
          sort={sort}
          direction={direction}
          onSortChange={handleSortChange}
          onPageChange={setPage}
        />
      )}
    </section>
  );
}
