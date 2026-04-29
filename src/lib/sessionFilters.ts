import type { AppSettings, GlobalSessionFilters, GlobalSessionsDefaultRange, ProjectSessionSortKey, SortDirection } from "./types";

export type SessionFilters = {
  source?: "claude" | "codex";
  projectId?: string;
  dateRange: GlobalSessionsDefaultRange | "today" | "custom";
  from?: string;
  to?: string;
  durationMinMinutes?: number;
  durationMaxMinutes?: number;
  tokensMin?: number;
  tokensMax?: number;
  unmatchedOnly: boolean;
  sort: ProjectSessionSortKey;
  direction: SortDirection;
  page: number;
};

const sourceValues = new Set(["claude", "codex"]);
const defaultRangeValues = new Set(["7d", "30d", "90d", "all"]);
const dateRangeValues = new Set(["today", "7d", "30d", "90d", "all", "custom"]);
const sortValues = new Set(["startedAt", "source", "durationMs", "messageCount", "tokensIn", "tokensOut", "tokenTotal"]);
const directionValues = new Set(["asc", "desc"]);

export function DEFAULT_FILTERS(settings?: Pick<AppSettings, "globalSessionsDefaultRange">): SessionFilters {
  const dateRange = settings?.globalSessionsDefaultRange ?? "7d";
  const dates = datesForRange(dateRange);
  return {
    dateRange,
    from: dates.from,
    to: dates.to,
    unmatchedOnly: false,
    sort: "startedAt",
    direction: "desc",
    page: 1
  };
}

export function parseFiltersFromUrl(
  params: URLSearchParams,
  defaults: SessionFilters = DEFAULT_FILTERS()
): SessionFilters {
  const source = parseSource(params.get("source"));
  const explicitRange = parseDateRange(params.get("range"));
  const from = parseDate(params.get("from"));
  const to = parseDate(params.get("to"));
  const hasCustomDates = from !== undefined || to !== undefined;
  const explicitRangeDates = explicitRange ? datesForRange(explicitRange) : undefined;
  const dateRange = explicitRange ?? (hasCustomDates ? inferDateRange(from, to) ?? "custom" : defaults.dateRange);

  return {
    ...defaults,
    source,
    projectId: parseString(params.get("project")),
    dateRange,
    from: from ?? (hasCustomDates ? undefined : explicitRange ? explicitRangeDates?.from : defaults.from),
    to: to ?? (hasCustomDates ? undefined : explicitRange ? explicitRangeDates?.to : defaults.to),
    durationMinMinutes: parseFiniteNumber(params.get("dmin")),
    durationMaxMinutes: parseFiniteNumber(params.get("dmax")),
    tokensMin: parseFiniteNumber(params.get("tmin")),
    tokensMax: parseFiniteNumber(params.get("tmax")),
    unmatchedOnly: params.get("unmatched") === "true",
    sort: parseSort(params.get("sort")) ?? defaults.sort,
    direction: parseDirection(params.get("dir")) ?? defaults.direction,
    page: parsePage(params.get("page"))
  };
}

export function serializeFiltersToUrl(filters: SessionFilters): URLSearchParams {
  const params = new URLSearchParams();
  params.set("range", filters.dateRange);
  setParam(params, "source", filters.source);
  setParam(params, "project", filters.projectId);
  setParam(params, "from", filters.from);
  setParam(params, "to", filters.to);
  setNumberParam(params, "dmin", filters.durationMinMinutes);
  setNumberParam(params, "dmax", filters.durationMaxMinutes);
  setNumberParam(params, "tmin", filters.tokensMin);
  setNumberParam(params, "tmax", filters.tokensMax);
  if (filters.unmatchedOnly) params.set("unmatched", "true");
  if (filters.sort !== "startedAt") params.set("sort", filters.sort);
  if (filters.direction !== "desc") params.set("dir", filters.direction);
  if (filters.page > 1) params.set("page", String(filters.page));
  return params;
}

