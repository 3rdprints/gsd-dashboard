import { ArrowDown, ArrowUp, ArrowUpDown } from "lucide-react";

import type { ProjectSessionRow, ProjectSessionSortKey, SortDirection } from "../../lib/types";

type Column = {
  key: ProjectSessionSortKey;
  label: string;
  numeric?: boolean;
};

const columns: Column[] = [
  { key: "startedAt", label: "Date" },
  { key: "source", label: "Source" },
  { key: "durationMs", label: "Duration", numeric: true },
  { key: "messageCount", label: "Messages", numeric: true },
  { key: "tokensIn", label: "Tokens In", numeric: true },
  { key: "tokensOut", label: "Tokens Out", numeric: true }
];

export type SessionsTableProps = {
  rows: ProjectSessionRow[];
  total: number;
  page: number;
  pageSize: number;
  sort: ProjectSessionSortKey;
  direction: SortDirection;
  showProject?: boolean;
  onSortChange: (sort: ProjectSessionSortKey, direction: SortDirection) => void;
  onPageChange: (page: number) => void;
};

export function SessionsTable({
  rows,
  total,
  page,
  pageSize,
  sort,
  direction,
  showProject = false,
  onSortChange,
  onPageChange
}: SessionsTableProps) {
  const pageCount = Math.max(1, Math.ceil(total / pageSize));
  const canGoPrevious = page > 1;
  const canGoNext = page < pageCount;
  const visibleColumnCount = columns.length + (showProject ? 1 : 0);

  function nextDirection(column: ProjectSessionSortKey): SortDirection {
    if (column !== sort) {
      return "desc";
    }
    return direction === "desc" ? "asc" : "desc";
  }

  return (
    <>
      <div className="sessions-table-wrapper">
        <table className="sessions-table">
          <thead>
            <tr>
              {columns.map((column) => {
                const active = sort === column.key;
                const ariaSort = active ? (direction === "asc" ? "ascending" : "descending") : "none";
                const SortIcon = active ? (direction === "asc" ? ArrowUp : ArrowDown) : ArrowUpDown;
                return (
                  <th key={column.key} aria-sort={ariaSort} className={column.numeric ? "numeric-cell" : undefined}>
                    <button
                      type="button"
                      className="sortable"
                      onClick={() => onSortChange(column.key, nextDirection(column.key))}
                    >
                      {column.label}
                      <SortIcon aria-hidden="true" size={14} strokeWidth={2} />
                    </button>
                  </th>
                );
              })}
              {showProject ? <th>Project</th> : null}
            </tr>
          </thead>
          <tbody>
            {rows.length === 0 ? (
              <tr>
                <td colSpan={visibleColumnCount} className="empty-table-cell">
                  No sessions match the current filters.
                </td>
              </tr>
            ) : (
              rows.map((row) => (
                <tr key={row.id}>
                  <td>{formatDate(row.startedAt)}</td>
                  <td>
                    <span className={`source-badge ${row.source}`}>{formatSource(row.source)}</span>
                  </td>
                  <td className="numeric-cell">{formatDuration(row.durationMs)}</td>
                  <td className="numeric-cell">{formatNumber(row.messageCount)}</td>
                  <td className="numeric-cell">{formatNumber(row.tokensIn)}</td>
                  <td className="numeric-cell">{formatNumber(row.tokensOut)}</td>
                  {showProject ? <td>{row.projectName ?? "Unmatched"}</td> : null}
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
      <div className="pagination-row">
        <button
          type="button"
          disabled={!canGoPrevious}
          aria-label="Previous page"
          onClick={() => onPageChange(page - 1)}
        >
          Previous
        </button>
        <span>
          Page {page} of {pageCount}
        </span>
        <button type="button" disabled={!canGoNext} aria-label="Next page" onClick={() => onPageChange(page + 1)}>
          Next
        </button>
      </div>
    </>
  );
}

function formatSource(source: ProjectSessionRow["source"]): string {
  return source === "claude" ? "Claude" : "Codex";
}

function formatDate(value: number | null): string {
  if (value === null) {
    return "Unknown";
  }
  return new Intl.DateTimeFormat(undefined, {
    month: "short",
    day: "numeric",
    hour: "numeric",
    minute: "2-digit"
  }).format(new Date(value));
}

function formatDuration(value: number | null): string {
  if (value === null) {
    return "Unknown";
  }
  const totalMinutes = Math.max(0, Math.round(value / 60_000));
  if (totalMinutes < 60) {
    return `${totalMinutes}m`;
  }
  const hours = Math.floor(totalMinutes / 60);
  const minutes = totalMinutes % 60;
  return minutes === 0 ? `${hours}h` : `${hours}h ${minutes}m`;
}

function formatNumber(value: number): string {
  return new Intl.NumberFormat().format(value);
}
