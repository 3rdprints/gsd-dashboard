import CalendarHeatmap from "react-calendar-heatmap";

import type { HeatmapDay } from "../../lib/types";

export type HeatmapCalendarValue = {
  date: string;
  count?: number;
  tokenTotal?: number;
  topProjectName?: string | null;
};

type ActivityHeatmapProps = {
  days: HeatmapDay[];
  endDate?: Date;
};

export function heatmapClassForValue(value?: HeatmapCalendarValue | null) {
  const count = value?.count ?? 0;

  if (count >= 15) return "heatmap-cell-5";
  if (count >= 8) return "heatmap-cell-4";
  if (count >= 4) return "heatmap-cell-3";
  if (count >= 2) return "heatmap-cell-2";
  if (count >= 1) return "heatmap-cell-1";
  return "heatmap-cell-0";
}

export function heatmapTitleForValue(value?: HeatmapCalendarValue | null) {
  const count = value?.count ?? 0;
  const sessionLabel = count === 1 ? "session" : "sessions";
  const projectName = value?.topProjectName ?? "unattributed";
  const tokenTotal = value?.tokenTotal ?? 0;

  return `${count.toLocaleString()} ${sessionLabel} · ${projectName} · ${tokenTotal.toLocaleString()} tokens`;
}

export function ActivityHeatmap({ days, endDate = new Date() }: ActivityHeatmapProps) {
  const values = days.map((day) => ({
    date: day.date,
    count: day.sessionCount,
    tokenTotal: day.tokenTotal,
    topProjectName: day.topProjectName
  }));
  const startDate = new Date(endDate);
  startDate.setDate(startDate.getDate() - 89);

  return (
    <section
      className="chart-card activity-heatmap-card"
      aria-label="Activity heatmap for the last 90 days"
    >
      <div className="chart-card-header">
        <div>
          <h2 className="chart-card-title">Activity — last 90 days</h2>
          <p className="chart-card-subtitle">Sessions started per day</p>
        </div>
      </div>
      <div className="heatmap-wrapper">
        <div className="heatmap-container">
          <CalendarHeatmap
            startDate={startDate}
            endDate={endDate}
            values={values}
            showWeekdayLabels
            gutterSize={2}
            classForValue={heatmapClassForValue}
            titleForValue={heatmapTitleForValue}
          />
        </div>
      </div>
    </section>
  );
}
