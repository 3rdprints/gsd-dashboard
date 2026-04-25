import { BrowserRouter, Link, Route, Routes } from "react-router-dom";

import { PortfolioPage } from "./routes/PortfolioPage";
import { ProjectDetailPage } from "./routes/ProjectDetailPage";
import { SettingsPage } from "./routes/SettingsPage";

export function App() {
  return (
    <BrowserRouter>
      <main className="app-shell">
        <div className="app-layout">
          <nav className="app-nav" aria-label="Main">
            <Link to="/">Portfolio</Link>
            <Link to="/settings">Settings</Link>
          </nav>
          <Routes>
            <Route path="/" element={<PortfolioPage />} />
            <Route path="/project/:id" element={<ProjectDetailPage />} />
            <Route path="/settings" element={<SettingsPage />} />
          </Routes>
        </div>
      </main>
    </BrowserRouter>
  );
}
