import { useApi } from "../api";
import type { SystemHealth } from "../types";
import { BatteryCard } from "../components/BatteryCard";
import { TanksCard } from "../components/TanksCard";
import { TpmsCard } from "../components/TpmsCard";
import { AlertsCard } from "../components/AlertsCard";
import { NodesCard } from "../components/NodesCard";

export function Dashboard() {
  // The health query doubles as the API-connection probe for the banner.
  const health = useApi<SystemHealth>("health", "/health");
  const apiBase = import.meta.env.VITE_API_BASE ?? "http://localhost:8080/api/v1";

  return (
    <>
      {health.isError && (
        <div className="banner banner--err">
          Backend unreachable — is the API running at {apiBase}?
        </div>
      )}
      <div className="grid">
        <BatteryCard />
        <TanksCard />
        <TpmsCard />
        <AlertsCard />
        <NodesCard />
      </div>
    </>
  );
}
