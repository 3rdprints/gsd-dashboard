import type { PortfolioStats } from "../lib/types";

type PortfolioHeaderStatsProps = {
  stats: PortfolioStats;
};

export function PortfolioHeaderStats({ stats }: PortfolioHeaderStatsProps) {
  return (
    <dl className="portfolio-stats" aria-label="Portfolio stats">
      <StatCell label="Projects tracked" value={stats.projectsTracked} />
      <StatCell label="Active milestones" value={stats.activeMilestones} />
      <StatCell label="Sessions today" value={stats.sessionsToday} />
      <StatCell label="Tokens today" value={stats.tokensToday} />
    </dl>
  );
}

function StatCell({ label, value }: { label: string; value: number }) {
  const displayValue = label === "Tokens today" ? formatCompactNumber(value) : value.toLocaleString();

  return (
    <div className="stat-cell">
      <dt>{label}</dt>
      <dd title={value.toLocaleString()}>{displayValue}</dd>
    </div>
  );
}

function formatCompactNumber(value: number) {
  if (Math.abs(value) < 1_000) {
    return value.toLocaleString();
  }

  return `${(value / 1_000).toFixed(1).replace(/\.0$/, "")}k`;
}
