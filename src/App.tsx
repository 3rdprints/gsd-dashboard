import { useEffect } from "react";
import { BarChart3, FolderKanban, Settings } from "lucide-react";
import { BrowserRouter, NavLink, Route, Routes } from "react-router-dom";

import { ThemeToggle } from "./components/ThemeToggle";
import { registerAppListeners } from "./lib/appListeners";
import { useThemeMode } from "./lib/theme";
import { PortfolioPage } from "./routes/PortfolioPage";
import { GlobalSessionsPage } from "./routes/GlobalSessionsPage";
import { ProjectDetailPage } from "./routes/ProjectDetailPage";
import { SettingsPage } from "./routes/SettingsPage";

/**
 * Renders the application shell with routing and shared query state.
 */
export function App() {
  const { setThemeMode, themeMode } = useThemeMode();

  useEffect(() => registerAppListeners(), []);

  return (
    <BrowserRouter>
      <main className="app-shell">
        <div className="app-layout">
          <nav className="app-nav" aria-label="Main">
            <div className="app-brand" aria-label="GSD Dashboard">
              <span aria-hidden="true" className="app-brand-mark" />
              <span>GSD</span>
            </div>
            <NavLink to="/" className={({ isActive }) => (isActive ? "nav-link active" : "nav-link")}>
              <FolderKanban aria-hidden="true" size={15} strokeWidth={2.2} />
              Portfolio
            </NavLink>
            <NavLink
              to="/sessions"
              className={({ isActive }) => (isActive ? "nav-link active" : "nav-link")}
            >
              <BarChart3 aria-hidden="true" size={15} strokeWidth={2.2} />
              Sessions
            </NavLink>
            <NavLink
              to="/settings"
              className={({ isActive }) => (isActive ? "nav-link active" : "nav-link")}
            >
              <Settings aria-hidden="true" size={15} strokeWidth={2.2} />
              Settings
            </NavLink>
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
