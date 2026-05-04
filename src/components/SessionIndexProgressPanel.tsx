import { AlertTriangle, CheckCircle2, Database, Loader2 } from "lucide-react";

import type { SessionIndexEvent, SessionIndexSummary } from "../lib/types";

export type SessionIndexRunStatus = "ready" | "indexing" | "complete" | "failed";

export type SessionIndexState = {
  status: SessionIndexRunStatus;
  filesProcessed: number;
  sessionsPersisted: number;
  unmatchedCount: number;
  errorCount: number;
  progressText: string;
  sourceLabel: string;
  livePartialSeen: boolean;
};

export const initialSessionIndexState: SessionIndexState = {
  status: "ready",
  filesProcessed: 0,
  sessionsPersisted: 0,
  unmatchedCount: 0,
  errorCount: 0,
  progressText: "Ready to index sessions",
  sourceLabel: "Claude Code and Codex",
  livePartialSeen: false
};

type SessionIndexProgressPanelProps = {
  state: SessionIndexState;
};

/**
 * Renders the session index progress panel.
 */
export function SessionIndexProgressPanel({ state }: SessionIndexProgressPanelProps) {
  const isIndexing = state.status === "indexing";
  const failed = state.status === "failed";
  const completedWithErrors = state.status === "complete" && state.errorCount > 0;

  return (
    <section className="scan-status" aria-labelledby="session-index-status-title">
      <div className="panel-heading">
        {failed || completedWithErrors ? (
          <AlertTriangle aria-hidden="true" size={20} strokeWidth={2} />
        ) : state.status === "complete" ? (
          <CheckCircle2 aria-hidden="true" size={20} strokeWidth={2} />
        ) : isIndexing ? (
          <Loader2 aria-hidden="true" size={20} strokeWidth={2} />
        ) : (
          <Database aria-hidden="true" size={20} strokeWidth={2} />
        )}
        <div>
          <p className="label-text">{state.sourceLabel}</p>
          <h2 id="session-index-status-title">
            {failed
              ? "Session indexing failed"
              : state.status === "complete"
                ? "Sessions indexed"
                : isIndexing
                  ? "Indexing sessions"
                  : "No sessions indexed"}
          </h2>
        </div>
      </div>

      <div className="scan-progress">
        <div className="scan-progress-track" aria-hidden="true">
          <div
            className="scan-progress-fill"
            style={{ width: isIndexing ? "55%" : state.status === "complete" ? "100%" : "0%" }}
          />
        </div>
        <p aria-live="polite">{state.progressText}</p>
      </div>

      <div className="session-index-metrics" aria-label="Session indexing totals">
        <span>{state.filesProcessed} files</span>
        <span>{state.sessionsPersisted} sessions</span>
        <span>{state.unmatchedCount} unmatched</span>
      </div>

      {state.livePartialSeen ? <p className="session-index-note">Live session still writing</p> : null}
    </section>
  );
}

/**
 * Reduces incoming progress events into session index event.
 */
export function reduceSessionIndexEvent(
  current: SessionIndexState,
  event: SessionIndexEvent
): SessionIndexState {
  switch (event.event) {
    case "started":
      return {
        ...initialSessionIndexState,
        status: "indexing",
        progressText: "Indexing sessions"
      };
    case "sourceStarted":
      return {
        ...current,
        status: "indexing",
        sourceLabel: sourceLabel(event.data.source),
        progressText: `Reading ${sourceLabel(event.data.source)}`
      };
    case "fileIndexed":
      return {
        ...current,
        status: "indexing",
        filesProcessed: current.filesProcessed + 1,
        sessionsPersisted: current.sessionsPersisted + event.data.sessionsPersisted,
        progressText: event.data.livePartial ? "Live session still writing" : "Saving session metadata",
        livePartialSeen: current.livePartialSeen || event.data.livePartial
      };
    case "fileIndexError":
      return {
        ...current,
        status: "indexing",
        filesProcessed: current.filesProcessed + 1,
        errorCount: current.errorCount + 1,
        progressText: "Some session files could not be indexed"
      };
    case "finished":
      return completeSessionIndexState(current, event.data);
  }
}

/**
 * Builds the complete session index state used by scan and session progress UI.
 */
export function completeSessionIndexState(
  current: SessionIndexState,
  summary: SessionIndexSummary
): SessionIndexState {
  const errorCount = readCount(summary.errorCount, summary.error_count);

  return {
    ...current,
    status: "complete",
    filesProcessed: readCount(summary.filesProcessed, summary.files_processed),
    sessionsPersisted: readCount(summary.sessionsPersisted, summary.sessions_persisted),
    unmatchedCount: readCount(summary.unmatchedCount, summary.unmatched_count),
    errorCount,
    progressText: errorCount > 0 ? "Some session files could not be indexed" : "Session index updated"
  };
}

function sourceLabel(source: "claude" | "codex") {
  return source === "claude" ? "Claude Code" : "Codex";
}

function readCount(...values: Array<number | undefined>) {
  return values.find((value) => Number.isFinite(value)) ?? 0;
}
