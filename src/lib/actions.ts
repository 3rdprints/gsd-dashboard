import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { openPath, openUrl } from "@tauri-apps/plugin-opener";

export async function copyNextCommand(command: string): Promise<void> {
  await writeText(command);
}

export async function openProjectInFinder(rootPath: string): Promise<void> {
  await openPath(rootPath);
}

export async function openProjectInVsCode(rootPath: string): Promise<void> {
  await openUrl(`vscode://file/${encodeURI(rootPath)}`);
}
