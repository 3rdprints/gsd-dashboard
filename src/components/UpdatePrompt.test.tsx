import "@testing-library/jest-dom/vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { UpdatePrompt } from "./UpdatePrompt";
import { checkForUpdate } from "../lib/update";

vi.mock("../lib/update", () => ({
  checkForUpdate: vi.fn(),
  installAndRestart: vi.fn()
}));

describe("UpdatePrompt", () => {
  beforeEach(() => {
    vi.mocked(checkForUpdate).mockReset();
  });

  it("renders the quiet up-to-date state by default", () => {
    render(<UpdatePrompt />);

    expect(screen.getByText("GSD Dashboard is up to date")).toBeInTheDocument();
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
      update: { downloadAndInstall: vi.fn() }
    });
    render(<UpdatePrompt />);

    fireEvent.click(screen.getByRole("button", { name: "Check for Updates" }));

    expect(await screen.findByText("Update available")).toBeInTheDocument();
    expect(screen.getByText("Version 1.2.3 is ready. Install it now or keep using this version.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Install Update" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Later" })).toBeInTheDocument();
  });

  it("renders nonblocking error copy and retry action", async () => {
    vi.mocked(checkForUpdate).mockResolvedValue({
      state: "error",
      message:
        "Update check failed. The dashboard will keep running on this version; check your network or try again later."
    });
    render(<UpdatePrompt />);

    fireEvent.click(screen.getByRole("button", { name: "Check for Updates" }));

    expect(await screen.findByRole("status")).toHaveTextContent(
      "Update check failed. The dashboard will keep running on this version; check your network or try again later."
    );
    expect(screen.getByRole("button", { name: "Try Again" })).toBeInTheDocument();
  });
});
