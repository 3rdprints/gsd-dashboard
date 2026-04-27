import { useMemo } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useSearchParams } from "react-router-dom";

import { FilterBar } from "../components/sessions/FilterBar";
import { FilterChipsRow } from "../components/sessions/FilterChipsRow";
import { SessionsTable } from "../components/sessions/SessionsTable";
import { getPortfolio, getSettings, listGlobalSessions } from "../lib/ipc";
import {
  createSaveSettingsMutationOptions,
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
import type { GlobalSessionsDefaultRange } from "../lib/types";
import "./GlobalSessionsPage.css";

const pageSize = 100;

export function GlobalSessionsPage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const queryClient = useQueryClient();
  const settings = useQuery({ queryKey: settingsQueryKey, queryFn: getSettings });
  const portfolio = useQuery({ queryKey: portfolioQueryKey, queryFn: getPortfolio });
  const saveSettings = useMutation(createSaveSettingsMutationOptions(queryClient));
  const defaultFilters = useMemo(
    () => DEFAULT_FILTERS(settings.data ? { globalSessionsDefaultRange: settings.data.globalSessionsDefaultRange } : undefined),
    [settings.data]
  );
  const filters = useMemo(() => parseFiltersFromUrl(searchParams, defaultFilters), [searchParams, defaultFilters]);
  const ipcFilters = useMemo(() => filtersToGlobalSessionFilters(filters), [filters]);
  const sessions = useQuery({
    queryKey: globalSessionsQueryKey(ipcFilters, filters.page, pageSize),
    queryFn: () => listGlobalSessions(ipcFilters, filters.page, pageSize)
  });
  const projects = portfolio.data?.projects ?? [];

  function setFilters(nextFilters: SessionFilters) {
    setSearchParams(serializeFiltersToUrl(nextFilters));
  }

  function clearFilters() {
    setSearchParams(new URLSearchParams());
  }

  function persistDefaultRange(range: GlobalSessionsDefaultRange) {
    if (!settings.data || settings.data.globalSessionsDefaultRange === range) return;
    saveSettings.mutate({ ...settings.data, globalSessionsDefaultRange: range });
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
      <section className="global-chart-placeholder" aria-label="Global session charts">
        <div>
          <h2>Global charts</h2>
          <p>Charts for the active filters are added in Plan 05-11.</p>
        </div>
      </section>
      {sessions.isLoading ? (
        <section className="chart-card">
          <div className="table-skeleton" aria-label="Loading sessions" />
        </section>
      ) : null}
      {sessions.isError ? (
        <section className="chart-card" role="alert">
          <h2 className="chart-card-title">Sessions could not be loaded</h2>
          <p className="chart-card-subtitle">Rebuild the cache or re-index sessions and try again.</p>
        </section>
      ) : null}
      {!sessions.isLoading && !sessions.isError ? (
        <section className="chart-card">
          {pageData.total === 0 ? (
            <div className="empty-state">
              <h3>No sessions found</h3>
              <p>Try widening the date range or removing active filters.</p>
            </div>
          ) : null}
          <SessionsTable
            rows={pageData.rows}
            total={pageData.total}
            page={pageData.page}
            pageSize={pageData.pageSize}
            sort="startedAt"
            direction="desc"
            showProject
            onSortChange={() => undefined}
            onPageChange={(page) => setFilters({ ...filters, page })}
          />
        </section>
      ) : null}
    </div>
  );
}

function formatTotal(total: number) {
  return `${new Intl.NumberFormat().format(total)} sessions`;
}
