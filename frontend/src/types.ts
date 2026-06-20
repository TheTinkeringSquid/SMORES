// TypeScript mirror of the backend REST responses, which derive from
// docs/SCHEMAS.md. Keep these in sync with backend/src/models.rs and api.rs.
// (A future milestone can generate these from the Rust types; see the review.)

export type Source = "mock" | "mqtt" | "can" | "serial" | "ble" | "manual";
export type ChargingState =
  | "charging"
  | "discharging"
  | "float"
  | "idle"
  | "unknown";
export type TankKind = "fresh" | "gray" | "black" | "propane" | "fuel";
export type NodeStatus = "online" | "degraded" | "offline";
export type Severity = "info" | "warning" | "critical";

export interface Battery {
  id: string;
  name?: string;
  voltage_v: number;
  soc_percent?: number;
  current_a?: number;
  power_w?: number;
  temp_c?: number;
  charging_state?: ChargingState;
  node_id: string;
  source: Source;
  timestamp: string;
  stale: boolean;
}
export interface BatteryResponse {
  battery: Battery | null;
}

export interface Tank {
  id: string;
  name?: string;
  kind: TankKind;
  level_percent: number;
  capacity_l?: number;
  temp_c?: number;
}
export interface TanksResponse {
  tanks: Tank[];
  node_id: string | null;
  timestamp: string | null;
  stale: boolean;
}

export interface TpmsSensor {
  position: string;
  pressure_kpa: number;
  temp_c?: number;
  sensor_battery_percent?: number;
  alarm?: boolean;
}
export interface TpmsResponse {
  sensors: TpmsSensor[];
  node_id: string | null;
  timestamp: string | null;
  stale: boolean;
}

export interface NodeView {
  node_id: string;
  last_seen: string;
  online: boolean;
  subsystems: string[];
  firmware_version?: string | null;
  status?: NodeStatus | null;
}

export interface Alert {
  schema: string;
  id?: string;
  severity: Severity;
  subsystem?: string;
  node_id?: string;
  code: string;
  message: string;
  active?: boolean;
  timestamp: string;
}

export interface SystemHealth {
  status: string;
  node_count: number;
  online_nodes: number;
  alert_count: number;
  subsystems_present: string[];
}

export interface HistoryPoint {
  ts: string;
  value: number;
}

export interface Thresholds {
  low_soc_percent: number;
  high_tank_percent: number;
  low_pressure_kpa: number;
}

export interface NodeConfig {
  id: string;
  name?: string | null;
  subsystems: string[];
  stale_after_seconds: number;
}

export interface AppConfig {
  system_name: string;
  thresholds: Thresholds;
  nodes: NodeConfig[];
}
