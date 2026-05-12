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

const updateHeadingByState: Record<UpdatePanelState["state"], string> = {
  available: "Update available",
  checking: "Checking for updates",
  error: "Update check failed",
  installing: "Installing update",
  restart_ready: "Restart to finish update",
  signature_error: "Update verification failed",
  unsupported: "Updates unavailable",
  up_to_date: "GSD Dashboard is up to date"
};

type UpdateBodyFormatter = (panelState: UpdatePanelState, currentVersion: string | null) => string;

const stableVersionMessage = "You are running the latest stable version. Automatic checks will keep looking in the background.";

const staticBody = (message: string): UpdateBodyFormatter => () => message;

const updateBodyByState: Record<UpdatePanelState["state"], UpdateBodyFormatter> = {
  available: (panelState, currentVersion) => {
    const updateState = panelState as AvailableUpdateState;

    return currentVersion
      ? `Version ${currentVersion} -> ${updateState.version} is ready. Install it now or keep using this version.`
      : `Version ${updateState.version} is ready. Install it now or keep using this version.`;
  },
  checking: staticBody("Checking for updates"),
  error: (panelState) => {
    const errorState = panelState as Extract<UpdateCheckState, { state: "error" }>;

    return errorState.message || UPDATE_CHECK_FAILED_MESSAGE;
  },
  installing: staticBody("Installing update"),
  restart_ready: staticBody("Update installed. Restart when you are ready."),
  signature_error: (panelState) => {
    const signatureState = panelState as Extract<UpdateCheckState, { state: "signature_error" }>;

    return signatureState.message || "Update could not be verified. The dashboard will stay on the current version.";
  },
  unsupported: staticBody(stableVersionMessage),
  up_to_date: staticBody(stableVersionMessage)
};

const updateStatusClassByState: Partial<Record<UpdatePanelState["state"], string>> = {
  available: "available",
  error: "warning",
  signature_error: "danger"
};

const updateIconByState = {
  available: Download,
  checking: Loader2,
  error: AlertTriangle,
  installing: Loader2,
  restart_ready: CheckCircle2,
  signature_error: ShieldAlert,
  unsupported: CheckCircle2,
  up_to_date: CheckCircle2
} satisfies Record<UpdatePanelState["state"], typeof CheckCircle2>;

const failureStates = new Set<UpdatePanelState["state"]>(["error", "signature_error"]);
const pendingStates = new Set<UpdatePanelState["state"]>(["checking", "installing"]);

const getHeading = (panelState: UpdatePanelState) => updateHeadingByState[panelState.state];
const getBody = (panelState: UpdatePanelState, currentVersion: string | null) =>
  updateBodyByState[panelState.state](panelState, currentVersion);
const getStatusClass = (panelState: UpdatePanelState) => updateStatusClassByState[panelState.state] ?? "neutral";
const normalizeUpdateState = (updateState: UpdateCheckState): UpdateCheckState =>
  updateState.state === "unsupported" ? { state: "up_to_date" } : updateState;

const useUpdatePanelState = () => {
  const [panelState, setPanelState] = useState<UpdatePanelState>({ state: "up_to_date" });
  const [currentVersion, setCurrentVersion] = useState<string | null>(null);

  useEffect(() => {
    getCurrentVersion().then(setCurrentVersion).catch(() => setCurrentVersion(null));
  }, []);

  const handleCheckForUpdates = async () => {
    setPanelState({ state: "checking" });
    try {
      const updateState = await checkForUpdate();
      setPanelState(normalizeUpdateState(updateState));
    } catch (error) {
      console.error("Update check failed", error);
      setPanelState({
        state: "error",
        message: UPDATE_CHECK_FAILED_MESSAGE
      });
    }
  };

  const handleInstall = async (update: AvailableUpdateState["update"]) => {
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
  };

  const handleLater = () => {
    setPanelState({ state: "up_to_date" });
  };

  return {
    currentVersion,
    handleCheckForUpdates,
    handleInstall,
    handleLater,
    panelState
  };
};

type UpdateActionProps = {
  handleCheckForUpdates: () => Promise<unknown>;
  handleInstall: (update: AvailableUpdateState["update"]) => Promise<unknown>;
  handleLater: () => unknown;
  isFailure: boolean;
  isPending: boolean;
  panelState: UpdatePanelState;
};

const getActions = ({
  handleCheckForUpdates,
  handleInstall,
  handleLater,
  isFailure,
  isPending,
  panelState
}: UpdateActionProps) => {
  if (panelState.state === "available") {
    return (
      <>
        <Button type="button" onClick={() => handleInstall(panelState.update).catch(() => undefined)}>
          <Download aria-hidden="true" size={16} strokeWidth={2} />
          Install Update
        </Button>
        <Button type="button" variant="outline" onClick={handleLater}>
          Later
        </Button>
      </>
    );
  }

  if (panelState.state === "restart_ready") {
    return (
      <>
        <Button type="button" onClick={() => installAndRestart(panelState.update).catch(() => undefined)}>
          <RefreshCw aria-hidden="true" size={16} strokeWidth={2} />
          Restart Now
        </Button>
        <Button type="button" variant="outline" onClick={handleLater}>
          Later
        </Button>
      </>
    );
  }

  if (isFailure) {
    return (
      <Button type="button" variant="outline" onClick={() => handleCheckForUpdates().catch(() => undefined)}>
        <RefreshCw aria-hidden="true" size={16} strokeWidth={2} />
        Try Again
      </Button>
    );
  }

  const ActionIcon = isPending ? Loader2 : RefreshCw;

  return (
    <Button type="button" variant="outline" onClick={() => handleCheckForUpdates().catch(() => undefined)} disabled={isPending}>
      <ActionIcon aria-hidden="true" size={16} strokeWidth={2} />
      Check for Updates
    </Button>
  );
};

/**
 * Renders the Settings update panel and coordinates manual update actions.
 */
export const UpdatePrompt = () => {
  const { currentVersion, handleCheckForUpdates, handleInstall, handleLater, panelState } = useUpdatePanelState();
  const isFailure = failureStates.has(panelState.state);
  const isPending = pendingStates.has(panelState.state);
  const StatusIcon = updateIconByState[panelState.state];

  return (
    <section className="settings-panel update-panel" aria-labelledby="update-panel-title">
      <div className="panel-heading">
        <StatusIcon aria-hidden="true" size={20} strokeWidth={2} />
        <div>
          <p className="label-text">Updates</p>
          <h2 id="update-panel-title">{getHeading(panelState)}</h2>
          <p className="update-version">Current version: {currentVersion ?? "Unavailable"}</p>
        </div>
      </div>

      <div
        className={`update-status ${getStatusClass(panelState)}`}
        role={isFailure ? "status" : undefined}
        aria-live={isFailure || isPending ? "polite" : undefined}
      >
        <p>{getBody(panelState, currentVersion)}</p>
      </div>

      <div className="update-actions">
        {getActions({
          handleCheckForUpdates,
          handleInstall,
          handleLater,
          isFailure,
          isPending,
          panelState
        })}
      </div>
    </section>
  );
};
