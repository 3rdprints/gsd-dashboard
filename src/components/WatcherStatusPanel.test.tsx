import "@testing-library/jest-dom/vitest";
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { WatcherStatusPanel } from "./WatcherStatusPanel";
import type { WatcherStatus } from "../lib/types";

describe("WatcherStatusPanel", () => {
  it("renders compact success copy for all native roots", () => {
    render(
      <WatcherStatusPanel
        status={{
          roots: [
            {
              root: "/Users/smacdonald/projects/example/.planning",
              mode: "native",
              retryEnabled: true
            }
          ]
        }}
        isLoading={false}
        isError={false}
      />
    );

    expect(screen.getByText("Live updates active")).toBeInTheDocument();
    expect(screen.getByText("All watched roots are using native file updates.")).toBeInTheDocument();
    expect(screen.queryByRole("status")).not.toBeInTheDocument();
  });

  it("renders degraded roots as a status banner with normalized copy", () => {
    const status: WatcherStatus = {
      roots: [
        {
          root: "/workspace/a/.planning",
          mode: "polling",
          reasonCategory: "permission",
          fixHint: "Grant folder access and reopen the app.",
          pollingIntervalSeconds: 60,
          retryEnabled: true
        },
        {
          root: "/workspace/b/.planning",
          mode: "polling",
          reasonCategory: "filesystem",
          fixHint: "Move the project to a local folder.",
          pollingIntervalSeconds: 60,
          retryEnabled: true
        }
      ]
    };

    render(<WatcherStatusPanel status={status} isLoading={false} isError={false} />);

    expect(screen.getByRole("status")).toHaveTextContent("Live updates are using polling");
    expect(screen.getByText("2 roots are being checked every 60 seconds.")).toBeInTheDocument();
    expect(screen.getByText("Permission denied")).toBeInTheDocument();
    expect(screen.getByText("Filesystem does not support native watching")).toBeInTheDocument();
    expect(screen.getAllByText("Polling every 60s")).toHaveLength(2);
    expect(screen.getAllByText("Auto-retry on")).toHaveLength(2);
  });

  it("renders loading and error states accessibly", () => {
    const { rerender } = render(<WatcherStatusPanel isLoading={true} isError={false} />);

    expect(screen.getByText("Loading live update status")).toBeInTheDocument();

    rerender(<WatcherStatusPanel isLoading={false} isError={true} />);

    expect(screen.getByRole("alert")).toHaveTextContent(
      "Live update status could not be loaded. Reopen Settings or rebuild the cache and try again."
    );
  });
});
