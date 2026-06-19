import { useApi } from "../api";
import type { TpmsResponse } from "../types";
import { fmt, kpaToPsi } from "../format";
import { StatusCard } from "./StatusCard";

export function TpmsCard() {
  const q = useApi<TpmsResponse>("tpms", "/tpms");
  const sensors = q.data?.sensors ?? [];
  return (
    <StatusCard
      title="Tire Pressure"
      stale={q.data?.stale}
      loading={q.isLoading}
      error={q.isError}
      empty={sensors.length === 0}
    >
      <div className="tpms-grid">
        {sensors.map((s) => (
          <div className={`tpms${s.alarm ? " tpms--alarm" : ""}`} key={s.position}>
            <div className="tpms__pos">{s.position.replace(/_/g, " ")}</div>
            <div className="tpms__psi">
              {fmt(kpaToPsi(s.pressure_kpa), 0)}
              <span className="metric__unit">psi</span>
            </div>
            <div className="tpms__temp">{fmt(s.temp_c, 0)}°C</div>
          </div>
        ))}
      </div>
    </StatusCard>
  );
}
