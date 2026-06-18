import { useApi } from "../api";
import type { BatteryResponse } from "../types";
import { fmt } from "../format";
import { StatusCard } from "./StatusCard";
import { Metric } from "./Metric";

export function BatteryCard() {
  const q = useApi<BatteryResponse>("battery", "/battery");
  const b = q.data?.battery ?? null;
  return (
    <StatusCard
      title="House Battery"
      source={b?.source}
      stale={b?.stale}
      loading={q.isLoading}
      error={q.isError}
      empty={!b}
    >
      {b && (
        <>
          <Metric label="State of charge" value={fmt(b.soc_percent, 1)} unit="%" big />
          <div className="metric-row">
            <Metric label="Voltage" value={fmt(b.voltage_v, 2)} unit="V" />
            <Metric label="Current" value={fmt(b.current_a, 1)} unit="A" />
            <Metric label="Temp" value={fmt(b.temp_c, 0)} unit="°C" />
          </div>
          <span className="chip">{b.charging_state ?? "unknown"}</span>
        </>
      )}
    </StatusCard>
  );
}
