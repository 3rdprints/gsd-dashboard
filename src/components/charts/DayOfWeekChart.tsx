import { Bar, BarChart, CartesianGrid, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";

import type { GlobalDayOfWeekBucket } from "../../lib/types";
import { ChartTooltip } from "./ChartTooltip";

const axisStyle = { fill: "#6B7280", fontSize: 12 };
const chartMargin = { top: 8, right: 12, bottom: 0, left: 0 };
const days = [
  { backendDay: 1, label: "Mon" },
  { backendDay: 2, label: "Tue" },
  { backendDay: 3, label: "Wed" },
  { backendDay: 4, label: "Thu" },
  { backendDay: 5, label: "Fri" },
  { backendDay: 6, label: "Sat" },
  { backendDay: 0, label: "Sun" }
];

export type DayOfWeekChartProps = {
  data: GlobalDayOfWeekBucket[];
};

/**
 * Renders the day of week chart.
 */
export function DayOfWeekChart({ data }: DayOfWeekChartProps) {
  const chartData = normalizeDays(data);

  return (
    <>
      <ResponsiveContainer width="100%" height={200}>
        <BarChart data={chartData} margin={chartMargin}>
          <CartesianGrid stroke="#E5E7EB" strokeDasharray="3 3" />
          <XAxis dataKey="label" tick={axisStyle} tickLine={false} />
          <YAxis tick={axisStyle} tickLine={false} width={44} />
          <Tooltip content={<ChartTooltip valueFormatter={formatNumber} />} />
          <Bar dataKey="count" name="Sessions" fill="#2563EB" fillOpacity={0.8} radius={[2, 2, 0, 0]} />
        </BarChart>
      </ResponsiveContainer>
      <div className="chart-legend-row" aria-label="Weekday buckets">
        {days.map((day) => (
          <span key={day.label} className="chart-legend-chip">
            {day.label}
          </span>
        ))}
      </div>
    </>
  );
}

function normalizeDays(data: GlobalDayOfWeekBucket[]) {
  const counts = new Map(data.map((row) => [row.day, row.count]));
  return days.map((day) => ({
    label: day.label,
    count: counts.get(day.backendDay) ?? 0
  }));
}

function formatNumber(value: number | string): string {
  return typeof value === "number" ? new Intl.NumberFormat().format(value) : value;
}
