import { CheckCircle2 } from "lucide-react";

export function App() {
  return (
    <main className="app-shell">
      <section className="status-panel" aria-labelledby="app-title">
        <div className="status-header">
          <CheckCircle2 aria-hidden="true" size={24} strokeWidth={2} />
          <div>
            <p className="status-label">Foundation scaffold</p>
            <h1 id="app-title">GSD Dashboard</h1>
          </div>
        </div>
        <div className="status-grid" aria-label="Foundation status">
          <div>
            <p className="status-label">Frontend</p>
            <p>React, TypeScript, Vite, and Tailwind are ready.</p>
          </div>
          <div>
            <p className="status-label">Native shell</p>
            <p>Tauri is configured for the desktop app foundation.</p>
          </div>
        </div>
      </section>
    </main>
  );
}
