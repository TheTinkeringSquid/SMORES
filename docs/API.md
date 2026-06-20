# S.M.O.R.E.S. REST API (v1)

Base URL: `http://<host>:8080/api/v1` (configurable via `SMORES_BIND`). All
responses are JSON. Types derive from [`SCHEMAS.md`](./SCHEMAS.md); the backend
returns typed views, each subsystem reading carrying `node_id`, `source`,
`timestamp`, and a computed `stale` flag.

> Read-only in Milestone 1–2. Command endpoints (actuators) come later and will
> be explicit, validated, logged, and fail-safe.

## Endpoints

### `GET /health`
System summary and liveness.
```json
{
  "status": "ok",
  "node_count": 3,
  "online_nodes": 3,
  "alert_count": 1,
  "subsystems_present": ["battery", "tanks", "tpms"]
}
```

### `GET /battery`
Latest house-battery reading, or `{ "battery": null }` if none yet.
```json
{
  "battery": {
    "id": "house-1", "name": "House Battery",
    "voltage_v": 12.7, "soc_percent": 78.2, "current_a": -8.4,
    "temp_c": 28.0, "charging_state": "discharging",
    "node_id": "battery-node-01", "source": "mock",
    "timestamp": "2026-06-18T20:00:00Z", "stale": false
  }
}
```

### `GET /tanks`
All tanks from the latest tank-node report.
```json
{
  "tanks": [
    { "id": "fresh-1", "name": "Fresh", "kind": "fresh", "level_percent": 64.0, "capacity_l": 150 }
  ],
  "node_id": "tank-node-01", "timestamp": "2026-06-18T20:00:00Z", "stale": false
}
```

### `GET /tpms`
All monitored wheels from the latest TPMS report. `pressure_kpa` is canonical
(the dashboard converts to psi).
```json
{
  "sensors": [
    { "position": "rear_left", "pressure_kpa": 448.0, "temp_c": 41.0, "alarm": true }
  ],
  "node_id": "tpms-node-01", "timestamp": "2026-06-18T20:00:00Z", "stale": false
}
```

### `GET /nodes`
The node registry with liveness (per-node stale windows from `smores.toml`).
```json
[
  {
    "node_id": "battery-node-01", "last_seen": "2026-06-18T20:00:00Z",
    "online": true, "subsystems": ["battery"],
    "firmware_version": "0.1.0-mock", "status": "online"
  }
]
```

### `GET /alerts`
Active alerts — both externally published (`smores/v1/system/alerts`) and
backend-derived by the alert engine.
```json
[
  {
    "schema": "smores.alert.v1", "id": "thr-low-soc", "severity": "warning",
    "subsystem": "battery", "code": "LOW_SOC",
    "message": "House battery SOC 48% below 55%", "active": true,
    "timestamp": "2026-06-18T20:00:00Z"
  }
]
```

### `GET /history`
Trend history for one subsystem+metric, sampled into SQLite every 10s. Returns
`[]` if history persistence is disabled (DB failed to open).

Query params: `subsystem` (default `battery`), `metric` (default `soc_percent`),
`since` (RFC 3339, default epoch), `limit` (default 500, max 5000). Metric keys:
`soc_percent` / `voltage_v` / `current_a` (battery), `level_percent:<tank_id>`
(tanks), `pressure_kpa:<position>` (tpms).

```
GET /history?subsystem=battery&metric=soc_percent&limit=200
```
```json
[
  { "ts": "2026-06-18T20:00:00Z", "value": 78.2 },
  { "ts": "2026-06-18T20:00:10Z", "value": 77.9 }
]
```

### `GET /stream`
Server-sent events. Emits an `update` event whose `data` is the key of the
subsystem that just changed (`battery` / `tanks` / `tpms` / `nodes` / `alerts`).
The dashboard refetches the matching data on receipt; the 15s query poll is the
fallback if the stream drops.

```
GET /stream          (Content-Type: text/event-stream)

event: update
data: battery
```

## Alert engine

The backend derives alerts from the latest readings and node liveness, raising
while the condition holds and clearing automatically when it resolves. Thresholds
live in `smores.toml` under `[thresholds]`.

| Code | Severity | Condition |
|------|----------|-----------|
| `LOW_SOC` | warning | battery `soc_percent` < `low_soc_percent` |
| `HIGH_TANK` | warning | gray/black tank `level_percent` > `high_tank_percent` |
| `LOW_PRESSURE` | critical | tire `pressure_kpa` < `low_pressure_kpa`, or sensor `alarm` |
| `NODE_OFFLINE` | warning | node silent past its `stale_after_seconds` |

## Configuration

See [`smores.toml`](../smores.toml). Connection settings can also be set via env
vars: `SMORES_BIND`, `SMORES_MQTT_HOST`, `SMORES_MQTT_PORT`, `SMORES_BASE_TOPIC`,
`SMORES_MOCK` (`0` disables the mock publisher), `SMORES_STALE_SECS`,
`SMORES_CONFIG` (path to the TOML file), `SMORES_DB` (SQLite history path,
default `smores.db`).
