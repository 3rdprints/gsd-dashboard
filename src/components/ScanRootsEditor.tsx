import { FormEvent, useEffect, useRef, useState } from "react";
import { CheckCircle2, FolderOpen, PanelTop, Plus, Save, X } from "lucide-react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { getPortfolio, getSettings } from "../lib/ipc";
import {
  createSaveSettingsMutationOptions,
  portfolioQueryKey,
  settingsQueryKey
} from "../lib/queryClient";
import type { AppError, SettingsInput, TrayBarSort } from "../lib/types";

const INVALID_SCAN_ROOT_MESSAGE =
  "This scan root is too broad. Choose a specific folder inside your home directory, such as ~/Documents or a project workspace.";
const DEFAULT_SCAN_ROOT = "~/Documents";
const DEFAULT_SETTINGS_INPUT: SettingsInput = {
  scanRoots: [DEFAULT_SCAN_ROOT],
  hiddenProjectIds: [],
  trayHiddenProjectIds: [],
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
  const portfolio = useQuery({
    queryKey: portfolioQueryKey,
    queryFn: getPortfolio
  });
  const saveSettings = useMutation(createSaveSettingsMutationOptions(queryClient));
  const [scanRootDrafts, setScanRootDrafts] = useState<string[]>([DEFAULT_SCAN_ROOT]);
  const [trayBarMaxProjects, setTrayBarMaxProjects] = useState(
    DEFAULT_SETTINGS_INPUT.trayBarMaxProjects
  );
  const [trayBarSort, setTrayBarSort] = useState<TrayBarSort>(DEFAULT_SETTINGS_INPUT.trayBarSort);
  const [trayHiddenProjectIds, setTrayHiddenProjectIds] = useState<string[]>(
    DEFAULT_SETTINGS_INPUT.trayHiddenProjectIds
  );
  const [hasLoadedSettings, setHasLoadedSettings] = useState(false);
  const hasEditedDrafts = useRef(false);
  const hasEditedTraySettings = useRef(false);
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
      if (!hasEditedTraySettings.current) {
        setTrayBarMaxProjects(settings.data.trayBarMaxProjects);
        setTrayBarSort(settings.data.trayBarSort);
        setTrayHiddenProjectIds(settings.data.trayHiddenProjectIds);
      }
      setHasLoadedSettings(true);
    }
  }, [hasLoadedSettings, settings.data]);

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    saveSettings.mutate(
      {
        ...settingsInput,
        scanRoots: normalizeScanRootDrafts(scanRootDrafts),
        trayBarMaxProjects: clampTrayBarMaxProjects(trayBarMaxProjects),
        trayBarSort,
        trayHiddenProjectIds
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

        <section className="scan-root-row" aria-labelledby="tray-display-title">
          <div className="panel-heading">
            <PanelTop aria-hidden="true" size={20} strokeWidth={2} />
            <div>
              <p className="label-text">Menu bar</p>
              <h2 id="tray-display-title">Tray Display</h2>
            </div>
          </div>

          <div className="scan-root-list">
            <div className="scan-root-row">
              <label className="field-label" htmlFor="tray-bar-max-projects">
                Max tray bars
              </label>
              <div className="control-row">
                <input
                  id="tray-bar-max-projects"
                  aria-label="Max tray bars"
                  type="number"
                  min={1}
                  max={16}
                  value={trayBarMaxProjects}
                  onChange={(event) => {
                    setTrayBarMaxProjects(Number(event.target.value));
                    hasEditedTraySettings.current = true;
                    setHasSavedSettings(false);
                  }}
                />
              </div>
            </div>

            <fieldset className="scan-root-row">
              <legend className="field-label">Sort order</legend>
              <div className="control-row">
                {TRAY_SORT_OPTIONS.map((option) => (
                  <label className="field-label" key={option.value}>
                    <input
                      type="radio"
                      name="tray-bar-sort"
                      value={option.value}
                      checked={trayBarSort === option.value}
                      onChange={() => {
                        setTrayBarSort(option.value);
                        hasEditedTraySettings.current = true;
                        setHasSavedSettings(false);
                      }}
                    />
                    {option.label}
                  </label>
                ))}
              </div>
            </fieldset>

            <div className="scan-root-row">
              <div className="field-label">Projects shown in tray</div>
              <div className="scan-root-list">
                {(portfolio.data?.projects ?? []).map((project) => (
                  <label className="field-label" key={project.id}>
                    <input
                      type="checkbox"
                      checked={!trayHiddenProjectIds.includes(project.id)}
                      onChange={(event) => {
                        setTrayHiddenProjectIds((current) =>
                          event.target.checked
                            ? current.filter((projectId) => projectId !== project.id)
                            : [...new Set([...current, project.id])]
                        );
                        hasEditedTraySettings.current = true;
                        setHasSavedSettings(false);
                      }}
                    />
                    {project.name}
                  </label>
                ))}
              </div>
            </div>
          </div>
        </section>

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
      if (isMissingTauriBridgeMessage(message)) {
        return "Settings can only be saved from the Tauri desktop app. The browser preview can edit the form, but it cannot persist settings.";
      }

      return message;
    }
  }

  return "Settings could not be saved. Open the Tauri app window and try again.";
}

function isMissingTauriBridgeMessage(message: string) {
  const lowerMessage = message.toLowerCase();

  return lowerMessage.includes("invoke") || lowerMessage.includes("__tauri");
}

function normalizeScanRootDrafts(scanRootDrafts: string[]) {
  const roots = scanRootDrafts.map((root) => root.trim()).filter(Boolean);

  return roots.length > 0 ? roots : [DEFAULT_SCAN_ROOT];
}

const TRAY_SORT_OPTIONS: { label: string; value: TrayBarSort }[] = [
  { label: "Recent activity", value: "recent_activity" },
  { label: "Progress", value: "progress" },
  { label: "Name", value: "name" }
];

function clampTrayBarMaxProjects(value: number) {
  if (!Number.isFinite(value)) {
    return DEFAULT_SETTINGS_INPUT.trayBarMaxProjects;
  }

  return Math.min(16, Math.max(1, Math.trunc(value)));
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
