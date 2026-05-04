import type { PortfolioProjectCard } from "../../lib/types";
import type { SessionFilters } from "../../lib/sessionFilters";
import { Button } from "../ui/button";
import { X } from "lucide-react";

type FilterChipsRowProps = {
  filters: SessionFilters;
  projects: PortfolioProjectCard[];
  onChange: (filters: SessionFilters) => void;
  onClearAll: () => void;
};

type Chip = {
  key: string;
  label: string;
  remove: () => SessionFilters;
};

/**
 * Provides the exported filter chips row function.
 */
export function FilterChipsRow({ filters, projects, onChange, onClearAll }: FilterChipsRowProps) {
  const chips = buildChips(filters, projects);
  if (chips.length === 0) return null;

  return (
    <div className="filter-chip-row" aria-label="Active filters">
      {chips.map((chip) => (
        <span key={chip.key} className="filter-chip">
          {chip.label}
          {chip.key === "source" ? (
            <Button
              type="button"
              className="size-5 rounded-md text-muted-foreground hover:text-foreground"
              aria-label="Remove source filter"
              onClick={() => onChange(chip.remove())}
              size="icon-xs"
              variant="ghost"
            >
              <X aria-hidden="true" size={12} strokeWidth={2} />
            </Button>
          ) : (
            <Button
              type="button"
              className="size-5 rounded-md text-muted-foreground hover:text-foreground"
              aria-label={`Remove ${chip.key} filter`}
              onClick={() => onChange(chip.remove())}
              size="icon-xs"
              variant="ghost"
            >
              <X aria-hidden="true" size={12} strokeWidth={2} />
            </Button>
          )}
        </span>
      ))}
      <Button type="button" onClick={onClearAll} size="sm" variant="destructive">
        Clear all
      </Button>
    </div>
  );
}

function buildChips(filters: SessionFilters, projects: PortfolioProjectCard[]): Chip[] {
  const chips: Chip[] = [];
  if (filters.source) {
    chips.push({
      key: "source",
      label: `Source: ${filters.source === "claude" ? "Claude" : "Codex"}`,
      remove: () => ({ ...filters, source: undefined, page: 1 })
    });
  }
  if (filters.projectId) {
    const project = projects.find((candidate) => candidate.id === filters.projectId);
    chips.push({
      key: "project",
      label: `Project: ${project?.name ?? filters.projectId}`,
      remove: () => ({ ...filters, projectId: undefined, page: 1 })
    });
  }
  if (filters.durationMinMinutes !== undefined || filters.durationMaxMinutes !== undefined) {
    chips.push({
      key: "duration",
      label: `Duration: ${rangeLabel(filters.durationMinMinutes, filters.durationMaxMinutes, "m")}`,
      remove: () => ({ ...filters, durationMinMinutes: undefined, durationMaxMinutes: undefined, page: 1 })
    });
  }
  if (filters.tokensMin !== undefined || filters.tokensMax !== undefined) {
    chips.push({
      key: "tokens",
      label: `Tokens: ${rangeLabel(filters.tokensMin, filters.tokensMax, "")}`,
      remove: () => ({ ...filters, tokensMin: undefined, tokensMax: undefined, page: 1 })
    });
  }
  if (filters.unmatchedOnly) {
    chips.push({
      key: "unmatched",
      label: "Unmatched only",
      remove: () => ({ ...filters, unmatchedOnly: false, page: 1 })
    });
  }
  return chips;
}

function rangeLabel(min: number | undefined, max: number | undefined, suffix: string) {
  if (min !== undefined && max !== undefined) return `${min}${suffix}-${max}${suffix}`;
  if (min !== undefined) return `>= ${min}${suffix}`;
  return `<= ${max}${suffix}`;
}
