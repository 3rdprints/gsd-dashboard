import { useMemo } from "react";
import { Loader2 } from "lucide-react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useSearchParams } from "react-router-dom";

import { ChartCard } from "../components/charts/ChartCard";
import { DayOfWeekChart } from "../components/charts/DayOfWeekChart";
import { StackedProjectsChart } from "../components/charts/StackedProjectsChart";
import { StackedSourcesChart } from "../components/charts/StackedSourcesChart";
import { TimeOfDayHistogram } from "../components/charts/TimeOfDayHistogram";
import { FilterBar } from "../components/sessions/FilterBar";
import { FilterChipsRow } from "../components/sessions/FilterChipsRow";
import { SessionsTable } from "../components/sessions/SessionsTable";
import { getGlobalChartData, getPortfolio, getSettings, listGlobalSessions } from "../lib/ipc";
import {
  createSaveSettingsMutationOptions,
  globalChartsQueryKey,
  globalSessionsQueryKey,
  portfolioQueryKey,
  settingsQueryKey
} from "../lib/queryClient";
import {
  DEFAULT_FILTERS,
  filtersToGlobalSessionFilters,
  parseFiltersFromUrl,
  serializeFiltersToUrl,
  type SessionFilters
} from "../lib/sessionFilters";
import type {
  AppSettings,
  GlobalChartData,
  GlobalSessionsDefaultRange,
  GlobalSessionsPage as GlobalSessionsPageDto
} from "../lib/types";
import "./GlobalSessionsPage.css";

const pageSize = 100;

const EMPTY_GLOBAL_CHART_DATA: GlobalChartData = {
  sessionsPerDayBySource: [],
  tokensPerDayByProject: [],
  timeOfDayHistogram: [],
  dayOfWeekDistribution: []
};

const formatTotal = (total: number) => `${new Intl.NumberFormat().format(total)} sessions`;

const LoadingSessionsHeading = () => (
  <>
    <p className="label-text">Session index</p>
    <h2 className="chart-card-title">Preparing session view</h2>
  </>
);

const LoadingSessionsView = () => (
  <div className="page-stack global-sessions-page">
    <div className="app-header">
      <header>
        <h1>Sessions</h1>
        <p>Preparing filters</p>
      </header>
    </div>
    <section className="chart-card sessions-loading-panel" aria-busy="true">
      <div className="panel-heading">
        <Loader2 aria-hidden="true" size={20} strokeWidth={2} />
        <div><LoadingSessionsHeading /></div>
      </div>
      <p className="chart-card-subtitle">
        Loading saved filters before querying the local session cache.
      </p>
      <div className="table-skeleton labeled-skeleton" aria-label="Loading sessions table">
        <span>Session table loading</span>
      </div>
    </section>
  </div>
);

const GlobalChartsError = () => (
  <section className="chart-card" role="alert">
    <h2 className="chart-card-title">Charts could not be loaded</h2>
    <p className="chart-card-subtitle">Rebuild the cache or re-index sessions and try again.</p>
  </section>
);

type GlobalChartsPanelProps = {
  chartData: GlobalChartData | undefined;
  isError: boolean;
  isLoading: boolean;
};

const hasRows = (rows: readonly unknown[] | undefined) => Boolean(rows?.length);

const hasPositiveCounts = (rows: readonly { count: number }[] | undefined) =>
  Boolean(rows?.some((bucket) => bucket.count > 0));

const GlobalChartsGrid = ({ chartData, isLoading }: Omit<GlobalChartsPanelProps, "isError">) => {
  const data = chartData ?? EMPTY_GLOBAL_CHART_DATA;
  return (
    <div className="charts-grid" aria-label="Global session charts">
      <ChartCard
        title="Sessions by source"
        subtitle="Daily Claude and Codex session volume"
        loading={isLoading}
        empty={!hasRows(data.sessionsPerDayBySource)}
      >
        <StackedSourcesChart data={data.sessionsPerDayBySource} />
      </ChartCard>
      <ChartCard
        title="Tokens by project"
        subtitle="Daily tokens for top projects"
        loading={isLoading}
        empty={!hasRows(data.tokensPerDayByProject)}
      >
        <StackedProjectsChart data={data.tokensPerDayByProject} />
      </ChartCard>
      <ChartCard
        title="Time of day"
        subtitle="Sessions by local start hour"
        loading={isLoading}
        empty={!hasPositiveCounts(data.timeOfDayHistogram)}
      >
        <TimeOfDayHistogram data={data.timeOfDayHistogram} />
      </ChartCard>
      <ChartCard
        title="Day of week"
        subtitle="Sessions by local weekday"
        loading={isLoading}
        empty={!hasPositiveCounts(data.dayOfWeekDistribution)}
      >
        <DayOfWeekChart data={data.dayOfWeekDistribution} />
      </ChartCard>
    </div>
  );
};

const GlobalChartsPanel = ({ chartData, isError, isLoading }: GlobalChartsPanelProps) =>
  isError ? <GlobalChartsError /> : <GlobalChartsGrid chartData={chartData} isLoading={isLoading} />;

type GlobalSessionsPanelProps = {
  direction: SessionFilters["direction"];
  isError: boolean;
  isLoading: boolean;
  onPageChange: (page: number) => void;
  onSortChange: (sort: SessionFilters["sort"], direction: SessionFilters["direction"]) => void;
  pageData: GlobalSessionsPageDto;
  sort: SessionFilters["sort"];
};

