import { Bar, BarChart, CartesianGrid, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";

import type { GlobalHistogramBucket } from "../../lib/types";
import { ChartTooltip } from "./ChartTooltip";

const axisStyle = { fill: "#6B7280", fontSize: 12 };
const chartMargin = { top: 8, right: 12, bottom: 0, left: 0 };
const hourTicks = [0, 4, 8, 12, 16, 20];

export type TimeOfDayHistogramProps = {
  data: GlobalHistogramBucket[];
};

/**
 * Provides the exported time of day histogram function.
 */
export function TimeOfDayHistogram({ data }: TimeOfDayHistogramProps) {
  const chartData = normalizeHours(data);

  return (
    <>
      <ResponsiveContainer width="100%" height={200}>
        <BarChart data={chartData} margin={chartMargin}>
          <CartesianGrid stroke="#E5E7EB" strokeDasharray="3 3" />
          <XAxis dataKey="hour" ticks={hourTicks} tick={axisStyle} tickFormatter={formatHourTick} tickLine={false} />
          <YAxis tick={axisStyle} tickLine={false} width={44} />
          <Tooltip content={<ChartTooltip valueFormatter={formatNumber} />} />
          <Bar dataKey="count" name="Sessions" fill="#2563EB" fillOpacity={0.8} radius={[2, 2, 0, 0]} />
        </BarChart>
      </ResponsiveContainer>
      <div className="chart-legend-row" aria-label="Hour buckets">
        {visibleHourLabels(chartData).map((hour) => (
          <span key={hour} className="chart-legend-chip">
            {formatHourTick(hour)}
          </span>
        ))}
      </div>
    </>
  );
}

function normalizeHours(data: GlobalHistogramBucket[]) {
  const counts = new Map<number, number>();
  for (const row of data) {
    counts.set(row.hour, (counts.get(row.hour) ?? 0) + row.count);
  }
  return Array.from({ length: 24 }, (_value, hour) => ({
    hour,
    count: counts.get(hour) ?? 0
  }));
}

function visibleHourLabels(data: GlobalHistogramBucket[]) {
  return Array.from(new Set([...hourTicks, ...data.filter((row) => row.count > 0).map((row) => row.hour)])).sort(
    (left, right) => left - right
  );
}

function formatHourTick(value: number) {
  return `${value}:00`;
}

function formatNumber(value: number | string): string {
  return typeof value === "number" ? new Intl.NumberFormat().format(value) : value;
}
