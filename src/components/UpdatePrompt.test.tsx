import "@testing-library/jest-dom/vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { UpdatePrompt } from "./UpdatePrompt";
import { checkForUpdate, getCurrentVersion, installAndRestart } from "../lib/update";

vi.mock("../lib/update", () => ({
  UPDATE_CHECK_FAILED_MESSAGE:
    "Update check failed. The dashboard will keep running on this version; check your network or try again later.",
  UPDATE_INSTALL_FAILED_MESSAGE: "Update install failed. The dashboard will keep running on this version; try again later.",
  checkForUpdate: vi.fn(),
  getCurrentVersion: vi.fn(),
  installAndRestart: vi.fn()
}));

const renderUpdatePrompt = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false
      }
    }
  });

  return render(
    <QueryClientProvider client={queryClient}>
      <UpdatePrompt />
    </QueryClientProvider>
  );
};

describe("UpdatePrompt", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    vi.mocked(checkForUpdate).mockReset();
    vi.mocked(getCurrentVersion).mockReset();
    vi.mocked(installAndRestart).mockReset();
    vi.mocked(getCurrentVersion).mockResolvedValue("0.1.5");
  });

  it("renders the quiet up-to-date state by default", async () => {
    renderUpdatePrompt();

    expect(screen.getByText("GSD Dashboard is up to date")).toBeInTheDocument();
    expect(await screen.findByText("Current version: 0.1.5")).toBeInTheDocument();
    expect(
      screen.getByText(
        "You are running the latest stable version. Automatic checks will keep looking in the background."
      )
    ).toBeInTheDocument();
  });

  it("renders available update actions after a manual check", async () => {
    vi.mocked(checkForUpdate).mockResolvedValue({
      state: "available",
      version: "1.2.3",
      update: { downloadAndInstall: vi.fn() } as never
    });
    renderUpdatePrompt();

    fireEvent.click(screen.getByRole("button", { name: "Check for Updates" }));

    expect(await screen.findByText("Update available")).toBeInTheDocument();
    expect(
      screen.getByText("Version 0.1.5 -> 1.2.3 is ready. Install it now or keep using this version.")
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Install Update" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Later" })).toBeInTheDocument();
  });

  it("renders nonblocking error copy and retry action", async () => {
    vi.mocked(checkForUpdate).mockResolvedValue({
      state: "error",
      message:
        "Update check failed. The dashboard will keep running on this version; check your network or try again later."
    });
    renderUpdatePrompt();

    fireEvent.click(screen.getByRole("button", { name: "Check for Updates" }));

    expect(await screen.findByRole("status")).toHaveTextContent(
      "Update check failed. The dashboard will keep running on this version; check your network or try again later."
    );
    expect(screen.getByText("Update check failed")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Try Again" })).toBeInTheDocument();
  });

  it("recovers when update checks reject unexpectedly", async () => {
    vi.spyOn(console, "error").mockImplementation(() => undefined);
    vi.mocked(checkForUpdate).mockRejectedValue(new Error("network down"));
    renderUpdatePrompt();

    fireEvent.click(screen.getByRole("button", { name: "Check for Updates" }));

    expect(await screen.findByRole("status")).toHaveTextContent(
      "Update check failed. The dashboard will keep running on this version; check your network or try again later."
    );
    expect(screen.getByText("Update check failed")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Try Again" })).toBeInTheDocument();
  });

  it("recovers when install rejects unexpectedly", async () => {
    vi.spyOn(console, "error").mockImplementation(() => undefined);
    const update = { downloadAndInstall: vi.fn() } as never;
    vi.mocked(checkForUpdate).mockResolvedValue({
      state: "available",
      version: "1.2.3",
      update
    });
    vi.mocked(installAndRestart).mockRejectedValue(new Error("install failed"));
    renderUpdatePrompt();

    fireEvent.click(screen.getByRole("button", { name: "Check for Updates" }));
    fireEvent.click(await screen.findByRole("button", { name: "Install Update" }));

    expect(await screen.findByRole("status")).toHaveTextContent(
      "Update install failed. The dashboard will keep running on this version; try again later."
    );
    expect(screen.getByText("Installation failed")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Try Again" })).toBeInTheDocument();
  });
});
