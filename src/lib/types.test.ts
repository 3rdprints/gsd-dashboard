import { describe, expect, it } from "vitest";

import type { AppSettings, SettingsInput } from "./types";

describe("settings DTO contracts", () => {
  it("keeps tray visibility separate from portfolio hidden projects", () => {
    const settings: AppSettings = {
      scanRoots: ["~/Documents"],
      hiddenProjectIds: ["portfolio-hidden"],
      trayHiddenProjectIds: ["tray-hidden"],
      autostartEnabled: false,
      trayBarMaxProjects: 8,
      trayBarSort: "recent_activity",
      globalSessionsDefaultRange: "7d"
    };

    const input: SettingsInput = {
      ...settings,
      hiddenProjectIds: ["portfolio-hidden-updated"],
      trayHiddenProjectIds: ["tray-hidden-updated"]
    };

    expect(settings.hiddenProjectIds).toEqual(["portfolio-hidden"]);
    expect(settings.trayHiddenProjectIds).toEqual(["tray-hidden"]);
    expect(settings.autostartEnabled).toBe(false);
    expect(input.hiddenProjectIds).toEqual(["portfolio-hidden-updated"]);
    expect(input.trayHiddenProjectIds).toEqual(["tray-hidden-updated"]);
  });

  it("accepts launch on login input intent", () => {
    const input: SettingsInput = {
      scanRoots: ["~/Documents"],
      hiddenProjectIds: [],
      trayHiddenProjectIds: [],
      autostartEnabled: true,
      trayBarMaxProjects: 8,
      trayBarSort: "recent_activity",
      globalSessionsDefaultRange: "7d"
    };

    expect(input.autostartEnabled).toBe(true);
  });
});
