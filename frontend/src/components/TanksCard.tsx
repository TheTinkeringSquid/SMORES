import { useApi, useHistory } from "../api";
import type { Tank, TanksResponse } from "../types";
import { fmt } from "../format";
import { StatusCard } from "./StatusCard";
import { Sparkline } from "./Sparkline";

function TankRow({ tank }: { tank: Tank }) {
  const hist = useHistory("tanks", `level_percent:${tank.id}`);
  const trend = hist.data?.map((p) => p.value) ?? [];
  const pct = Math.max(0, Math.min(100, tank.level_percent));
  return (
    <div className="bar">
      <div className="bar__head">
        <span>{tank.name ?? tank.kind}</span>
        <span>{fmt(tank.level_percent, 0)}%</span>
      </div>
      <div className="bar__track">
        <div className={`bar__fill bar__fill--${tank.kind}`} style={{ width: `${pct}%` }} />
      </div>
      {trend.length > 1 && (
        <div className={`spark-wrap spark-wrap--${tank.kind}`}>
          <Sparkline values={trend} height={22} />
        </div>
      )}
    </div>
  );
}

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
        {tanks.map((t) => (
          <TankRow tank={t} key={t.id} />
        ))}
      </div>
    </StatusCard>
  );
}
