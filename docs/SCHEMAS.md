# S.M.O.R.E.S. Data Schemas (`smores.*.v1`)

> **This document is the contract.** Backend models, node firmware, mock
> publishers, and dashboard types all derive from what is written here. If a
> field is not described in this file, no component should rely on it. Change
> this doc first, bump the schema version, then change the code.

S.M.O.R.E.S. unifies many otherwise-isolated RV systems (TPMS, tanks, solar,
HVAC, powertrain, battery, ...) into one pane of glass. The only thing that
makes that possible is that **every source — mock or real — emits the same
envelope on the same topics.** A mock publisher and a real ESP32 node are
indistinguishable to the backend.

---

## 1. Versioning & compatibility rules

- Every message carries a `schema` string, e.g. `smores.telemetry.v1`.
- **Additive, non-breaking changes** (new *optional* field) do **not** bump the
  version. Consumers must ignore unknown fields.
- **Breaking changes** (renamed/removed field, changed type or unit, new
  *required* field) bump the integer: `...v1` → `...v2`. Both may run in
  parallel during migration.
- Consumers must **not drop** a message just because its `subsystem` is
  unrecognized — store it as raw/unknown so a new subsystem can appear without a
  backend release.

## 2. Units & naming conventions

Units are encoded in the field name suffix. SI unless noted.

| Suffix | Unit | Example |
|--------|------|---------|
| `_v` | volts | `voltage_v` |
| `_a` | amperes (signed; **+ = charging/in, − = load/out**) | `current_a` |
| `_w` | watts | `power_w` |
| `_c` | degrees Celsius | `temp_c` |
| `_percent` | 0–100 | `soc_percent` |
| `_kpa` | kilopascals (pressure canonical unit) | `pressure_kpa` |
| `_l` | liters | `capacity_l` |
| `_s` | seconds | `uptime_s` |
| `_dbm` | dBm (signal) | `rssi_dbm` |

- All identifiers are `snake_case`.
- All timestamps are **UTC, RFC 3339 / ISO 8601** with a `Z`: `2026-06-18T20:00:00Z`.
- Display conversions (psi, °F, gallons) happen in the **frontend only**. The
  bus is always canonical units.

## 3. Topic conventions

Base prefix: `smores/v1`. `<node_id>` is a stable, unique, kebab/lower id.

```
smores/v1/<node_id>/telemetry/<subsystem>   # sensor readings (this doc, §5)
smores/v1/<node_id>/state/<subsystem>        # actuator/reported state (future)
smores/v1/<node_id>/command/<subsystem>      # commands TO a node (future, validated)
smores/v1/<node_id>/config                   # node config snapshot (future)
smores/v1/<node_id>/health                   # node liveness/diagnostics (§6)
smores/v1/system/events                      # system-wide events (future)
smores/v1/system/alerts                      # alerts (§7)
```

Backend subscriptions for Milestone 1:

```
smores/v1/+/telemetry/+
smores/v1/+/health
smores/v1/system/alerts
```

Recommended QoS 1 for telemetry/health/alerts. Health may use an MQTT **Last
Will** publishing an `offline` health message so node loss is detected even on a
hard crash.

---

## 4. Canonical telemetry envelope — `smores.telemetry.v1`

Every telemetry message shares this outer envelope. The subsystem-specific
payload lives entirely inside `data` (§5).

```json
{
  "schema": "smores.telemetry.v1",
  "node_id": "battery-node-01",
  "subsystem": "battery",
  "source": "mqtt",
  "timestamp": "2026-06-18T20:00:00Z",
  "data": { }
}
```

| Field | Type | Req | Notes |
|-------|------|-----|-------|
| `schema` | string | ✓ | Must equal `smores.telemetry.v1`. |
| `node_id` | string | ✓ | Stable unique node id. |
| `subsystem` | string | ✓ | One of the registered subsystems (§5). Drives routing. |
| `source` | enum | ✓ | `mock` \| `mqtt` \| `can` \| `serial` \| `ble` \| `manual`. How the node *acquired* the data. |
| `timestamp` | string | ✓ | UTC RFC 3339. Time the reading was taken at the node. |
| `data` | object | ✓ | Subsystem payload, schema per §5. |

The backend additionally records its own **`received_at`** wall-clock time per
(node_id, subsystem) for staleness detection — this is *not* part of the wire
format.

---

## 5. Subsystem `data` payloads

