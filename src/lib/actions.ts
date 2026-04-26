import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { openUrl, revealItemInDir } from "@tauri-apps/plugin-opener";

export async function copyNextCommand(command: string): Promise<void> {
  await writeText(command);
}

export async function openProjectInFinder(rootPath: string): Promise<void> {
  await revealItemInDir(rootPath);
}

export async function openProjectInVsCode(rootPath: string): Promise<void> {
  const encodedPath = rootPath.replace(/\\/g, "/").split("/").map(encodeURIComponent).join("/");
  await openUrl(`vscode://file/${encodedPath}`);
}
