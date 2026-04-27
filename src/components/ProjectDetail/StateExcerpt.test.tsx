import "@testing-library/jest-dom/vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { StateExcerpt } from "./StateExcerpt";

const openProjectInVsCodeMock = vi.fn(() => Promise.resolve());

vi.mock("../../lib/actions", () => ({
  openProjectInVsCode: openProjectInVsCodeMock
}));

describe("StateExcerpt", () => {
  it("renders literal script text without injecting raw HTML", () => {
    render(
      <StateExcerpt
        statePath="/Users/smacdonald/homegit/gsd-dashboard/.planning/STATE.md"
        excerpt={"## Current Position\n<script>alert('xss')</script>\nPlan: 8 of 12"}
      />
    );

    expect(screen.getByText("<script>alert('xss')</script>")).toBeInTheDocument();
    expect(document.querySelector("script")).toBeNull();
  });

  it("wires Open STATE.md to the existing opener action", async () => {
    render(
      <StateExcerpt
        statePath="/Users/smacdonald/homegit/gsd-dashboard/.planning/STATE.md"
        excerpt={"## Current Position\nPhase: 05"}
      />
    );

    fireEvent.click(screen.getByRole("button", { name: "Open STATE.md" }));
    expect(openProjectInVsCodeMock).toHaveBeenCalledWith(
      "/Users/smacdonald/homegit/gsd-dashboard/.planning/STATE.md"
    );
  });
});
