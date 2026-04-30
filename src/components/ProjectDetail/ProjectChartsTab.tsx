import { useState } from "react";
import {
  Bar,
  BarChart,
  CartesianGrid,
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis
} from "recharts";
import { useQuery } from "@tanstack/react-query";

import { ChartCard } from "../charts/ChartCard";
import { ChartTooltip } from "../charts/ChartTooltip";
import { getProjectChartData } from "../../lib/ipc";
import { projectChartsQueryKey } from "../../lib/queryClient";
import type { ProjectChartRange, ProjectDailyAverageDuration } from "../../lib/types";

const ranges: Array<{ value: ProjectChartRange; label: string }> = [
  { value: "7d", label: "7d" },
  { value: "30d", label: "30d" },
  { value: "90d", label: "90d" },
  { value: "all", label: "All" }
];

const axisStyle = { fill: "#6B7280", fontSize: 12 };
const chartMargin = { top: 8, right: 12, bottom: 0, left: 0 };

export type ProjectChartsTabProps = {
  projectId: string;
};

export function ProjectChartsTab({ projectId }: ProjectChartsTabProps) {
  const [range, setRange] = useState<ProjectChartRange>("30d");
  const charts = useQuery({
    queryKey: projectChartsQueryKey(projectId, range),
    queryFn: () => getProjectChartData(projectId, range)
  });
  const data = charts.data;

  if (charts.isError) {
    return (
      <section className="chart-card" role="alert">
        <h2 className="chart-card-title">Charts could not be loaded</h2>
        <p className="chart-card-subtitle">Rebuild the cache or re-index sessions and try again.</p>
      </section>
    );
  }

  return (
    <div className="charts-tab">
      <div className="range-selector" aria-label="Chart range">
        {ranges.map((option) => (
          <button
            key={option.value}
            type="button"
            className="range-btn"
            aria-pressed={range === option.value}
            onClick={() => setRange(option.value)}
          >
            {option.label}
          </button>
        ))}
      </div>

      <div className="charts-grid">
        <ChartCard
          title="Sessions per day"
          subtitle={`${rangeLabel(range)} session volume`}
          loading={charts.isLoading}
          empty={!data?.sessionsPerDay.length}
        >
          <ResponsiveContainer width="100%" height={200}>
            <BarChart data={data?.sessionsPerDay ?? []} margin={chartMargin}>
              <CartesianGrid stroke="#E5E7EB" strokeDasharray="3 3" />
              <XAxis dataKey="date" tick={axisStyle} tickLine={false} />
              <YAxis tick={axisStyle} tickLine={false} width={44} />
              <Tooltip content={<ChartTooltip valueFormatter={formatNumber} />} />
              <Bar dataKey="count" name="Sessions" fill="#2563EB" fillOpacity={0.8} radius={[2, 2, 0, 0]} />
            </BarChart>
          </ResponsiveContainer>
        </ChartCard>

        <ChartCard
          title="Tokens per day"
          subtitle={`${rangeLabel(range)} attributed token totals`}
          loading={charts.isLoading}
          empty={!data?.tokensPerDay.length}
        >
          <ResponsiveContainer width="100%" height={200}>
            <BarChart data={data?.tokensPerDay ?? []} margin={chartMargin}>
              <CartesianGrid stroke="#E5E7EB" strokeDasharray="3 3" />
              <XAxis dataKey="date" tick={axisStyle} tickLine={false} />
              <YAxis tick={axisStyle} tickLine={false} tickFormatter={formatCompactNumber} width={52} />
              <Tooltip content={<ChartTooltip valueFormatter={formatNumber} />} />
              <Bar dataKey="tokens" name="Tokens" fill="#7C3AED" fillOpacity={0.8} radius={[2, 2, 0, 0]} />
            </BarChart>
          </ResponsiveContainer>
        </ChartCard>

        <ChartCard
          title="Average duration"
          subtitle={`${rangeLabel(range)} wall-clock session duration`}
          loading={charts.isLoading}
          empty={!data?.averageDurationPerDay.length}
        >
          <ResponsiveContainer width="100%" height={200}>
            <LineChart data={durationData(data?.averageDurationPerDay ?? [])} margin={chartMargin}>
              <CartesianGrid stroke="#E5E7EB" strokeDasharray="3 3" />
              <XAxis dataKey="date" tick={axisStyle} tickLine={false} />
              <YAxis tick={axisStyle} tickLine={false} tickFormatter={(value) => `${value}m`} width={44} />
              <Tooltip content={<ChartTooltip valueFormatter={(value) => `${value} min`} />} />
              <Line
                type="monotone"
                dataKey="averageMinutes"
                name="Average duration"
                stroke="#2563EB"
                strokeWidth={2}
                dot={{ r: 3 }}
                activeDot={{ r: 5 }}
              />
            </LineChart>
          </ResponsiveContainer>
        </ChartCard>

        <ChartCard
          title="Milestone velocity"
          subtitle={`${rangeLabel(range)} completed plans per week`}
          loading={charts.isLoading}
          empty={!data?.milestoneVelocity.length}
        >
          <ResponsiveContainer width="100%" height={200}>
            <BarChart data={data?.milestoneVelocity ?? []} margin={chartMargin}>
              <CartesianGrid stroke="#E5E7EB" strokeDasharray="3 3" />
              <XAxis dataKey="week" tick={axisStyle} tickLine={false} />
              <YAxis tick={axisStyle} tickLine={false} width={44} />
              <Tooltip content={<ChartTooltip valueFormatter={formatNumber} />} />
              <Bar
                dataKey="completedPlans"
                name="Completed plans"
                fill="#059669"
                fillOpacity={0.8}
                radius={[2, 2, 0, 0]}
              />
            </BarChart>
          </ResponsiveContainer>
        </ChartCard>
      </div>
    </div>
  );
}

function durationData(rows: ProjectDailyAverageDuration[]) {
  return rows.map((row) => ({
    date: row.date,
    averageMinutes: Math.round(row.averageDurationMs / 60_000)
  }));
}

function rangeLabel(range: ProjectChartRange): string {
  if (range === "all") {
    return "All-time";
  }
  return `Last ${range.replace("d", " days")}`;
}

function formatNumber(value: number | string): string {
  return typeof value === "number" ? new Intl.NumberFormat().format(value) : value;
}

function formatCompactNumber(value: number): string {
  return new Intl.NumberFormat(undefined, { notation: "compact", maximumFractionDigits: 1 }).format(value);
}
