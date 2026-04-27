import "@testing-library/jest-dom/vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

describe("SessionsTable", () => {
  const rows = [
    {
      id: "session-1",
      source: "claude" as const,
      sourcePath: "/tmp/claude.jsonl",
      startedAt: 1_777_200_000_000,
      endedAt: 1_777_200_300_000,
      durationMs: 300_000,
      messageCount: 12,
      tokensIn: 1200,
      tokensOut: 3400,
      tokenTotal: 4600,
      model: "claude-sonnet"
    },
    {
      id: "session-2",
      source: "codex" as const,
      sourcePath: "/tmp/codex.jsonl",
      startedAt: null,
      endedAt: null,
      durationMs: null,
      messageCount: 3,
      tokensIn: 100,
      tokensOut: 250,
      tokenTotal: 350,
      model: null
    }
  ];

  it("renders sortable headers with aria-sort and toggles the active direction", async () => {
    const { SessionsTable } = await import("./SessionsTable");
    const onSortChange = vi.fn();

    render(
      <SessionsTable
        rows={rows}
        total={75}
        page={2}
        pageSize={50}
        sort="startedAt"
        direction="desc"
        onSortChange={onSortChange}
        onPageChange={vi.fn()}
      />
    );

    expect(screen.getByRole("columnheader", { name: /Date/ })).toHaveAttribute("aria-sort", "descending");
    expect(screen.getByRole("columnheader", { name: /Duration/ })).toHaveAttribute("aria-sort", "none");
    fireEvent.click(screen.getByRole("button", { name: /Date/ }));
    expect(onSortChange).toHaveBeenCalledWith("startedAt", "asc");
    fireEvent.click(screen.getByRole("button", { name: /Tokens Out/ }));
    expect(onSortChange).toHaveBeenCalledWith("tokensOut", "desc");
  });

  it("renders source badges, numeric cells, empty state, and prev-next pagination", async () => {
    const { SessionsTable } = await import("./SessionsTable");
    const onPageChange = vi.fn();

    const { rerender } = render(
      <SessionsTable
        rows={rows}
        total={75}
        page={2}
        pageSize={50}
        sort="startedAt"
        direction="desc"
        onSortChange={vi.fn()}
        onPageChange={onPageChange}
      />
    );

    expect(screen.getByText("Claude")).toBeInTheDocument();
    expect(screen.getByText("Codex")).toBeInTheDocument();
    expect(screen.getByText("5m")).toBeInTheDocument();
    expect(screen.getByText("3,400")).toBeInTheDocument();
    expect(screen.getByText("Page 2 of 2")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Previous page" }));
    expect(onPageChange).toHaveBeenCalledWith(1);
    expect(screen.getByRole("button", { name: "Next page" })).toBeDisabled();

    rerender(
      <SessionsTable
        rows={[]}
        total={0}
        page={1}
        pageSize={50}
        sort="startedAt"
        direction="desc"
        onSortChange={vi.fn()}
        onPageChange={vi.fn()}
      />
    );
    expect(screen.getByText("No sessions match the current filters.")).toBeInTheDocument();
  });
});
