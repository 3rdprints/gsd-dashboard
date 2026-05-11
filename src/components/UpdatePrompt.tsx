import { AlertTriangle, CheckCircle2, Download, Loader2, RefreshCw, ShieldAlert } from "lucide-react";
import { useEffect, useState } from "react";

import { Button } from "./ui/button";
import {
  checkForUpdate,
  getCurrentVersion,
  installAndRestart,
  UPDATE_CHECK_FAILED_MESSAGE,
  UPDATE_INSTALL_FAILED_MESSAGE,
  type UpdateCheckState
} from "../lib/update";

type AvailableUpdateState = Extract<UpdateCheckState, { state: "available" }>;
type UpdatePanelState =
  | { state: "checking" }
  | { state: "installing"; update: AvailableUpdateState["update"] }
  | { state: "restart_ready"; update: AvailableUpdateState["update"] }
  | UpdateCheckState;

/**
 * Renders the Settings update panel and coordinates manual update actions.
 */
export function UpdatePrompt() {
  const [panelState, setPanelState] = useState<UpdatePanelState>({ state: "up_to_date" });
  const [currentVersion, setCurrentVersion] = useState<string | null>(null);

  useEffect(() => {
    void getCurrentVersion().then(setCurrentVersion);
  }, []);

  async function handleCheckForUpdates() {
    setPanelState({ state: "checking" });
    try {
      const updateState = await checkForUpdate();
      setPanelState(updateState.state === "unsupported" ? { state: "up_to_date" } : updateState);
    } catch (error) {
      console.error("Update check failed", error);
      setPanelState({
        state: "error",
        message: UPDATE_CHECK_FAILED_MESSAGE
      });
    }
  }

  async function handleInstall(update: AvailableUpdateState["update"]) {
    setPanelState({ state: "installing", update });
    try {
      await installAndRestart(update);
      setPanelState({ state: "restart_ready", update });
    } catch (error) {
      console.error("Update install failed", error);
      setPanelState({
        state: "error",
        message: UPDATE_INSTALL_FAILED_MESSAGE
      });
    }
  }

  function handleLater() {
    setPanelState({ state: "up_to_date" });
  }

  const isChecking = panelState.state === "checking";
  const isInstalling = panelState.state === "installing";
  const isFailure = panelState.state === "error" || panelState.state === "signature_error";

  return (
    <section className="settings-panel update-panel" aria-labelledby="update-panel-title">
      <div className="panel-heading">
        {panelState.state === "signature_error" ? (
          <ShieldAlert aria-hidden="true" size={20} strokeWidth={2} />
        ) : isFailure ? (
          <AlertTriangle aria-hidden="true" size={20} strokeWidth={2} />
        ) : isChecking || isInstalling ? (
          <Loader2 aria-hidden="true" size={20} strokeWidth={2} />
        ) : panelState.state === "available" ? (
          <Download aria-hidden="true" size={20} strokeWidth={2} />
        ) : (
          <CheckCircle2 aria-hidden="true" size={20} strokeWidth={2} />
        )}
        <div>
          <p className="label-text">Updates</p>
          <h2 id="update-panel-title">{getHeading(panelState)}</h2>
          <p className="update-version">Current version: {currentVersion ?? "Unavailable"}</p>
        </div>
      </div>

      <div
        className={`update-status ${getStatusClass(panelState)}`}
        role={isFailure ? "status" : undefined}
        aria-live={isFailure || isChecking || isInstalling ? "polite" : undefined}
      >
        <p>{getBody(panelState, currentVersion)}</p>
      </div>

      <div className="update-actions">
        {panelState.state === "available" ? (
          <>
            <Button type="button" onClick={() => void handleInstall(panelState.update)}>
              <Download aria-hidden="true" size={16} strokeWidth={2} />
              Install Update
            </Button>
            <Button type="button" variant="outline" onClick={handleLater}>
              Later
            </Button>
          </>
        ) : panelState.state === "restart_ready" ? (
          <>
            <Button type="button" onClick={() => void installAndRestart(panelState.update)}>
              <RefreshCw aria-hidden="true" size={16} strokeWidth={2} />
              Restart Now
            </Button>
            <Button type="button" variant="outline" onClick={handleLater}>
              Later
            </Button>
          </>
        ) : isFailure ? (
          <Button type="button" variant="outline" onClick={() => void handleCheckForUpdates()}>
            <RefreshCw aria-hidden="true" size={16} strokeWidth={2} />
            Try Again
          </Button>
        ) : (
          <Button type="button" variant="outline" onClick={() => void handleCheckForUpdates()} disabled={isChecking || isInstalling}>
            {isChecking || isInstalling ? (
              <Loader2 aria-hidden="true" size={16} strokeWidth={2} />
            ) : (
              <RefreshCw aria-hidden="true" size={16} strokeWidth={2} />
            )}
            Check for Updates
          </Button>
        )}
      </div>
    </section>
  );
}

function getHeading(panelState: UpdatePanelState) {
  switch (panelState.state) {
    case "available":
      return "Update available";
    case "checking":
      return "Checking for updates";
    case "installing":
      return "Installing update";
    case "restart_ready":
      return "Restart to finish update";
    case "error":
      return "Update check failed";
    case "signature_error":
      return "Update verification failed";
    case "unsupported":
      return "Updates unavailable";
    case "up_to_date":
    default:
      return "GSD Dashboard is up to date";
  }
}

function getBody(panelState: UpdatePanelState, currentVersion: string | null) {
  switch (panelState.state) {
    case "checking":
      return "Checking for updates";
    case "available":
      return currentVersion
        ? `Version ${currentVersion} -> ${panelState.version} is ready. Install it now or keep using this version.`
        : `Version ${panelState.version} is ready. Install it now or keep using this version.`;
    case "installing":
      return "Installing update";
    case "restart_ready":
      return "Update installed. Restart when you are ready.";
    case "error":
      return (
        panelState.message ||
        "Update check failed. The dashboard will keep running on this version; check your network or try again later."
      );
    case "signature_error":
      return panelState.message || "Update could not be verified. The dashboard will stay on the current version.";
    case "unsupported":
    case "up_to_date":
    default:
      return "You are running the latest stable version. Automatic checks will keep looking in the background.";
  }
}

function getStatusClass(panelState: UpdatePanelState) {
  switch (panelState.state) {
    case "error":
      return "warning";
    case "signature_error":
      return "danger";
    case "available":
      return "available";
    default:
      return "neutral";
  }
}
