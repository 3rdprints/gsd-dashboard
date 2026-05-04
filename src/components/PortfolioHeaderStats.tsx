import type { PortfolioStats } from "../lib/types";

type PortfolioHeaderStatsProps = {
  stats: PortfolioStats;
};

/**
 * Provides the exported portfolio header stats function.
 */
export function PortfolioHeaderStats({ stats }: PortfolioHeaderStatsProps) {
  return (
    <dl className="portfolio-stats" aria-label="Portfolio stats">
      <StatCell label="Projects tracked" value={stats.projectsTracked} />
      <StatCell label="Active milestones" value={stats.activeMilestones} />
      <StatCell label="Sessions today" value={stats.sessionsToday} />
      <StatCell label="Tokens today" value={stats.tokensToday} compact />
    </dl>
  );
}

function StatCell({ compact = false, label, value }: { compact?: boolean; label: string; value: number }) {
  const displayValue = compact
    ? compactNumberFormatter.format(value)
    : numberFormatter.format(value);

  return (
    <div className="stat-cell">
      <dt>{label}</dt>
      <dd title={numberFormatter.format(value)}>{displayValue}</dd>
    </div>
  );
}

const numberFormatter = new Intl.NumberFormat();
const compactNumberFormatter = new Intl.NumberFormat(undefined, {
  compactDisplay: "short",
  maximumFractionDigits: 1,
  notation: "compact"
});
