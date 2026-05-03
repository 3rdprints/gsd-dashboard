import type { Update } from "@tauri-apps/plugin-updater";

export const UPDATE_CHECK_FAILED_MESSAGE =
  "Update check failed. The dashboard will keep running on this version; check your network or try again later.";
export const UPDATE_SIGNATURE_FAILED_MESSAGE =
  "Update could not be verified. The dashboard will stay on the current version.";

export type UpdateCheckState =
  | { state: "unsupported" }
  | { state: "up_to_date" }
  | { state: "available"; update: Update; version: string; body?: string }
  | { state: "error"; message: typeof UPDATE_CHECK_FAILED_MESSAGE }
  | { state: "signature_error"; message: typeof UPDATE_SIGNATURE_FAILED_MESSAGE };

export async function checkForUpdate(): Promise<UpdateCheckState> {
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
}

export async function installAndRestart(update: Pick<Update, "downloadAndInstall"> | null) {
  if (!update) {
    return;
  }

  await update.downloadAndInstall();
  const { relaunch } = await import("@tauri-apps/plugin-process");
  await relaunch();
}

function hasTauriInternals() {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

function isSignatureError(error: unknown) {
  const message = error instanceof Error ? error.message : String(error);

  return /signature|verif/i.test(message);
}
