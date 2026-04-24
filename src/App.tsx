import { BootStatus } from "./components/BootStatus";
import { ScanRootsEditor } from "./components/ScanRootsEditor";

export function App() {
  return (
    <main className="app-shell">
      <div className="foundation-layout" aria-labelledby="app-title">
        <div className="app-header">
          <header>
            <h1 id="app-title">GSD Dashboard</h1>
            <p>No projects scanned yet</p>
          </header>
        </div>

        <div className="foundation-grid">
          <BootStatus />
          <ScanRootsEditor />
        </div>

        <section className="empty-state" aria-labelledby="empty-title">
          <h2 id="empty-title">No projects scanned yet</h2>
          <p>
            GSD Dashboard is initialized with ~/Documents as the default scan root. Project
            discovery starts in the next phase.
          </p>
        </section>
      </div>
    </main>
  );
}
