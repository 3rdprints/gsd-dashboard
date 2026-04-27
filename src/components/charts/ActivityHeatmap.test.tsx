import "@testing-library/jest-dom/vitest";
import { render, screen } from "@testing-library/react";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

describe("ActivityHeatmap", () => {
  it("maps session counts to the six heatmap buckets", async () => {
    const { heatmapClassForValue } = await import("./ActivityHeatmap");

    expect(heatmapClassForValue(undefined)).toBe("heatmap-cell-0");
    expect(heatmapClassForValue({ date: "2026-04-01", count: 1 })).toBe("heatmap-cell-1");
    expect(heatmapClassForValue({ date: "2026-04-02", count: 2 })).toBe("heatmap-cell-2");
    expect(heatmapClassForValue({ date: "2026-04-03", count: 4 })).toBe("heatmap-cell-3");
    expect(heatmapClassForValue({ date: "2026-04-04", count: 8 })).toBe("heatmap-cell-4");
    expect(heatmapClassForValue({ date: "2026-04-05", count: 15 })).toBe("heatmap-cell-5");
  });

  it("formats accessible tooltip titles with project fallback and token totals", async () => {
    const { heatmapTitleForValue } = await import("./ActivityHeatmap");

    expect(
      heatmapTitleForValue({
        date: "2026-04-27",
        count: 2,
        tokenTotal: 12400,
        topProjectName: "GSD Dashboard"
      })
    ).toBe("2 sessions · GSD Dashboard · 12,400 tokens");
    expect(
      heatmapTitleForValue({
        date: "2026-04-26",
        count: 1,
        tokenTotal: 0,
        topProjectName: null
      })
    ).toBe("1 session · unattributed · 0 tokens");
  });

  it("renders an all-zero grid as bucket-0 cells", async () => {
    const { ActivityHeatmap } = await import("./ActivityHeatmap");
    const days = Array.from({ length: 90 }, (_, index) => ({
      date: `2026-01-${String(index + 1).padStart(2, "0")}`,
      sessionCount: 0,
      tokenTotal: 0,
      topProjectId: null,
      topProjectName: null
    }));

    render(<ActivityHeatmap days={days} endDate={new Date("2026-03-31T12:00:00Z")} />);

    expect(screen.getByRole("region", { name: "Activity heatmap for the last 90 days" })).toBeInTheDocument();
    expect(document.querySelector(".heatmap-cell-1")).not.toBeInTheDocument();
    expect(document.querySelectorAll(".heatmap-cell-0").length).toBeGreaterThanOrEqual(90);
  });

  it("does not import the package stylesheet", () => {
    const source = readFileSync(join(process.cwd(), "src/components/charts/ActivityHeatmap.tsx"), "utf8");
    const forbiddenImport = "react-calendar-heatmap/dist/" + "styles.css";

    expect(source).not.toContain(forbiddenImport);
  });
});