export function filtersToGlobalSessionFilters(filters: SessionFilters): GlobalSessionFilters {
  return {
    source: filters.source,
    projectId: filters.unmatchedOnly ? undefined : filters.projectId,
    startedAfter: dateToStartMs(filters.from),
    startedBefore: dateToEndMs(filters.to),
    durationMinMs: minutesToMs(filters.durationMinMinutes),
    durationMaxMs: minutesToMs(filters.durationMaxMinutes),
    tokensMin: filters.tokensMin,
    tokensMax: filters.tokensMax,
    unmatchedOnly: filters.unmatchedOnly || undefined
  };
}

export function applyDateRange(filters: SessionFilters, dateRange: SessionFilters["dateRange"]): SessionFilters {
  const dates = datesForRange(dateRange);
  return {
    ...filters,
    dateRange,
    from: dates.from,
    to: dates.to,
    page: 1
  };
}

function datesForRange(dateRange: SessionFilters["dateRange"]) {
  const today = new Date();
  const todayIso = toDateInputValue(today);
  if (dateRange === "all" || dateRange === "custom") {
    return {};
  }
  const days = dateRange === "today" ? 0 : Number(dateRange.replace("d", "")) - 1;
  const from = new Date(today);
  from.setDate(today.getDate() - days);
  return { from: toDateInputValue(from), to: todayIso };
}

function inferDateRange(from: string | undefined, to: string | undefined): SessionFilters["dateRange"] | undefined {
  if (!from || !to) return undefined;
  for (const range of ["today", "7d", "30d", "90d"] as const) {
    const dates = datesForRange(range);
    if (dates.from === from && dates.to === to) return range;
  }
  return undefined;
}

function parseSource(value: string | null) {
  return value && sourceValues.has(value) ? (value as "claude" | "codex") : undefined;
}

function parseDateRange(value: string | null) {
  return value && dateRangeValues.has(value) ? (value as SessionFilters["dateRange"]) : undefined;
}

function parseSort(value: string | null) {
  return value && sortValues.has(value) ? (value as ProjectSessionSortKey) : undefined;
}

function parseDirection(value: string | null) {
  return value && directionValues.has(value) ? (value as SortDirection) : undefined;
}

function parseString(value: string | null) {
  return value && value.trim().length > 0 ? value : undefined;
}

function parseDate(value: string | null) {
  if (!value || !/^\d{4}-\d{2}-\d{2}$/.test(value)) return undefined;
  const parsed = new Date(`${value}T00:00:00`);
  return Number.isNaN(parsed.getTime()) || toDateInputValue(parsed) !== value ? undefined : value;
}

function parseFiniteNumber(value: string | null) {
  if (!value) return undefined;
  const parsed = Number(value);
  return Number.isFinite(parsed) && parsed >= 0 ? parsed : undefined;
}

function parsePage(value: string | null) {
  const parsed = parseFiniteNumber(value);
  return parsed && Number.isInteger(parsed) && parsed > 0 ? parsed : 1;
}

function setParam(params: URLSearchParams, key: string, value: string | undefined) {
  if (value) params.set(key, value);
}

function setNumberParam(params: URLSearchParams, key: string, value: number | undefined) {
  if (value !== undefined && Number.isFinite(value)) params.set(key, String(value));
}

function dateToStartMs(value: string | undefined) {
  return dateToLocalMs(value, 0, 0, 0, 0);
}

function dateToEndMs(value: string | undefined) {
  return dateToLocalMs(value, 23, 59, 59, 999);
}

function dateToLocalMs(value: string | undefined, hours: number, minutes: number, seconds: number, milliseconds: number) {
  if (!value) return undefined;
  // Date inputs are local calendar days, so convert boundaries using local time.
  const [year, month, day] = value.split("-").map(Number);
  return new Date(year, month - 1, day, hours, minutes, seconds, milliseconds).getTime();
}

function minutesToMs(value: number | undefined) {
  return value === undefined ? undefined : Math.round(value * 60_000);
}

function toDateInputValue(value: Date) {
  const year = value.getFullYear();
  const month = String(value.getMonth() + 1).padStart(2, "0");
  const day = String(value.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}
