import { useApi } from "../api";
import type { Alert } from "../types";
import { relativeTime } from "../format";
import { StatusCard } from "./StatusCard";

export function AlertsCard() {
  const q = useApi<Alert[]>("alerts", "/alerts");
  const alerts = q.data ?? [];
  return (
    <StatusCard
      title="Alerts"
      loading={q.isLoading}
      error={q.isError}
      empty={alerts.length === 0}
      emptyText="No active alerts"
    >
      <ul className="alerts">
        {alerts.map((a, i) => (
          <li className={`alert alert--${a.severity}`} key={a.id ?? `${a.code}-${i}`}>
            <span className="alert__code">{a.code}</span>
            <span className="alert__msg">{a.message}</span>
            <span className="alert__time">{relativeTime(a.timestamp)}</span>
          </li>
        ))}
      </ul>
    </StatusCard>
  );
}
