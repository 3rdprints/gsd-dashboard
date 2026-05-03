import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { openUrl, revealItemInDir } from "@tauri-apps/plugin-opener";

/**
 * Copies a project next-command string to the system clipboard.
 */
export async function copyNextCommand(command: string): Promise<void> {
  await writeText(command);
}

/**
 * Opens a project directory in the operating system file manager.
 */
export async function openProjectInFinder(rootPath: string): Promise<void> {
  await revealItemInDir(rootPath);
}

/**
 * Opens a project directory in VS Code through the configured URI handler.
 */
export async function openProjectInVsCode(rootPath: string): Promise<void> {
  const encodedPath = rootPath.replace(/\\/g, "/").split("/").map(encodeURIComponent).join("/");
  await openUrl(`vscode://file/${encodedPath}`);
}
