import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import "@testing-library/jest-dom/vitest";
import { render, screen } from "@testing-library/react";
import type { ReactNode } from "react";
import { describe, expect, it, vi } from "vitest";

import { SettingsPage } from "./SettingsPage";
import * as ipc from "../lib/ipc";
import type { PortfolioDto, WatcherStatus } from "../lib/types";

vi.mock("../components/ScanRootsEditor", () => ({
  ScanRootsEditor: ({ title }: { title: string }) => (
    <section aria-labelledby="scan-roots-title">
      <h2 id="scan-roots-title">{title}</h2>
    </section>
  )
}));

const watcherStatus: WatcherStatus = {
  roots: [
    {
      root: "/Users/smacdonald/projects/example/.planning",
      mode: "polling",
      reasonCategory: "watchLimit",
      fixHint: "Increase fs.inotify.max_user_watches for this system.",
      pollingIntervalSeconds: 60,
      retryEnabled: false
    }
  ]
};

const portfolio: PortfolioDto = {
  stats: {
    projectsTracked: 0,
    activeMilestones: 0,
    sessionsToday: 0,
    tokensToday: 0
  },
  projects: [],
  hiddenProjects: [],
  unmatchedSessions: {
    count: 0,
    label: "No unmatched sessions",
    claudeCount: 0,
    codexCount: 0,
    recent: []
  }
};

describe("SettingsPage live update watcher status", () => {
  it("groups live status with support panels before maintenance controls", async () => {
    vi.spyOn(ipc, "getSettings").mockResolvedValue({
      scanRoots: [],
      hiddenProjectIds: [],
      trayHiddenProjectIds: [],
      autostartEnabled: false,
      trayBarMaxProjects: 4,
      trayBarSort: "name",
      globalSessionsDefaultRange: "7d"
    });
    vi.spyOn(ipc, "getPortfolio").mockResolvedValue(portfolio);
    vi.spyOn(ipc, "getWatcherStatus").mockResolvedValue(watcherStatus);

    renderWithClient(<SettingsPage />);

    expect(await screen.findByText("Live updates are using polling")).toBeInTheDocument();
    expect(screen.getByText("/Users/smacdonald/projects/example/.planning")).toBeInTheDocument();
    expect(screen.getByText("System watch limit reached")).toBeInTheDocument();
    expect(screen.getByText("Polling every 60s")).toBeInTheDocument();
    expect(screen.getByText("GSD Dashboard is up to date")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Check for Updates" })).toBeInTheDocument();
    expect(screen.queryByText(/Auto-retry/)).not.toBeInTheDocument();
    expect(screen.getByRole("status")).toHaveTextContent("Live updates are using polling");

    const sectionHeadings = screen.getAllByRole("heading", { level: 2 }).map((heading) => heading.textContent);
    expect(sectionHeadings.indexOf("Watcher Status")).toBeGreaterThan(sectionHeadings.indexOf("Scan roots"));
    expect(sectionHeadings.indexOf("Watcher Status")).toBeLessThan(sectionHeadings.indexOf("Hidden projects"));
    expect(sectionHeadings.indexOf("GSD Dashboard is up to date")).toBeGreaterThan(sectionHeadings.indexOf("Hidden projects"));
    expect(sectionHeadings.indexOf("GSD Dashboard is up to date")).toBeLessThan(sectionHeadings.indexOf("Rebuild Cache"));
    expect(sectionHeadings.indexOf("Scan status")).toBeLessThan(sectionHeadings.indexOf("Rebuild Cache"));
    expect(sectionHeadings.indexOf("Rebuild Cache")).toBeLessThan(sectionHeadings.indexOf("Indexing"));
  });
});

function renderWithClient(children: ReactNode) {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false }
    }
  });

  return render(<QueryClientProvider client={queryClient}>{children}</QueryClientProvider>);
}
