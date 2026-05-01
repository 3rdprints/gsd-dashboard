import { Activity, AlertTriangle, CheckCircle2, Loader2 } from "lucide-react";

import type { WatcherRootStatus, WatcherStatus } from "../lib/types";

type WatcherStatusPanelProps = {
  status?: WatcherStatus;
  isLoading: boolean;
  isError: boolean;
};

export function WatcherStatusPanel({ status, isLoading, isError }: WatcherStatusPanelProps) {
  const degradedRoots = status?.roots.filter((root) => root.mode === "polling") ?? [];

  return (
    <section className="settings-panel" aria-labelledby="watcher-status-title">
      <div className="panel-heading">
        {isError ? (
          <AlertTriangle aria-hidden="true" size={20} strokeWidth={2} />
        ) : degradedRoots.length > 0 ? (
          <Activity aria-hidden="true" size={20} strokeWidth={2} />
        ) : isLoading ? (
          <Loader2 aria-hidden="true" size={20} strokeWidth={2} />
        ) : (
          <CheckCircle2 aria-hidden="true" size={20} strokeWidth={2} />
        )}
        <div>
          <p className="label-text">Live updates</p>
          <h2 id="watcher-status-title">Watcher Status</h2>
        </div>
      </div>

      {isError ? (
        <div className="parse-error-alert" role="alert">
          <p>Live update status could not be loaded. Reopen Settings or rebuild the cache and try again.</p>
        </div>
      ) : isLoading ? (
        <div className="watcher-status-skeleton" aria-busy="true">
          <span>Loading live update status</span>
        </div>
      ) : degradedRoots.length > 0 ? (
        <>
          <div className="watcher-status-banner" role="status">
            <strong>Live updates are using polling</strong>
            <p>{degradedRoots.length === 1 ? "Data is delayed, not stopped." : `${degradedRoots.length} roots are being checked every 60 seconds.`}</p>
          </div>
          <ul className="watcher-status-list">
            {degradedRoots.map((root) => (
              <li className="watcher-root-row" key={root.root}>
                <p className="watcher-root-path">{root.root}</p>
                <div className="watcher-root-meta">
                  <span className="watcher-status-pill polling">Polling</span>
                  <span>{reasonLabel(root.reasonCategory)}</span>
                  <span>Polling every {root.pollingIntervalSeconds ?? 60}s</span>
                </div>
                {root.fixHint ? <p className="watcher-fix-hint">{root.fixHint}</p> : null}
              </li>
            ))}
          </ul>
        </>
      ) : (
        <div className="watcher-root-row">
          <p className="watcher-root-path">Live updates active</p>
          <div className="watcher-root-meta">
            <span className="watcher-status-pill native">Native</span>
            <span>All watched roots are using native file updates.</span>
          </div>
        </div>
      )}
    </section>
  );
}

function reasonLabel(reasonCategory: WatcherRootStatus["reasonCategory"]) {
  switch (reasonCategory) {
    case "permission":
      return "Permission denied";
    case "watchLimit":
      return "System watch limit reached";
    case "filesystem":
      return "Filesystem does not support native watching";
    case "unknown":
    default:
      return "Native watcher unavailable";
  }
}
