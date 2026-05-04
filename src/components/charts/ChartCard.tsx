import type { ReactNode } from "react";

export type ChartCardProps = {
  title: string;
  subtitle: string;
  loading?: boolean;
  empty?: boolean;
  children: ReactNode;
};

/**
 * Provides the exported chart card function.
 */
export function ChartCard({ title, subtitle, loading = false, empty = false, children }: ChartCardProps) {
  return (
    <section className="chart-card" aria-label={`${title} chart`}>
      <div className="chart-card-header">
        <div>
          <h2 className="chart-card-title">{title}</h2>
          <p className="chart-card-subtitle">{subtitle}</p>
        </div>
      </div>
      <div className="chart-card-body" aria-label={loading ? `Loading ${title}` : undefined}>
        {loading ? <div className="chart-skeleton" /> : null}
        {!loading && empty ? <div className="chart-empty-state">No data for this period.</div> : null}
        {!loading && !empty ? children : null}
      </div>
    </section>
  );
}
