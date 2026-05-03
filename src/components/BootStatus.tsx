import { CheckCircle2, Database } from "lucide-react";
import { useQuery } from "@tanstack/react-query";

import { getBootStatus } from "../lib/ipc";
import { bootStatusQueryKey } from "../lib/queryClient";

/**
 * Provides the exported boot status function.
 */
export function BootStatus() {
  const bootStatus = useQuery({
    queryKey: bootStatusQueryKey,
    queryFn: getBootStatus
  });

  const cacheReady = bootStatus.data?.cacheReady === true;
  const migrationsApplied = (bootStatus.data?.migrationsApplied ?? 0) > 0;

  return (
    <section className="status-panel" aria-label="Boot status">
      <div className="panel-heading">
        <Database aria-hidden="true" size={20} strokeWidth={2} />
        <div>
          <p className="label-text">Cache</p>
          <h2>{cacheReady ? "Cache ready" : "Cache pending"}</h2>
        </div>
      </div>

      <div className="status-list">
        <StatusRow label={migrationsApplied ? "Migrations applied" : "Migrations pending"} />
        <StatusRow label={bootStatus.data?.walEnabled ? "WAL enabled" : "WAL pending"} />
      </div>
    </section>
  );
}

function StatusRow({ label }: { label: string }) {
  return (
    <div className="status-row">
      <CheckCircle2 aria-hidden="true" size={16} strokeWidth={2} />
      <span>{label}</span>
    </div>
  );
}
