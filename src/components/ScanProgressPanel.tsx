import { AlertTriangle, CheckCircle2, Loader2, Search } from "lucide-react";

import type { ScanEvent, ScanSummary } from "../lib/types";

export type ScanRunStatus = "ready" | "scanning" | "complete" | "failed";

export type ScanState = {
  status: ScanRunStatus;
  discoveredCount: number;
  errorCount: number;
  progressText: string;
  firstParseError: {
    projectName: string;
    filePath: string;
  } | null;
};

export const initialScanState: ScanState = {
  status: "ready",
  discoveredCount: 0,
  errorCount: 0,
  progressText: "Ready to scan",
  firstParseError: null
};

type ScanProgressPanelProps = {
  state: ScanState;
};

/**
 * Renders the scan progress panel.
 */
export function ScanProgressPanel({ state }: ScanProgressPanelProps) {
  const isScanning = state.status === "scanning";
  const completedWithErrors = state.status === "complete" && state.errorCount > 0;
  const failed = state.status === "failed";
  const progressPercent = isScanning ? 55 : state.status === "complete" ? 100 : 0;

  return (
    <section className="scan-status" aria-labelledby="scan-status-title">
      <div className="panel-heading">
        {failed || completedWithErrors ? (
          <AlertTriangle aria-hidden="true" size={20} strokeWidth={2} />
        ) : state.status === "complete" ? (
          <CheckCircle2 aria-hidden="true" size={20} strokeWidth={2} />
        ) : isScanning ? (
          <Loader2 aria-hidden="true" size={20} strokeWidth={2} />
        ) : (
          <Search aria-hidden="true" size={20} strokeWidth={2} />
        )}
        <div>
          <p className="label-text">Scan status</p>
          <h2 id="scan-status-title">
            {failed
              ? "Scan failed"
              : completedWithErrors
                ? "Scan completed with parse errors"
                : state.status === "complete"
                  ? "Scan complete"
                  : isScanning
                    ? "Scanning projects"
                    : "Ready to scan"}
          </h2>
        </div>
      </div>

      <div className="scan-progress">
        <div
          className="scan-progress-track"
          role="progressbar"
          aria-valuemin={0}
          aria-valuemax={100}
          aria-valuenow={progressPercent}
          aria-valuetext={state.progressText}
        >
          <div
            className="scan-progress-fill"
            style={{ width: `${progressPercent}%` }}
          />
        </div>
        <p aria-live="polite">{state.progressText}</p>
      </div>

      {failed ? (
        <div className="parse-error-alert" role="alert">
          <p>Scan could not start. Check that the configured scan root exists and is allowed.</p>
        </div>
      ) : state.firstParseError ? (
        <div className="parse-error-alert" role="alert">
          <p>
            Some planning files could not be parsed. Scanning continued; review the affected
            project and file.
          </p>
          <p>
            {state.firstParseError.projectName} · {state.firstParseError.filePath}
          </p>
        </div>
      ) : null}
    </section>
  );
}

/**
 * Reduces incoming progress events into scan event.
 */
export function reduceScanEvent(current: ScanState, event: ScanEvent): ScanState {
  switch (event.event) {
    case "started":
      return {
        ...initialScanState,
        status: "scanning",
        progressText: "Walking scan roots"
      };
    case "rootStarted":
      return {
        ...current,
        status: "scanning",
        progressText: "Walking scan roots"
      };
    case "projectFound":
      return {
        ...current,
        status: "scanning",
        discoveredCount: current.discoveredCount + 1,
        progressText: `Found ${event.data.projectName}`
      };
    case "projectParsed":
      return {
        ...current,
        status: "scanning",
        progressText: "Saving project snapshot"
      };
    case "projectParseError":
      return {
        ...current,
        status: "scanning",
        errorCount: current.errorCount + 1,
        progressText: `Parsing ${event.data.projectName}`,
        firstParseError:
          current.firstParseError ?? {
            projectName: event.data.projectName,
            filePath: event.data.filePath
          }
      };
    case "finished":
      return completeScanState(current, event.data);
  }
}

type ScanSummaryPayload = ScanSummary & {
  discovered_count?: number;
  projectCount?: number;
  project_count?: number;
  parsed_count?: number;
  error_count?: number;
};

/**
 * Builds the complete scan state used by scan and session progress UI.
 */
export function completeScanState(current: ScanState, summary: ScanSummaryPayload): ScanState {
  const discoveredCount = readCount(
    summary.discoveredCount,
    summary.discovered_count,
    summary.projectCount,
    summary.project_count,
    current.discoveredCount
  );
  const errorCount = readCount(summary.errorCount, summary.error_count);

  return {
    ...current,
    status: "complete",
    discoveredCount,
    errorCount,
    firstParseError: errorCount > 0 ? current.firstParseError : null,
    progressText: errorCount > 0 ? "Scan completed with parse errors" : "Scan complete"
  };
}

function readCount(...values: Array<number | undefined>) {
  return values.find((value) => Number.isFinite(value)) ?? 0;
}