Each subsystem defines its own `data` shape. Milestone 1 implements **battery,
tanks, tpms**. Others are reserved (listed for forward planning) and may be
stored as unknown until implemented.

### 5.1 `battery`

Single battery/bank per envelope (use distinct `node_id`s for multiple banks).

```json
{
  "id": "house-1",
  "name": "House Battery",
  "soc_percent": 78.2,
  "voltage_v": 12.7,
  "current_a": -8.4,
  "power_w": -106.7,
  "temp_c": 31.0,
  "charging_state": "discharging"
}
```

| Field | Type | Req | Notes |
|-------|------|-----|-------|
| `id` | string | ✓ | Logical battery id. |
| `name` | string |  | Human label. |
| `voltage_v` | number | ✓ | Terminal voltage. |
| `soc_percent` | number |  | State of charge 0–100. |
| `current_a` | number |  | Signed: + charging, − load. |
| `power_w` | number |  | Signed, same convention. |
| `temp_c` | number |  | Battery temperature. |
| `charging_state` | enum |  | `charging` \| `discharging` \| `float` \| `idle` \| `unknown`. |

### 5.2 `tanks`

A node may report several tanks at once → `tanks` is an **array**.

```json
{
  "tanks": [
    { "id": "fresh-1", "name": "Fresh", "kind": "fresh", "level_percent": 64.0, "capacity_l": 150 },
    { "id": "gray-1",  "name": "Gray",  "kind": "gray",  "level_percent": 22.5, "capacity_l": 120 },
    { "id": "lp-1",    "name": "Propane","kind": "propane","level_percent": 41.0 }
  ]
}
```

| Field | Type | Req | Notes |
|-------|------|-----|-------|
| `tanks[]` | array | ✓ | One entry per tank. |
| `tanks[].id` | string | ✓ | Logical tank id. |
| `tanks[].kind` | enum | ✓ | `fresh` \| `gray` \| `black` \| `propane` \| `fuel`. |
| `tanks[].level_percent` | number | ✓ | 0–100. |
| `tanks[].name` | string |  | Human label. |
| `tanks[].capacity_l` | number |  | Nominal capacity. |
| `tanks[].temp_c` | number |  | Optional (e.g. water heater). |

### 5.3 `tpms`

A receiver node typically reports all wheels → `sensors` is an **array**.

```json
{
  "sensors": [
    { "position": "front_left",  "pressure_kpa": 379.0, "temp_c": 34.0, "sensor_battery_percent": 88, "alarm": false },
    { "position": "front_right", "pressure_kpa": 372.0, "temp_c": 35.0, "sensor_battery_percent": 90, "alarm": false },
    { "position": "rear_left",   "pressure_kpa": 448.0, "temp_c": 41.0, "sensor_battery_percent": 76, "alarm": true  }
  ]
}
```

| Field | Type | Req | Notes |
|-------|------|-----|-------|
| `sensors[]` | array | ✓ | One entry per monitored wheel. |
| `sensors[].position` | string | ✓ | `front_left`, `rear_right`, `rear_inner_left`, `spare`, ... |
| `sensors[].pressure_kpa` | number | ✓ | Canonical pressure unit (frontend may show psi). |
| `sensors[].temp_c` | number |  | Tire/sensor temperature. |
| `sensors[].sensor_battery_percent` | number |  | TPMS sender battery. |
| `sensors[].alarm` | boolean |  | Node-side fast alarm flag (fault/leak). |

### 5.4 Reserved subsystems (planned, not in M1)

Named now so node authors stay consistent; each gets a `data` shape here before
it is implemented. Until then the backend stores them as `unknown` (never
dropped, per §1) and the dashboard may show a generic card.

`solar`, `inverter`, `shore_power`, `ems`, `generator`, `water`, `hvac`,
`fans`, `fridge`, `safety`, `leveling`, `slides`, `awning`, `chassis`,
`powertrain`, `lighting`, `connectivity`, `cameras`, `infotainment`,
`location`, `security`, `appliances`.

See [`RV_SYSTEMS.md`](./RV_SYSTEMS.md) for what each subsystem covers on a real
RV, the signals it exposes, and the open integration path (RV-C, Victron,
Modbus, BLE) for sourcing its data.

---

## 6. Node health — `smores.health.v1`

Published per node on `smores/v1/<node_id>/health`, every ~10 s and via Last
Will on disconnect.

