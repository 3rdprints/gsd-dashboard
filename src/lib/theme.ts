import { useEffect, useState } from "react";

export type ThemeMode = "light" | "dark" | "system";
type ResolvedTheme = "light" | "dark";

const THEME_STORAGE_KEY = "gsd-dashboard-theme";
const THEME_MEDIA_QUERY = "(prefers-color-scheme: dark)";

export function useThemeMode() {
  const [themeMode, setThemeModeState] = useState<ThemeMode>(() => storedThemeMode());
  const [systemTheme, setSystemTheme] = useState<ResolvedTheme>(() => currentSystemTheme());

  useEffect(() => {
    if (typeof window.matchMedia !== "function") {
      return;
    }

    const mediaQuery = window.matchMedia(THEME_MEDIA_QUERY);
    function handleChange(event?: MediaQueryListEvent) {
      setSystemTheme((event?.matches ?? mediaQuery.matches) ? "dark" : "light");
    }

    handleChange();
    mediaQuery.addEventListener("change", handleChange);
    return () => mediaQuery.removeEventListener("change", handleChange);
  }, []);

  const resolvedTheme = themeMode === "system" ? systemTheme : themeMode;

  useEffect(() => {
    document.documentElement.dataset.colorScheme = resolvedTheme;
    document.documentElement.classList.toggle("dark", resolvedTheme === "dark");
  }, [resolvedTheme]);

  function setThemeMode(nextThemeMode: ThemeMode) {
    storeThemeMode(nextThemeMode);
    setThemeModeState(nextThemeMode);
  }

  return { resolvedTheme, setThemeMode, themeMode };
}

function storedThemeMode(): ThemeMode {
  const stored = readStoredThemeMode();
  return isThemeMode(stored) ? stored : "system";
}

function readStoredThemeMode() {
  try {
    return typeof window.localStorage?.getItem === "function"
      ? window.localStorage.getItem(THEME_STORAGE_KEY)
      : null;
  } catch {
    return null;
  }
}

function storeThemeMode(themeMode: ThemeMode) {
  try {
    if (typeof window.localStorage?.setItem === "function") {
      window.localStorage.setItem(THEME_STORAGE_KEY, themeMode);
    }
  } catch {
    // Theme selection still works for the current window when storage is unavailable.
  }
}

function currentSystemTheme(): ResolvedTheme {
  if (typeof window.matchMedia !== "function") {
    return "light";
  }

  return window.matchMedia(THEME_MEDIA_QUERY).matches ? "dark" : "light";
}

function isThemeMode(value: string | null): value is ThemeMode {
  return value === "light" || value === "dark" || value === "system";
}
