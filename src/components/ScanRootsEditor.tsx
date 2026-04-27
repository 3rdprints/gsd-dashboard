import { FormEvent, useEffect, useRef, useState } from "react";
import { CheckCircle2, FolderOpen, Plus, Save, X } from "lucide-react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { getSettings } from "../lib/ipc";
import {
  createSaveSettingsMutationOptions,
  settingsQueryKey
} from "../lib/queryClient";
import type { AppError, SettingsInput } from "../lib/types";

const INVALID_SCAN_ROOT_MESSAGE =
  "This scan root is too broad. Choose a specific folder inside your home directory, such as ~/Documents or a project workspace.";
const DEFAULT_SCAN_ROOT = "~/Documents";
const DEFAULT_SETTINGS_INPUT: SettingsInput = {
  scanRoots: [DEFAULT_SCAN_ROOT],
  hiddenProjectIds: [],
  autostartEnabled: false,
  trayBarMaxProjects: 8,
  trayBarSort: "recent_activity",
  globalSessionsDefaultRange: "7d"
};

type ScanRootsEditorProps = {
  title?: string;
};

export function ScanRootsEditor({ title = "Settings" }: ScanRootsEditorProps) {
  const queryClient = useQueryClient();
  const settings = useQuery({
    queryKey: settingsQueryKey,
    queryFn: getSettings
  });
  const saveSettings = useMutation(createSaveSettingsMutationOptions(queryClient));
  const [scanRootDrafts, setScanRootDrafts] = useState<string[]>([DEFAULT_SCAN_ROOT]);
  const [hasLoadedSettings, setHasLoadedSettings] = useState(false);
  const hasEditedDrafts = useRef(false);
  const [hasSavedSettings, setHasSavedSettings] = useState(false);
  const rejectedScanRoot = parseRejectedScanRoot(saveSettings.error);
  const saveErrorMessage = getSaveErrorMessage(saveSettings.error);
  const settingsInput = settings.data ?? DEFAULT_SETTINGS_INPUT;
  const canSaveSettings = Boolean(settings.data) || settings.isError;

  useEffect(() => {
    if (settings.data && !hasLoadedSettings && !hasEditedDrafts.current) {
      setScanRootDrafts(
        settings.data.scanRoots.length > 0 ? settings.data.scanRoots : [DEFAULT_SCAN_ROOT]
      );
      setHasLoadedSettings(true);
    }
  }, [hasLoadedSettings, settings.data]);

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    saveSettings.mutate(
      {
        ...settingsInput,
        scanRoots: normalizeScanRootDrafts(scanRootDrafts)
      },
      {
        onSuccess: () => setHasSavedSettings(true),
        onError: () => setHasSavedSettings(false)
      }
    );
  }

  return (
    <section className="settings-panel" aria-labelledby="scan-roots-title">
      <div className="panel-heading">
        <FolderOpen aria-hidden="true" size={20} strokeWidth={2} />
        <div>
          <p className="label-text">Default scan root</p>
          <h2 id="scan-roots-title">{title}</h2>
        </div>
      </div>

      <form className="scan-root-form" onSubmit={handleSubmit}>
        <div className="scan-root-list">
          {scanRootDrafts.map((scanRootDraft, index) => {
            const inputId = `scan-root-${index}`;
            return (
              <div className="scan-root-row" key={inputId}>
                <label className="field-label" htmlFor={inputId}>
                  {index === 0 ? "Default scan root" : `Scan root ${index + 1}`}
                </label>
                <div className="control-row">
                  <input
                    id={inputId}
                    aria-label={index === 0 ? "Scan root 1" : `Scan root ${index + 1}`}
                    value={scanRootDraft}
                    onChange={(event) => {
                      setScanRootDrafts((current) =>
                        current.map((root, rootIndex) =>
                          rootIndex === index ? event.target.value : root
                        )
                      );
                      hasEditedDrafts.current = true;
                      setHasSavedSettings(false);
                    }}
                  />
                  <button
                    className="secondary-button"
                    type="button"
                    onClick={() => {
                      setScanRootDrafts((current) =>
                        current.length > 1
                          ? current.filter((_root, rootIndex) => rootIndex !== index)
                          : [DEFAULT_SCAN_ROOT]
                      );
                      hasEditedDrafts.current = true;
                      setHasSavedSettings(false);
                    }}
                  >
                    <X aria-hidden="true" size={16} strokeWidth={2} />
                    Remove Root
                  </button>
                </div>
              </div>
            );
          })}
        </div>

        <div className="settings-actions">
          <button
            className="secondary-button"
            type="button"
            onClick={() => {
              setScanRootDrafts((current) => [...normalizeScanRootDrafts(current), ""]);
              hasEditedDrafts.current = true;
              setHasSavedSettings(false);
            }}
          >
            <Plus aria-hidden="true" size={16} strokeWidth={2} />
            Add Root
          </button>
          <button type="submit" disabled={!canSaveSettings || saveSettings.isPending}>
            <Save aria-hidden="true" size={16} strokeWidth={2} />
            Save Settings
          </button>
        </div>
      </form>

      {rejectedScanRoot ? (
        <div className="scan-root-error" role="alert">
          <p>{INVALID_SCAN_ROOT_MESSAGE}</p>
          <p>Rejected path: {rejectedScanRoot.path}</p>
        </div>
      ) : null}

      {!rejectedScanRoot && saveSettings.isError ? (
        <div className="scan-root-error" role="alert">
          <p>{saveErrorMessage}</p>
        </div>
      ) : null}

      {hasSavedSettings && !saveSettings.isError ? (
        <div className="settings-saved">
          <CheckCircle2 aria-hidden="true" size={16} strokeWidth={2} />
          <span>Settings saved</span>
        </div>
      ) : null}
    </section>
  );
}

function getSaveErrorMessage(error: unknown) {
  if (error && typeof error === "object" && "message" in error) {
    const message = (error as { message?: unknown }).message;
    if (typeof message === "string" && message.length > 0) {
      return message;
    }
  }

  return "Settings could not be saved. Open the Tauri app window and try again.";
}

function normalizeScanRootDrafts(scanRootDrafts: string[]) {
  const roots = scanRootDrafts.map((root) => root.trim()).filter(Boolean);

  return roots.length > 0 ? roots : [DEFAULT_SCAN_ROOT];
}

function parseRejectedScanRoot(error: unknown): AppError | null {
  if (!error || typeof error !== "object") {
    return null;
  }

  const appError = error as Partial<AppError>;

  if (appError.kind !== "invalidScanRoot" || !appError.path) {
    return null;
  }

  return {
    kind: "invalidScanRoot",
    message: appError.message ?? INVALID_SCAN_ROOT_MESSAGE,
    path: appError.path,
    reason: appError.reason
  };
}
