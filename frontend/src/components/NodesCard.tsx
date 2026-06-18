import { useApi } from "../api";
import type { NodeView } from "../types";
import { relativeTime } from "../format";
import { StatusCard } from "./StatusCard";

export function NodesCard() {
  const q = useApi<NodeView[]>("nodes", "/nodes");
  const nodes = q.data ?? [];
  return (
    <StatusCard
      title="Nodes"
      loading={q.isLoading}
      error={q.isError}
      empty={nodes.length === 0}
      emptyText="No nodes seen yet"
    >
      <ul className="nodes">
        {nodes.map((n) => (
          <li className="node" key={n.node_id}>
            <span className={`dot ${n.online ? "dot--ok" : "dot--off"}`} />
            <span className="node__id">{n.node_id}</span>
            <span className="node__subs">{n.subsystems.join(", ")}</span>
            <span className="node__time">{relativeTime(n.last_seen)}</span>
          </li>
        ))}
      </ul>
    </StatusCard>
  );
}
