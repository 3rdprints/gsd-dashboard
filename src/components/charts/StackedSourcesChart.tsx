import { Bar, BarChart, CartesianGrid, Legend, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";

import type { GlobalSessionsBySourceDay } from "../../lib/types";
import { ChartTooltip } from "./ChartTooltip";

const axisStyle = { fill: "#6B7280", fontSize: 12 };
const chartMargin = { top: 8, right: 12, bottom: 0, left: 0 };

export type StackedSourcesChartProps = {
  data: GlobalSessionsBySourceDay[];
};

export function StackedSourcesChart({ data }: StackedSourcesChartProps) {
  return (
    <>
      <ResponsiveContainer width="100%" height={200}>
        <BarChart data={data} margin={chartMargin}>
          <CartesianGrid stroke="#E5E7EB" strokeDasharray="3 3" />
          <XAxis dataKey="date" tick={axisStyle} tickLine={false} />
          <YAxis tick={axisStyle} tickLine={false} width={44} />
          <Tooltip content={<ChartTooltip valueFormatter={formatNumber} />} />
          <Legend wrapperStyle={{ color: "#4B5563", fontSize: 12 }} />
          <Bar dataKey="claude" name="Claude" stackId="src" fill="#2563EB" fillOpacity={0.8} radius={[2, 2, 0, 0]} />
          <Bar dataKey="codex" name="Codex" stackId="src" fill="#7C3AED" fillOpacity={0.8} radius={[2, 2, 0, 0]} />
        </BarChart>
      </ResponsiveContainer>
      <div className="chart-legend-row" aria-label="Source chart legend">
        <span className="chart-legend-chip">Claude</span>
        <span className="chart-legend-chip">Codex</span>
      </div>
    </>
  );
}

function formatNumber(value: number | string): string {
  return typeof value === "number" ? new Intl.NumberFormat().format(value) : value;
}
