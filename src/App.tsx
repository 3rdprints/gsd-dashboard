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

const navLinkClassName = ({ isActive }: { isActive: boolean }) =>
  isActive ? "nav-link active" : "nav-link";

type AppFrameProps = {
  onThemeModeChange: ReturnType<typeof useThemeMode>["setThemeMode"];
  themeMode: ReturnType<typeof useThemeMode>["themeMode"];
};

/**
 * Renders the application shell with routing and shared query state.
 */
export const App = () => {
  const { setThemeMode, themeMode } = useThemeMode();

  useEffect(() => registerAppListeners(), []);

  return (
    <BrowserRouter>
      <AppFrame themeMode={themeMode} onThemeModeChange={setThemeMode} />
    </BrowserRouter>
  );
};

const AppFrame = ({ onThemeModeChange, themeMode }: AppFrameProps) => (
  <main className="app-shell">
    <div className="app-layout">
      <AppNavigation themeMode={themeMode} onThemeModeChange={onThemeModeChange} />
      <AppRoutes />
    </div>
  </main>
);

const AppNavigation = ({ onThemeModeChange, themeMode }: AppFrameProps) => (
  <nav className="app-nav" aria-label="Main">
    <div className="app-brand" aria-label="GSD Dashboard">
      <span aria-hidden="true" className="app-brand-mark" />
      <span>GSD</span>
    </div>
    <NavLink to="/" end className={navLinkClassName}>
      <FolderKanban aria-hidden="true" size={15} strokeWidth={2.2} />
      Portfolio
    </NavLink>
    <NavLink to="/sessions" className={navLinkClassName}>
      <BarChart3 aria-hidden="true" size={15} strokeWidth={2.2} />
      Sessions
    </NavLink>
    <NavLink to="/settings" className={navLinkClassName}>
      <Settings aria-hidden="true" size={15} strokeWidth={2.2} />
      Settings
    </NavLink>
    <ThemeToggle themeMode={themeMode} onThemeModeChange={onThemeModeChange} />
  </nav>
);

const AppRoutes = () => (
  <Routes>
    <Route path="/" element={<PortfolioPage />} />
    <Route path="/project/:id" element={<ProjectDetailPage />} />
    <Route path="/sessions" element={<GlobalSessionsPage />} />
    <Route path="/settings" element={<SettingsPage />} />
  </Routes>
);