```json
{
  "schema": "smores.health.v1",
  "node_id": "speedo-node-01",
  "firmware_version": "0.1.0",
  "status": "online",
  "uptime_s": 12345,
  "supply_voltage_v": 12.6,
  "rssi_dbm": -61,
  "errors": [],
  "timestamp": "2026-06-18T20:00:00Z"
}
```

| Field | Type | Req | Notes |
|-------|------|-----|-------|
| `schema` | string | ✓ | `smores.health.v1`. |
| `node_id` | string | ✓ | |
| `status` | enum | ✓ | `online` \| `degraded` \| `offline`. |
| `firmware_version` | string | ✓ | |
| `uptime_s` | number |  | |
| `supply_voltage_v` | number |  | Node input voltage (brown-out detection). |
| `rssi_dbm` | number |  | Wi-Fi signal, if wireless. |
| `errors` | string[] |  | Free-form fault codes. |
| `timestamp` | string | ✓ | UTC RFC 3339. |

The backend also derives a node **stale/offline** status when
`now - received_at > stale_after_seconds` (per-node config), independent of any
self-reported `status`.

---

## 7. Alerts — `smores.alert.v1`

Published on `smores/v1/system/alerts`. May originate from a node or from the
backend alert engine (Phase 2).

```json
{
  "schema": "smores.alert.v1",
  "id": "a3f1c2",
  "severity": "warning",
  "subsystem": "battery",
  "node_id": "battery-node-01",
  "code": "LOW_SOC",
  "message": "House battery SOC is below configured threshold",
  "active": true,
  "timestamp": "2026-06-18T20:00:00Z"
}
```

| Field | Type | Req | Notes |
|-------|------|-----|-------|
| `schema` | string | ✓ | `smores.alert.v1`. |
| `severity` | enum | ✓ | `info` \| `warning` \| `critical`. |
| `code` | string | ✓ | Stable machine code, e.g. `LOW_SOC`, `HIGH_TANK`, `TPMS_LEAK`. |
| `message` | string | ✓ | Human-readable. |
| `subsystem` | string |  | Affected subsystem. |
| `node_id` | string |  | Originating/affected node. |
| `id` | string |  | Stable id for dedupe/ack. |
| `active` | boolean |  | `true` raise, `false` clears the same `code`/`id`. |
| `timestamp` | string | ✓ | UTC RFC 3339. |

---

## 8. Example `mosquitto_pub` commands

From inside the dev container (broker host `mqtt`):

```bash
# Battery telemetry
mosquitto_pub -h mqtt -t smores/v1/battery-node-01/telemetry/battery -m '{
  "schema":"smores.telemetry.v1","node_id":"battery-node-01","subsystem":"battery",
  "source":"mqtt","timestamp":"2026-06-18T20:00:00Z",
  "data":{"id":"house-1","name":"House Battery","soc_percent":78.2,"voltage_v":12.7,"current_a":-8.4,"charging_state":"discharging"}
}'

# Tank levels (multiple tanks in one message)
mosquitto_pub -h mqtt -t smores/v1/tank-node-01/telemetry/tanks -m '{
  "schema":"smores.telemetry.v1","node_id":"tank-node-01","subsystem":"tanks",
  "source":"mqtt","timestamp":"2026-06-18T20:00:00Z",
  "data":{"tanks":[{"id":"fresh-1","kind":"fresh","level_percent":64.0},{"id":"gray-1","kind":"gray","level_percent":22.5}]}
}'

# TPMS (one wheel alarming)
mosquitto_pub -h mqtt -t smores/v1/tpms-node-01/telemetry/tpms -m '{
  "schema":"smores.telemetry.v1","node_id":"tpms-node-01","subsystem":"tpms",
  "source":"mqtt","timestamp":"2026-06-18T20:00:00Z",
  "data":{"sensors":[{"position":"rear_left","pressure_kpa":448.0,"temp_c":41.0,"alarm":true}]}
}'

# Alert
mosquitto_pub -h mqtt -t smores/v1/system/alerts -m '{
  "schema":"smores.alert.v1","severity":"warning","subsystem":"battery","code":"LOW_SOC",
  "message":"House battery SOC is below configured threshold","active":true,
  "timestamp":"2026-06-18T20:00:00Z"
}'

# Node health
mosquitto_pub -h mqtt -t smores/v1/battery-node-01/health -m '{
  "schema":"smores.health.v1","node_id":"battery-node-01","status":"online",
  "firmware_version":"0.1.0","uptime_s":12345,"supply_voltage_v":12.6,
  "errors":[],"timestamp":"2026-06-18T20:00:00Z"
}'
```
