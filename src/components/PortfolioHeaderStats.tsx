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
  return (
    <div className="stat-cell">
      <dt>{label}</dt>
      <dd>{value.toLocaleString()}</dd>
    </div>
  );
}
