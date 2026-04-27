import { FormEvent, useEffect, useState } from "react";
import { CheckCircle2, FolderOpen, Plus, Save, X } from "lucide-react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { getSettings } from "../lib/ipc";
import {
  createSaveSettingsMutationOptions,
  settingsQueryKey
} from "../lib/queryClient";
import type { AppError } from "../lib/types";

const INVALID_SCAN_ROOT_MESSAGE =
  "This scan root is too broad. Choose a specific folder inside your home directory, such as ~/Documents or a project workspace.";

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
  const [scanRootDrafts, setScanRootDrafts] = useState<string[]>([]);
  const [hasSavedSettings, setHasSavedSettings] = useState(false);
  const rejectedScanRoot = parseRejectedScanRoot(saveSettings.error);

  useEffect(() => {
    if (settings.data && scanRootDrafts.length === 0) {
      setScanRootDrafts(settings.data.scanRoots.length > 0 ? settings.data.scanRoots : [""]);
    }
  }, [scanRootDrafts.length, settings.data]);

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    if (!settings.data) {
      return;
    }

    saveSettings.mutate(
      {
        ...settings.data,
        scanRoots: scanRootDrafts.map((root) => root.trim()).filter(Boolean)
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
                      setHasSavedSettings(false);
                    }}
                    disabled={!settings.data}
                  />
                  <button
                    className="secondary-button"
                    type="button"
                    onClick={() => {
                      setScanRootDrafts((current) =>
                        current.length > 1
                          ? current.filter((_root, rootIndex) => rootIndex !== index)
                          : [""]
                      );
                      setHasSavedSettings(false);
                    }}
                    disabled={!settings.data}
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
              setScanRootDrafts((current) => [...current, ""]);
              setHasSavedSettings(false);
            }}
            disabled={!settings.data}
          >
            <Plus aria-hidden="true" size={16} strokeWidth={2} />
            Add Root
          </button>
          <button type="submit" disabled={!settings.data || saveSettings.isPending}>
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

      {hasSavedSettings && !saveSettings.isError ? (
        <div className="settings-saved">
          <CheckCircle2 aria-hidden="true" size={16} strokeWidth={2} />
          <span>Settings saved</span>
        </div>
      ) : null}
    </section>
  );
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
