import { CheckCircle2, Database, FolderOpen, Settings2 } from "lucide-react";

export function App() {
  return (
    <main className="app-shell">
      <section className="foundation-panel" aria-labelledby="app-title">
        <div className="app-header">
          <div>
            <h1 id="app-title">GSD Dashboard</h1>
            <p>No projects scanned yet</p>
          </div>
          <span className="ready-badge">
            <CheckCircle2 aria-hidden="true" size={16} strokeWidth={2} />
            Settings saved
          </span>
        </div>

        <div className="status-grid" aria-label="Foundation status">
          <article className="status-card">
            <Database aria-hidden="true" size={20} strokeWidth={2} />
            <p className="status-label">Frontend</p>
            <h2>Cache ready</h2>
            <p>Migrations applied</p>
          </article>
          <article className="status-card">
            <FolderOpen aria-hidden="true" size={20} strokeWidth={2} />
            <p className="status-label">Default scan root</p>
            <h2>~/Documents</h2>
            <p>Project discovery starts in the next phase.</p>
          </article>
          <article className="status-card status-card-wide">
            <Settings2 aria-hidden="true" size={20} strokeWidth={2} />
            <p className="status-label">Guardrail</p>
            <h2>Broad roots refused</h2>
            <p>
              This scan root is too broad. Choose a specific folder inside your
              home directory, such as ~/Documents or a project workspace.
            </p>
          </article>
        </div>
      </section>
    </main>
  );
}
