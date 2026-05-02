import { useState } from "react";
import { Database, Eye, Loader2, Trash2 } from "lucide-react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { ScanRootsEditor } from "../components/ScanRootsEditor";
import { WatcherStatusPanel } from "../components/WatcherStatusPanel";
import { Button } from "../components/ui/button";
import { Checkbox } from "../components/ui/checkbox";
import {
  completeScanState,
  initialScanState,
  reduceScanEvent,
  ScanProgressPanel
} from "../components/ScanProgressPanel";
import { clearSessionIndex, getPortfolio, getSettings, getWatcherStatus, rebuildCache } from "../lib/ipc";
import {
  createSaveSettingsMutationOptions,
  portfolioQueryKey,
  settingsQueryKey,
  watcherStatusQueryKey
} from "../lib/queryClient";
import "./SettingsPage.css";

const REBUILD_CONFIRMATION =
  "Rebuild cache: This clears the derived project cache and runs a full rescan. Source `.planning/` files will not be changed.";
const CLEAR_SESSION_INDEX_CONFIRMATION =
  "Clear session index: This removes derived Claude/Codex session rows and index offsets. Source session files will not be changed.";

export function SettingsPage() {
  const queryClient = useQueryClient();
  const settings = useQuery({ queryKey: settingsQueryKey, queryFn: getSettings });
  const portfolio = useQuery({ queryKey: portfolioQueryKey, queryFn: getPortfolio });
  const watcherStatus = useQuery({ queryKey: watcherStatusQueryKey(), queryFn: getWatcherStatus });
  const saveSettings = useMutation(createSaveSettingsMutationOptions(queryClient));
  const [scanState, setScanState] = useState(initialScanState);
  const [confirmedRebuild, setConfirmedRebuild] = useState(false);
  const [confirmedClearSessionIndex, setConfirmedClearSessionIndex] = useState(false);
  const [clearSessionIndexError, setClearSessionIndexError] = useState<string | null>(null);
  const rebuildCacheMutation = useMutation({
    mutationFn: () =>
      rebuildCache((event) => {
        setScanState((current) => reduceScanEvent(current, event));
      }),
    onMutate: () => {
      setScanState({
        ...initialScanState,
        status: "scanning",
        progressText: "Walking scan roots"
      });
    },
    onSuccess: async (summary) => {
      setScanState((current) => completeScanState(current, summary));
      await queryClient.invalidateQueries({ queryKey: portfolioQueryKey });
    },
    onError: () => {
      setScanState((current) => ({
        ...current,
        status: "failed",
        progressText: "Scan failed"
      }));
    }
  });
  const isRebuilding = scanState.status === "scanning" || rebuildCacheMutation.isPending;
  const clearSessionIndexMutation = useMutation({
    mutationFn: clearSessionIndex,
    onMutate: () => {
      setClearSessionIndexError(null);
    },
    onSuccess: async () => {
      setConfirmedClearSessionIndex(false);
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: portfolioQueryKey }),
        queryClient.invalidateQueries({
          predicate: (query) =>
            query.queryKey[0] === "globalSessions" ||
            query.queryKey[0] === "globalCharts" ||
            query.queryKey[0] === "project"
        })
      ]);
    },
    onError: (error) => {
      setConfirmedClearSessionIndex(false);
      setClearSessionIndexError(error instanceof Error ? error.message : String(error || "Clear session index failed"));
    }
  });

  async function handleUnhide(projectId: string) {
    if (!settings.data) return;

    await saveSettings.mutateAsync({
      ...settings.data,
      hiddenProjectIds: settings.data.hiddenProjectIds.filter((id) => id !== projectId)
    });
  }

  function handleRebuild() {
    if (!confirmedRebuild || isRebuilding) return;

    rebuildCacheMutation.mutate();
  }

  function handleClearSessionIndex() {
    if (!confirmedClearSessionIndex || clearSessionIndexMutation.isPending) return;

    clearSessionIndexMutation.mutate();
  }

  return (
    <div className="page-stack">
      <div className="app-header">
        <header>
          <h1>Settings</h1>
          <p>Scan roots and cache controls</p>
        </header>
      </div>

      <ScanRootsEditor title="Scan roots" />

      <WatcherStatusPanel
        status={watcherStatus.data}
        isLoading={watcherStatus.isLoading}
        isError={watcherStatus.isError}
      />

      <section className="settings-panel" aria-labelledby="hidden-projects-title">
        <div className="panel-heading">
          <Eye aria-hidden="true" size={20} strokeWidth={2} />
          <div>
            <p className="label-text">Portfolio visibility</p>
            <h2 id="hidden-projects-title">Hidden projects</h2>
          </div>
        </div>

        {portfolio.data && portfolio.data.hiddenProjects.length > 0 ? (
          <ul className="settings-list">
            {portfolio.data.hiddenProjects.map((project) => (
              <li key={project.id}>
                <span>{project.name}</span>
                <Button
                  variant="outline"
                  type="button"
                  onClick={() => handleUnhide(project.id)}
                  disabled={saveSettings.isPending}
                >
                  Unhide Project
                </Button>
              </li>
            ))}
          </ul>
        ) : (
          <p className="muted-copy">No hidden projects</p>
        )}
      </section>

      <section className="settings-panel" aria-labelledby="rebuild-cache-title">
        <div className="panel-heading">
          <Database aria-hidden="true" size={20} strokeWidth={2} />
          <div>
            <p className="label-text">Derived cache</p>
            <h2 id="rebuild-cache-title">Rebuild Cache</h2>
          </div>
        </div>
        <p className="confirmation-copy">{REBUILD_CONFIRMATION}</p>
        <label className="checkbox-row">
          <Checkbox
            checked={confirmedRebuild}
            onCheckedChange={(checked) => setConfirmedRebuild(Boolean(checked))}
          />
          Confirm rebuild cache
        </label>
        <Button type="button" onClick={handleRebuild} disabled={!confirmedRebuild || isRebuilding}>
          {isRebuilding ? (
            <Loader2 aria-hidden="true" size={16} strokeWidth={2} />
          ) : (
            <Database aria-hidden="true" size={16} strokeWidth={2} />
          )}
          Rebuild Cache
        </Button>
      </section>

      <ScanProgressPanel state={scanState} />

      <section className="settings-panel" aria-labelledby="indexing-title">
        <h2 id="indexing-title">Indexing</h2>
        <p className="confirmation-copy">{CLEAR_SESSION_INDEX_CONFIRMATION}</p>
        <label className="checkbox-row">
          <Checkbox
            checked={confirmedClearSessionIndex}
            onCheckedChange={(checked) => setConfirmedClearSessionIndex(Boolean(checked))}
          />
          Confirm clear session index
        </label>
        <Button
          type="button"
          onClick={handleClearSessionIndex}
          disabled={!confirmedClearSessionIndex || clearSessionIndexMutation.isPending}
        >
          {clearSessionIndexMutation.isPending ? (
            <Loader2 aria-hidden="true" size={16} strokeWidth={2} />
          ) : (
            <Trash2 aria-hidden="true" size={16} strokeWidth={2} />
          )}
          Clear Session Index
        </Button>
        {clearSessionIndexError ? (
          <div className="parse-error-alert" role="alert">
            <p>{clearSessionIndexError}</p>
          </div>
        ) : null}
        <label className="checkbox-row disabled-row">
          <Checkbox disabled />
          Index tool usage
        </label>
        <label className="checkbox-row disabled-row">
          <Checkbox disabled />
          Index message content
        </label>
      </section>
    </div>
  );
}
