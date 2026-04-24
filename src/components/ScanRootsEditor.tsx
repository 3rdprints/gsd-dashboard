import { FormEvent, useEffect, useState } from "react";
import { CheckCircle2, FolderOpen, Save } from "lucide-react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { getSettings } from "../lib/ipc";
import {
  createSaveSettingsMutationOptions,
  settingsQueryKey
} from "../lib/queryClient";

export function ScanRootsEditor() {
  const queryClient = useQueryClient();
  const settings = useQuery({
    queryKey: settingsQueryKey,
    queryFn: getSettings
  });
  const saveSettings = useMutation(createSaveSettingsMutationOptions(queryClient));
  const [scanRootDraft, setScanRootDraft] = useState("");
  const [hasSavedSettings, setHasSavedSettings] = useState(false);

  useEffect(() => {
    if (settings.data && scanRootDraft === "") {
      setScanRootDraft(settings.data.scanRoots[0] ?? "");
      setHasSavedSettings(true);
    }
  }, [scanRootDraft, settings.data]);

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    if (!settings.data) {
      return;
    }

    saveSettings.mutate(
      {
        ...settings.data,
        scanRoots: [scanRootDraft]
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
          <h2 id="scan-roots-title">Settings</h2>
        </div>
      </div>

      <form className="scan-root-form" onSubmit={handleSubmit}>
        <label className="field-label" htmlFor="default-scan-root">
          Default scan root
        </label>
        <div className="control-row">
          <input
            id="default-scan-root"
            value={scanRootDraft}
            onChange={(event) => {
              setScanRootDraft(event.target.value);
              setHasSavedSettings(false);
            }}
            disabled={!settings.data}
          />
          <button type="submit" disabled={!settings.data || saveSettings.isPending}>
            <Save aria-hidden="true" size={16} strokeWidth={2} />
            Save Settings
          </button>
        </div>
      </form>

      {hasSavedSettings && !saveSettings.isError ? (
        <div className="settings-saved">
          <CheckCircle2 aria-hidden="true" size={16} strokeWidth={2} />
          <span>Settings saved</span>
        </div>
      ) : null}
    </section>
  );
}
