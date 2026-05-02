import { useEffect } from "react";
import { BrowserRouter, Link, Route, Routes } from "react-router-dom";

import { ThemeToggle } from "./components/ThemeToggle";
import { registerAppListeners } from "./lib/appListeners";
import { useThemeMode } from "./lib/theme";
import { PortfolioPage } from "./routes/PortfolioPage";
import { GlobalSessionsPage } from "./routes/GlobalSessionsPage";
import { ProjectDetailPage } from "./routes/ProjectDetailPage";
import { SettingsPage } from "./routes/SettingsPage";

export function App() {
  const { setThemeMode, themeMode } = useThemeMode();

  useEffect(() => registerAppListeners(), []);

  return (
    <BrowserRouter>
      <main className="app-shell">
        <div className="app-layout">
          <nav className="app-nav" aria-label="Main">
            <Link to="/">Portfolio</Link>
            <Link to="/sessions">Sessions</Link>
            <Link to="/settings">Settings</Link>
            <ThemeToggle themeMode={themeMode} onThemeModeChange={setThemeMode} />
          </nav>
          <Routes>
            <Route path="/" element={<PortfolioPage />} />
            <Route path="/project/:id" element={<ProjectDetailPage />} />
            <Route path="/sessions" element={<GlobalSessionsPage />} />
            <Route path="/settings" element={<SettingsPage />} />
          </Routes>
        </div>
      </main>
    </BrowserRouter>
  );
}
