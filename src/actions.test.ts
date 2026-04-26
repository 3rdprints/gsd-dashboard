import { beforeEach, describe, expect, it, vi } from "vitest";

const { invokeMock, openUrlMock, revealItemInDirMock, writeTextMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
  openUrlMock: vi.fn(),
  revealItemInDirMock: vi.fn(),
  writeTextMock: vi.fn()
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock
}));

vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({
  writeText: writeTextMock
}));

vi.mock("@tauri-apps/plugin-opener", () => ({
  openUrl: openUrlMock,
  revealItemInDir: revealItemInDirMock
}));

describe("safe action wrappers", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    openUrlMock.mockReset();
    revealItemInDirMock.mockReset();
    writeTextMock.mockReset();
  });

  it("copyNextCommand writes clipboard text without backend invoke", async () => {
    const { copyNextCommand } = await import("./lib/actions");

    writeTextMock.mockResolvedValueOnce(undefined);

    await copyNextCommand("/gsd-next");

    expect(writeTextMock).toHaveBeenCalledWith("/gsd-next");
    expect(invokeMock).not.toHaveBeenCalled();
  });

  it("reveals project paths through the official Tauri opener wrapper", async () => {
    const { openProjectInFinder } = await import("./lib/actions");
    const rootPath = "/Users/smacdonald/homegit/gsd-dashboard";

    revealItemInDirMock.mockResolvedValueOnce(undefined);

    await openProjectInFinder(rootPath);

    expect(revealItemInDirMock).toHaveBeenCalledWith(rootPath);
  });

  it("encodes VS Code file URLs without losing filesystem delimiters", async () => {
    const { openProjectInVsCode } = await import("./lib/actions");
    const rootPath = "/Users/smacdonald/homegit/project #1?draft";

    openUrlMock.mockResolvedValueOnce(undefined);

    await openProjectInVsCode(rootPath);

    expect(openUrlMock).toHaveBeenCalledWith(
      "vscode://file//Users/smacdonald/homegit/project%20%231%3Fdraft"
    );
  });
});
