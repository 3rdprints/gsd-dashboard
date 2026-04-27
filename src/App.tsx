import { useEffect } from "react";
import { BrowserRouter, Link, Route, Routes } from "react-router-dom";

import { registerAppListeners } from "./lib/appListeners";
import { PortfolioPage } from "./routes/PortfolioPage";
import { GlobalSessionsPage } from "./routes/GlobalSessionsPage";
import { ProjectDetailPage } from "./routes/ProjectDetailPage";
import { SettingsPage } from "./routes/SettingsPage";

export function App() {
  useEffect(() => registerAppListeners(), []);

  return (
    <BrowserRouter>
      <main className="app-shell">
        <div className="app-layout">
          <nav className="app-nav" aria-label="Main">
            <Link to="/">Portfolio</Link>
            <Link to="/sessions">Sessions</Link>
            <Link to="/settings">Settings</Link>
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
