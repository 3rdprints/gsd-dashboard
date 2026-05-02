import "@testing-library/jest-dom/vitest";
import { act, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useThemeMode } from "./theme";

let mediaQueryListener: ((event: MediaQueryListEvent) => void) | null = null;
let systemPrefersDark = false;

function ThemeProbe() {
  const { resolvedTheme, setThemeMode, themeMode } = useThemeMode();
  return (
    <div>
      <p>{`${themeMode}:${resolvedTheme}`}</p>
      <button type="button" onClick={() => setThemeMode("dark")}>
        Dark
      </button>
      <button type="button" onClick={() => setThemeMode("system")}>
        System
      </button>
    </div>
  );
}

describe("theme mode", () => {
  beforeEach(() => {
    const storage = new Map<string, string>();
    Object.defineProperty(window, "localStorage", {
      configurable: true,
      value: {
        getItem: vi.fn((key: string) => storage.get(key) ?? null),
        setItem: vi.fn((key: string, value: string) => storage.set(key, value)),
        clear: vi.fn(() => storage.clear())
      }
    });
    window.localStorage.clear();
    mediaQueryListener = null;
    systemPrefersDark = false;
    Object.defineProperty(window, "matchMedia", {
      configurable: true,
      value: vi.fn(
        () =>
          ({
            matches: systemPrefersDark,
            addEventListener: (_event: string, listener: (event: MediaQueryListEvent) => void) => {
              mediaQueryListener = listener;
            },
            removeEventListener: vi.fn()
          }) as unknown as MediaQueryList
      )
    });
  });

  it("defaults to system theme and follows system dark mode changes", () => {
    render(<ThemeProbe />);

    expect(screen.getByText("system:light")).toBeInTheDocument();
    systemPrefersDark = true;
    act(() => mediaQueryListener?.({ matches: true } as MediaQueryListEvent));

    expect(screen.getByText("system:dark")).toBeInTheDocument();
    expect(document.documentElement.dataset.colorScheme).toBe("dark");
  });
});
