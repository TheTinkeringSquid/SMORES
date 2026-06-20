import { useApi } from "../api";
import type { AppConfig, NodeView } from "../types";
import { fmt, kpaToPsi, relativeTime } from "../format";

/** Read-only view of the effective config: system name, alert thresholds, and
 *  the node registry merged with live status from /nodes. */
export function Settings() {
  const cfg = useApi<AppConfig>("config", "/config");
  const live = useApi<NodeView[]>("nodes", "/nodes");
  const c = cfg.data;
  const liveById = new Map((live.data ?? []).map((n) => [n.node_id, n]));

  return (
    <div className="settings">
      {cfg.isError && (
        <div className="banner banner--err">Backend unreachable — config unavailable.</div>
      )}

      <section className="card">
        <header className="card__head">
          <h2>System</h2>
        </header>
        <div className="card__body">
          {c ? (
            <ul className="kvlist">
              <li>
                <span>Name</span>
                <span>{c.system_name}</span>
              </li>
            </ul>
          ) : (
            <p className="muted">Loading…</p>
          )}
        </div>
      </section>

      <section className="card">
        <header className="card__head">
          <h2>Alert thresholds</h2>
        </header>
        <div className="card__body">
          {c ? (
            <ul className="kvlist">
              <li>
                <span>Low battery SOC</span>
                <span>&lt; {fmt(c.thresholds.low_soc_percent, 0)}%</span>
              </li>
              <li>
                <span>High gray/black tank</span>
                <span>&gt; {fmt(c.thresholds.high_tank_percent, 0)}%</span>
              </li>
              <li>
                <span>Low tire pressure</span>
                <span>
                  &lt; {fmt(c.thresholds.low_pressure_kpa, 0)} kPa (
                  {fmt(kpaToPsi(c.thresholds.low_pressure_kpa), 0)} psi)
                </span>
              </li>
            </ul>
          ) : (
            <p className="muted">Loading…</p>
          )}
        </div>
      </section>

      <section className="card settings__nodes">
        <header className="card__head">
          <h2>Node registry</h2>
        </header>
        <div className="card__body">
          {c && c.nodes.length > 0 ? (
            <table className="ntable">
              <thead>
                <tr>
                  <th aria-label="status" />
                  <th>Node</th>
                  <th>Subsystems</th>
                  <th>Stale after</th>
                  <th>Firmware</th>
                  <th>Last seen</th>
                </tr>
              </thead>
              <tbody>
                {c.nodes.map((n) => {
                  const l = liveById.get(n.id);
                  const online = l?.online ?? false;
                  return (
                    <tr key={n.id}>
                      <td>
                        <span className={`dot ${online ? "dot--ok" : "dot--off"}`} />
                      </td>
                      <td>
                        <div>{n.name ?? n.id}</div>
                        <div className="muted small">{n.id}</div>
                      </td>
                      <td>{n.subsystems.join(", ")}</td>
                      <td>{n.stale_after_seconds}s</td>
                      <td>{l?.firmware_version ?? "—"}</td>
                      <td>{l ? relativeTime(l.last_seen) : "—"}</td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          ) : (
            <p className="muted">No nodes configured (add some in smores.toml).</p>
          )}
        </div>
      </section>
    </div>
  );
}
