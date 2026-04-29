import { Bar, BarChart, CartesianGrid, Legend, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";

import type { GlobalTokensByProjectDay } from "../../lib/types";
import { ChartTooltip } from "./ChartTooltip";

const axisStyle = { fill: "#6B7280", fontSize: 12 };
const chartMargin = { top: 8, right: 12, bottom: 0, left: 0 };
const projectColors = ["#2563EB", "#7C3AED", "#059669", "#D97706", "#DC2626"];

type ProjectSeries = {
  key: string;
  name: string;
  fill: string;
};

export type StackedProjectsChartProps = {
  data: GlobalTokensByProjectDay[];
};

export function StackedProjectsChart({ data }: StackedProjectsChartProps) {
  const { chartData, series } = toStackedProjectData(data);

  return (
    <>
      <ResponsiveContainer width="100%" height={200}>
        <BarChart data={chartData} margin={chartMargin}>
          <CartesianGrid stroke="#E5E7EB" strokeDasharray="3 3" />
          <XAxis dataKey="date" tick={axisStyle} tickLine={false} />
          <YAxis tick={axisStyle} tickLine={false} tickFormatter={formatCompactNumber} width={52} />
          <Tooltip content={<ChartTooltip valueFormatter={formatNumber} />} />
          <Legend wrapperStyle={{ color: "#4B5563", fontSize: 12 }} />
          {series.map((item) => (
            <Bar
              key={item.key}
              dataKey={item.key}
              name={item.name}
              stackId="projects"
              fill={item.fill}
              fillOpacity={0.8}
              radius={[2, 2, 0, 0]}
            />
          ))}
        </BarChart>
      </ResponsiveContainer>
      <div className="chart-legend-row" aria-label="Project chart legend">
        {series.map((item) => (
          <span key={item.key} className="chart-legend-chip">
            {item.name}
          </span>
        ))}
      </div>
    </>
  );
}

function toStackedProjectData(data: GlobalTokensByProjectDay[]) {
  const projectIds = Array.from(new Set(data.filter((row) => row.projectId).map((row) => row.projectId as string))).slice(0, 5);
  const projectIdSet = new Set(projectIds);
  const series: ProjectSeries[] = projectIds.map((projectId, index) => {
    const row = data.find((item) => item.projectId === projectId);
    return {
      key: `project${index}`,
      name: row?.projectName ?? projectId,
      fill: projectColors[index]
    };
  });

  if (data.some((row) => !row.projectId || !projectIdSet.has(row.projectId))) {
    series.push({ key: "other", name: "Other", fill: "#9CA3AF" });
  }

  const projectKeyById = new Map(projectIds.map((projectId, index) => [projectId, `project${index}`]));
  const rowsByDate = new Map<string, Record<string, number | string>>();
  for (const row of data) {
    const chartRow = rowsByDate.get(row.date) ?? { date: row.date };
    const key = row.projectId && projectIdSet.has(row.projectId) ? projectKeyById.get(row.projectId) : "other";
    if (key) {
      chartRow[key] = Number(chartRow[key] ?? 0) + row.tokens;
    }
    rowsByDate.set(row.date, chartRow);
  }

  return { chartData: Array.from(rowsByDate.values()), series };
}

function formatNumber(value: number | string): string {
  return typeof value === "number" ? new Intl.NumberFormat().format(value) : value;
}

function formatCompactNumber(value: number): string {
  return new Intl.NumberFormat(undefined, { notation: "compact", maximumFractionDigits: 1 }).format(value);
}
