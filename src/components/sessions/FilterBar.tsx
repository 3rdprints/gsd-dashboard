import { useEffect, useRef, useState } from "react";

import type { PortfolioProjectCard } from "../../lib/types";
import { applyDateRange, type SessionFilters } from "../../lib/sessionFilters";
import { Checkbox } from "../ui/checkbox";
import { Input } from "../ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../ui/select";

type FilterBarProps = {
  filters: SessionFilters;
  projects: PortfolioProjectCard[];
  onChange: (filters: SessionFilters) => void;
  onDateRangePersist: (range: "7d" | "30d" | "90d" | "all") => void;
};

const ALL_VALUE = "__all__";

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
      const parsed = value === "" ? NaN : Number(value);
      update({ [key]: Number.isFinite(parsed) && parsed >= 0 ? parsed : undefined });
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
        <Select value={filters.source ?? ALL_VALUE} onValueChange={(value) => update({ source: sourceValue(value) })}>
          <SelectTrigger aria-label="Source">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value={ALL_VALUE}>All</SelectItem>
            <SelectItem value="claude">Claude</SelectItem>
            <SelectItem value="codex">Codex</SelectItem>
          </SelectContent>
        </Select>
      </label>
      <label className="filter-control">
        <span className="field-label">Project</span>
        <Select
          value={filters.projectId ?? ALL_VALUE}
          disabled={filters.unmatchedOnly}
          onValueChange={(value) => update({ projectId: value === ALL_VALUE ? undefined : value })}
        >
          <SelectTrigger aria-label="Project">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value={ALL_VALUE}>All</SelectItem>
            {projects.map((project) => (
              <SelectItem key={project.id} value={project.id}>
                {project.name}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </label>
      <label className="filter-control">
        <span className="field-label">Date range</span>
        <Select value={filters.dateRange} onValueChange={(value) => updateDateRange(value as SessionFilters["dateRange"])}>
          <SelectTrigger aria-label="Date range">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="today">Today</SelectItem>
            <SelectItem value="7d">Last 7 days</SelectItem>
            <SelectItem value="30d">Last 30 days</SelectItem>
            <SelectItem value="90d">Last 90 days</SelectItem>
            <SelectItem value="all">All</SelectItem>
            <SelectItem value="custom">Custom</SelectItem>
          </SelectContent>
        </Select>
      </label>
      {filters.dateRange === "custom" ? (
        <>
          <label className="filter-control compact">
            <span className="field-label">From</span>
            <Input type="date" value={filters.from ?? ""} onChange={(event) => update({ from: event.target.value || undefined })} />
          </label>
          <label className="filter-control compact">
            <span className="field-label">To</span>
            <Input type="date" value={filters.to ?? ""} onChange={(event) => update({ to: event.target.value || undefined })} />
          </label>
        </>
      ) : null}
      <label className="filter-control compact">
        <span className="field-label">Minimum duration</span>
        <Input
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
        <Input
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
        <Input
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
        <Input
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
        <Checkbox
          checked={filters.unmatchedOnly}
          onCheckedChange={(checked) => update({ unmatchedOnly: Boolean(checked), projectId: checked ? undefined : filters.projectId })}
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
