import { Monitor, Moon, Sun } from "lucide-react";

import type { ThemeMode } from "../lib/theme";

type ThemeToggleProps = {
  themeMode: ThemeMode;
  onThemeModeChange: (themeMode: ThemeMode) => void;
};

const THEME_OPTIONS: Array<{ value: ThemeMode; label: string; icon: typeof Sun }> = [
  { value: "system", label: "System", icon: Monitor },
  { value: "light", label: "Light", icon: Sun },
  { value: "dark", label: "Dark", icon: Moon }
];

export function ThemeToggle({ onThemeModeChange, themeMode }: ThemeToggleProps) {
  return (
    <div className="theme-toggle" aria-label="Theme mode" role="group">
      {THEME_OPTIONS.map((option) => {
        const Icon = option.icon;
        return (
          <button
            aria-pressed={themeMode === option.value}
            className={themeMode === option.value ? "theme-toggle-active" : undefined}
            key={option.value}
            onClick={() => onThemeModeChange(option.value)}
            title={option.label}
            type="button"
          >
            <Icon aria-hidden="true" size={15} strokeWidth={2} />
            <span>{option.label}</span>
          </button>
        );
      })}
    </div>
  );
}
