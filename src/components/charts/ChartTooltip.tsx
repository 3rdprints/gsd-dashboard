type TooltipPayload = {
  name?: string;
  value?: number | string;
  color?: string;
};

export type ChartTooltipProps = {
  active?: boolean;
  label?: string;
  payload?: TooltipPayload[];
  valueFormatter?: (value: number | string) => string;
};

/**
 * Provides the exported chart tooltip function.
 */
export function ChartTooltip({ active, label, payload, valueFormatter = String }: ChartTooltipProps) {
  if (!active || !payload?.length) {
    return null;
  }

  return (
    <div className="chart-tooltip">
      {label ? <p className="chart-tooltip-label">{label}</p> : null}
      {payload.map((entry, index) => (
        <p key={`${entry.name ?? "value"}-${entry.value}-${index}`} className="chart-tooltip-row">
          <span className="chart-tooltip-dot" style={{ background: entry.color ?? "#2563EB" }} />
          <span>{entry.name ?? "Value"}</span>
          <strong>{valueFormatter(entry.value ?? 0)}</strong>
        </p>
      ))}
    </div>
  );
}
