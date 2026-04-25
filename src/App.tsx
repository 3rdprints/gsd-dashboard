import { useState } from "react";
import { AlertTriangle, CheckCircle2, Loader2, Search } from "lucide-react";

import { BootStatus } from "./components/BootStatus";
import { ScanRootsEditor } from "./components/ScanRootsEditor";
import { scanProjects } from "./lib/ipc";
import type { ScanEvent, ScanSummary } from "./lib/types";

type ScanRunStatus = "ready" | "scanning" | "complete" | "failed";

type ParseErrorSummary = {
  projectName: string;
  filePath: string;
};

type ScanState = {
  status: ScanRunStatus;
  discoveredCount: number;
  errorCount: number;
  progressText: string;
  firstParseError: ParseErrorSummary | null;
};

const initialScanState: ScanState = {
  status: "ready",
  discoveredCount: 0,
  errorCount: 0,
  progressText: "Ready to scan",
  firstParseError: null
};

export function App() {
  const [scanState, setScanState] = useState<ScanState>(initialScanState);
  const isScanning = scanState.status === "scanning";
  const scanCompletedWithErrors = scanState.status === "complete" && scanState.errorCount > 0;
  const scanFailed = scanState.status === "failed";
  const headerSubtitle = formatHeaderSubtitle(scanState);

  async function handleScanProjects() {
    setScanState({
      ...initialScanState,
      status: "scanning",
      progressText: "Walking scan roots"
    });

    try {
      const summary = await scanProjects(handleScanEvent);
      setScanState((current) => completeScanState(current, summary));
    } catch {
      setScanState((current) => ({
        ...current,
        status: "failed",
        progressText: "Scan failed"
      }));
    }
  }

  function handleScanEvent(event: ScanEvent) {
    setScanState((current) => reduceScanEvent(current, event));
  }

  return (
    <main className="app-shell">
      <div className="foundation-layout" aria-labelledby="app-title">
        <div className="app-header">
          <header>
            <h1 id="app-title">GSD Dashboard</h1>
            <p>{headerSubtitle}</p>
          </header>
          <button
            className="scan-cta"
            type="button"
            onClick={handleScanProjects}
            disabled={isScanning}
          >
            {isScanning ? (
              <Loader2 aria-hidden="true" size={16} strokeWidth={2} />
            ) : (
              <Search aria-hidden="true" size={16} strokeWidth={2} />
            )}
            Scan Projects
          </button>
        </div>

        <div className="foundation-grid">
          <BootStatus />
          <ScanRootsEditor />
        </div>

        <section className="scan-status" aria-labelledby="scan-status-title">
          <div className="panel-heading">
            {scanFailed || scanCompletedWithErrors ? (
              <AlertTriangle aria-hidden="true" size={20} strokeWidth={2} />
            ) : scanState.status === "complete" ? (
              <CheckCircle2 aria-hidden="true" size={20} strokeWidth={2} />
            ) : isScanning ? (
              <Loader2 aria-hidden="true" size={20} strokeWidth={2} />
            ) : (
              <Search aria-hidden="true" size={20} strokeWidth={2} />
            )}
            <div>
              <p className="label-text">Scan status</p>
              <h2 id="scan-status-title">
                {scanFailed
                  ? "Scan failed"
                  : scanCompletedWithErrors
                  ? "Scan completed with parse errors"
                  : scanState.status === "complete"
                    ? "Scan complete"
                    : isScanning
                      ? "Scanning projects"
                      : "Ready to scan"}
              </h2>
            </div>
          </div>

          <div className="scan-progress">
            <div className="scan-progress-track" aria-hidden="true">
              <div
                className="scan-progress-fill"
                style={{ width: isScanning ? "55%" : scanState.status === "complete" ? "100%" : "0%" }}
              />
            </div>
            <p aria-live="polite">{scanState.progressText}</p>
          </div>

          {scanFailed ? (
            <div className="parse-error-alert" role="alert">
              <p>Scan could not start. Check that the configured scan root exists and is allowed.</p>
            </div>
          ) : scanState.firstParseError ? (
            <div className="parse-error-alert" role="alert">
              <p>
                Some planning files could not be parsed. Scanning continued; open the scan details
                to review the affected project and file.
              </p>
              <p>
                {scanState.firstParseError.projectName} · {scanState.firstParseError.filePath}
              </p>
            </div>
          ) : null}
        </section>

        <section className="empty-state" aria-labelledby="empty-title">
          <h2 id="empty-title">No projects scanned yet</h2>
          <p>
            GSD Dashboard is ready to scan your configured roots. Start a scan to discover projects
            with `.planning/` directories.
          </p>
        </section>
      </div>
    </main>
  );
}

function reduceScanEvent(current: ScanState, event: ScanEvent): ScanState {
  switch (event.event) {
    case "started":
      return {
        ...current,
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

function completeScanState(current: ScanState, summary: ScanSummary): ScanState {
  const errorCount = Math.max(summary.errorCount, current.errorCount);

  return {
    ...current,
    status: "complete",
    discoveredCount: Math.max(summary.discoveredCount, current.discoveredCount),
    errorCount,
    progressText: errorCount > 0 ? "Scan completed with parse errors" : "Scan complete"
  };
}

function formatHeaderSubtitle(scanState: ScanState) {
  if (scanState.status === "scanning") {
    return "Scanning projects";
  }

  if (scanState.status === "failed") {
    return "Scan failed";
  }

  if (scanState.status === "complete" || scanState.discoveredCount > 0) {
    const projectText = `${scanState.discoveredCount} projects discovered`;
    return scanState.errorCount > 0
      ? `${projectText} · ${scanState.errorCount} parse errors`
      : projectText;
  }

  return "Ready to scan";
}
