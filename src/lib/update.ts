import type { Update } from "@tauri-apps/plugin-updater";

export const UPDATE_CHECK_FAILED_MESSAGE =
  "Update check failed. The dashboard will keep running on this version; check your network or try again later.";
export const UPDATE_INSTALL_FAILED_MESSAGE =
  "Update install failed. The dashboard will keep running on this version; try again later.";
export const UPDATE_SIGNATURE_FAILED_MESSAGE =
  "Update could not be verified. The dashboard will stay on the current version.";

export type UpdateCheckState =
  | { state: "unsupported" }
  | { state: "up_to_date" }
  | { state: "available"; update: Update; version: string; body?: string }
  | { state: "error"; message: string }
  | { state: "signature_error"; message: string };

const hasTauriInternals = () => {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
};

const isSignatureError = (error: unknown) => {
  const message = error instanceof Error ? error.message : String(error);

  return /signature|verif/i.test(message);
};

/**
 * Reads the current desktop app version from Tauri when running inside the app.
 */
export const getCurrentVersion = async () => {
  if (!hasTauriInternals()) {
    return null;
  }

  try {
    const { getVersion } = await import("@tauri-apps/api/app");
    return await getVersion();
  } catch {
    return null;
  }
};

/**
 * Checks the Tauri updater for an available release and normalizes the UI state.
 */
export const checkForUpdate = async (): Promise<UpdateCheckState> => {
  if (!hasTauriInternals()) {
    return { state: "unsupported" };
  }

  try {
    const { check } = await import("@tauri-apps/plugin-updater");
    const update = await check();

    if (!update) {
      return { state: "up_to_date" };
    }

    return {
      state: "available",
      update,
      version: update.version,
      body: update.body
    };
  } catch (error) {
    if (isSignatureError(error)) {
      return {
        state: "signature_error",
        message: UPDATE_SIGNATURE_FAILED_MESSAGE
      };
    }

    return {
      state: "error",
      message: UPDATE_CHECK_FAILED_MESSAGE
    };
  }
};

/**
 * Installs a downloaded update and restarts the desktop app through Tauri.
 */
export const installAndRestart = async (update: Pick<Update, "downloadAndInstall"> | null) => {
  if (!update) {
    return;
  }

  await update.downloadAndInstall();
  const { relaunch } = await import("@tauri-apps/plugin-process");
  await relaunch();
};
