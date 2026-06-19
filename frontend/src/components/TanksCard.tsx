import { useApi } from "../api";
import type { TanksResponse } from "../types";
import { fmt } from "../format";
import { StatusCard } from "./StatusCard";

export function TanksCard() {
  const q = useApi<TanksResponse>("tanks", "/tanks");
  const tanks = q.data?.tanks ?? [];
  return (
    <StatusCard
      title="Tanks"
      stale={q.data?.stale}
      loading={q.isLoading}
      error={q.isError}
      empty={tanks.length === 0}
    >
      <div className="bars">
        {tanks.map((t) => {
          const pct = Math.max(0, Math.min(100, t.level_percent));
          return (
            <div className="bar" key={t.id}>
              <div className="bar__head">
                <span>{t.name ?? t.kind}</span>
                <span>{fmt(t.level_percent, 0)}%</span>
              </div>
              <div className="bar__track">
                <div className={`bar__fill bar__fill--${t.kind}`} style={{ width: `${pct}%` }} />
              </div>
            </div>
          );
        })}
      </div>
    </StatusCard>
  );
}
