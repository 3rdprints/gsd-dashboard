import { useEffect, useRef, useState } from "react";

import type { PortfolioProjectCard } from "../../lib/types";
import { applyDateRange, type SessionFilters } from "../../lib/sessionFilters";

type FilterBarProps = {
  filters: SessionFilters;
  projects: PortfolioProjectCard[];
  onChange: (filters: SessionFilters) => void;
  onDateRangePersist: (range: "7d" | "30d" | "90d" | "all") => void;
};

export function FilterBar({ filters, projects, onChange, onDateRangePersist }: FilterBarProps) {
  const debounceRef = useRef<number | undefined>(undefined);
  const [durationMin, setDurationMin] = useState(toInputValue(filters.durationMinMinutes));
  const [durationMax, setDurationMax] = useState(toInputValue(filters.durationMaxMinutes));
  const [tokensMin, setTokensMin] = useState(toInputValue(filters.tokensMin));
  const [tokensMax, setTokensMax] = useState(toInputValue(filters.tokensMax));

  useEffect(() => {
    window.clearTimeout(debounceRef.current);
    setDurationMin(toInputValue(filters.durationMinMinutes));
    setDurationMax(toInputValue(filters.durationMaxMinutes));
    setTokensMin(toInputValue(filters.tokensMin));
    setTokensMax(toInputValue(filters.tokensMax));
  }, [filters.durationMinMinutes, filters.durationMaxMinutes, filters.tokensMin, filters.tokensMax]);

  useEffect(() => {
    return () => window.clearTimeout(debounceRef.current);
  }, []);

  function update(next: Partial<SessionFilters>) {
    onChange({ ...filters, ...next, page: 1 });
  }

  function updateNumber(key: keyof Pick<SessionFilters, "durationMinMinutes" | "durationMaxMinutes" | "tokensMin" | "tokensMax">, value: string) {
    window.clearTimeout(debounceRef.current);
    debounceRef.current = window.setTimeout(() => {
      const parsed = value === "" ? undefined : Number(value);
      update({ [key]: Number.isFinite(parsed) && parsed !== undefined && parsed >= 0 ? parsed : undefined });
    }, 300);
  }

  function updateDateRange(value: SessionFilters["dateRange"]) {
    onChange(applyDateRange(filters, value));
    if (value === "7d" || value === "30d" || value === "90d" || value === "all") {
      onDateRangePersist(value);
    }
  }

  return (
    <section className="filter-bar" aria-label="Filter sessions">
      <label className="filter-control">
        <span className="field-label">Source</span>
        <select value={filters.source ?? ""} onChange={(event) => update({ source: sourceValue(event.target.value) })}>
          <option value="">All</option>
          <option value="claude">Claude</option>
          <option value="codex">Codex</option>
        </select>
      </label>
      <label className="filter-control">
        <span className="field-label">Project</span>
        <select
          value={filters.projectId ?? ""}
          disabled={filters.unmatchedOnly}
          onChange={(event) => update({ projectId: event.target.value || undefined })}
        >
          <option value="">All</option>
          {projects.map((project) => (
            <option key={project.id} value={project.id}>
              {project.name}
            </option>
          ))}
        </select>
      </label>
      <label className="filter-control">
        <span className="field-label">Date range</span>
        <select value={filters.dateRange} onChange={(event) => updateDateRange(event.target.value as SessionFilters["dateRange"])}>
          <option value="today">Today</option>
          <option value="7d">Last 7 days</option>
          <option value="30d">Last 30 days</option>
          <option value="90d">Last 90 days</option>
          <option value="all">All</option>
          <option value="custom">Custom</option>
        </select>
      </label>
      {filters.dateRange === "custom" ? (
        <>
          <label className="filter-control compact">
            <span className="field-label">From</span>
            <input type="date" value={filters.from ?? ""} onChange={(event) => update({ from: event.target.value || undefined })} />
          </label>
          <label className="filter-control compact">
            <span className="field-label">To</span>
            <input type="date" value={filters.to ?? ""} onChange={(event) => update({ to: event.target.value || undefined })} />
          </label>
        </>
      ) : null}
      <label className="filter-control compact">
        <span className="field-label">Minimum duration</span>
        <input
          type="number"
          min="0"
          value={durationMin}
          onChange={(event) => {
            setDurationMin(event.target.value);
            updateNumber("durationMinMinutes", event.target.value);
          }}
        />
      </label>
      <label className="filter-control compact">
        <span className="field-label">Maximum duration</span>
        <input
          type="number"
          min="0"
          value={durationMax}
          onChange={(event) => {
            setDurationMax(event.target.value);
            updateNumber("durationMaxMinutes", event.target.value);
          }}
        />
      </label>
      <label className="filter-control compact">
        <span className="field-label">Minimum tokens</span>
        <input
          type="number"
          min="0"
          value={tokensMin}
          onChange={(event) => {
            setTokensMin(event.target.value);
            updateNumber("tokensMin", event.target.value);
          }}
        />
      </label>
      <label className="filter-control compact">
        <span className="field-label">Maximum tokens</span>
        <input
          type="number"
          min="0"
          value={tokensMax}
          onChange={(event) => {
            setTokensMax(event.target.value);
            updateNumber("tokensMax", event.target.value);
          }}
        />
      </label>
      <label className="unmatched-toggle">
        <input
          type="checkbox"
          checked={filters.unmatchedOnly}
          onChange={(event) => update({ unmatchedOnly: event.target.checked, projectId: event.target.checked ? undefined : filters.projectId })}
        />
        Unmatched only
      </label>
    </section>
  );
}

function sourceValue(value: string) {
  return value === "claude" || value === "codex" ? value : undefined;
}

function toInputValue(value: number | undefined) {
  return value === undefined ? "" : String(value);
}