const GlobalSessionsPanel = ({
  direction,
  isError,
  isLoading,
  onPageChange,
  onSortChange,
  pageData,
  sort
}: GlobalSessionsPanelProps) => {
  if (isLoading) {
    return (
      <section className="chart-card">
        <div className="table-skeleton" aria-label="Loading sessions" />
      </section>
    );
  }

  if (isError) {
    return (
      <section className="chart-card" role="alert">
        <h2 className="chart-card-title">Sessions could not be loaded</h2>
        <p className="chart-card-subtitle">Rebuild the cache or re-index sessions and try again.</p>
      </section>
    );
  }

  return (
    <section className="chart-card">
      <SessionsTable
        rows={pageData.rows}
        total={pageData.total}
        page={pageData.page}
        pageSize={pageData.pageSize}
        sort={sort}
        direction={direction}
        showProject
        onSortChange={onSortChange}
        onPageChange={onPageChange}
      />
    </section>
  );
};

const getDefaultFilters = (settings: AppSettings | undefined, isError: boolean) => {
  if (settings) {
    return DEFAULT_FILTERS({ globalSessionsDefaultRange: settings.globalSessionsDefaultRange });
  }

  return isError ? DEFAULT_FILTERS({ globalSessionsDefaultRange: "7d" }) : undefined;
};

const getNextDefaultRangeSettings = (settings: AppSettings | undefined, range: GlobalSessionsDefaultRange) => {
  if (!settings || settings.globalSessionsDefaultRange === range) {
    return undefined;
  }

  return { ...settings, globalSessionsDefaultRange: range };
};

const useParsedFilters = (
  searchParams: URLSearchParams,
  settings: AppSettings | undefined,
  isSettingsError: boolean
) => {
  const defaultFilters = useMemo(
    () => getDefaultFilters(settings, isSettingsError),
    [settings, isSettingsError]
  );

  return useMemo(
    () => (defaultFilters ? parseFiltersFromUrl(searchParams, defaultFilters) : undefined),
    [searchParams, defaultFilters]
  );
};

const useGlobalSessionsQuery = (
  filters: SessionFilters | undefined,
  ipcFilters: ReturnType<typeof filtersToGlobalSessionFilters> | undefined
) =>
  useQuery({
    queryKey: globalSessionsQueryKey(
      ipcFilters ?? {},
      filters?.sort ?? "startedAt",
      filters?.direction ?? "desc",
      filters?.page ?? 1,
      pageSize
    ),
    queryFn: () => listGlobalSessions(ipcFilters!, filters!.sort, filters!.direction, filters!.page, pageSize),
    enabled: !!ipcFilters && !!filters
  });

const useGlobalChartsQuery = (ipcFilters: ReturnType<typeof filtersToGlobalSessionFilters> | undefined) =>
  useQuery({
    queryKey: globalChartsQueryKey(ipcFilters ?? {}),
    queryFn: () => getGlobalChartData(ipcFilters!),
    enabled: !!ipcFilters
  });

/**
 * Renders the global sessions route.
 */
export const GlobalSessionsPage = () => {
  const [searchParams, setSearchParams] = useSearchParams();
  const queryClient = useQueryClient();
  const settings = useQuery({ queryKey: settingsQueryKey, queryFn: getSettings });
  const portfolio = useQuery({ queryKey: portfolioQueryKey, queryFn: getPortfolio });
  const saveSettings = useMutation(createSaveSettingsMutationOptions(queryClient));
  const filters = useParsedFilters(searchParams, settings.data, settings.isError);
  const ipcFilters = useMemo(() => (filters ? filtersToGlobalSessionFilters(filters) : undefined), [filters]);
  const sessions = useGlobalSessionsQuery(filters, ipcFilters);
  const charts = useGlobalChartsQuery(ipcFilters);
  const projects = portfolio.data?.projects ?? [];

  const setFilters = (nextFilters: SessionFilters) => {
    setSearchParams(serializeFiltersToUrl(nextFilters));
  };

  const clearFilters = () => {
    setSearchParams(new URLSearchParams());
  };

  const persistDefaultRange = (range: GlobalSessionsDefaultRange) => {
    const nextSettings = getNextDefaultRangeSettings(settings.data, range);
    if (nextSettings) saveSettings.mutate(nextSettings);
  };

  if (!filters) {
    return <LoadingSessionsView />;
  }

  const pageData = sessions.data ?? { rows: [], total: 0, page: filters.page, pageSize };
  return (
    <div className="page-stack global-sessions-page">
      <div className="app-header">
        <header>
          <h1>Sessions</h1>
          <p>{formatTotal(pageData.total)}</p>
        </header>
      </div>
      <FilterBar
        filters={filters}
        projects={projects}
        onChange={setFilters}
        onDateRangePersist={persistDefaultRange}
      />
      <FilterChipsRow filters={filters} projects={projects} onChange={setFilters} onClearAll={clearFilters} />
      <GlobalChartsPanel chartData={charts.data} isError={charts.isError} isLoading={charts.isLoading} />
      <GlobalSessionsPanel
        direction={filters.direction}
        isError={sessions.isError}
        isLoading={sessions.isLoading}
        onPageChange={(page) => setFilters({ ...filters, page })}
        onSortChange={(sort, direction) => setFilters({ ...filters, sort, direction, page: 1 })}
        pageData={pageData}
        sort={filters.sort}
      />
    </div>
  );
};
