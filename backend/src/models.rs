//! Canonical wire types — the Rust mirror of `docs/SCHEMAS.md`.
//!
//! These are the `smores.*.v1` schemas. Every component (firmware, mock
//! publisher, dashboard) derives from this contract. If you change a shape
//! here, change `docs/SCHEMAS.md` first and bump the schema version.
//!
//! NOTE: for Milestone 1 these live inside the backend crate. When a second
//! Rust consumer appears (e.g. a CAN/RV-C bridge) extract this module into a
//! shared `crates/models` workspace crate.

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub type Timestamp = chrono::DateTime<chrono::Utc>;

/// How a node acquired a reading (`source` in the envelope).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Mock,
    Mqtt,
    Can,
    Serial,
    Ble,
    Manual,
}

/// The canonical telemetry envelope (`smores.telemetry.v1`). The subsystem
/// payload is carried opaquely in `data` and parsed per-subsystem.
#[derive(Debug, Clone, Deserialize)]
pub struct TelemetryEnvelope {
    pub schema: String,
    pub node_id: String,
    pub subsystem: String,
    pub source: Source,
    pub timestamp: Timestamp,
    pub data: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChargingState {
    Charging,
    Discharging,
    Float,
    Idle,
    Unknown,
}

/// `battery` subsystem payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Battery {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub voltage_v: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub soc_percent: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_a: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub power_w: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temp_c: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub charging_state: Option<ChargingState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TankKind {
    Fresh,
    Gray,
    Black,
    Propane,
    Fuel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tank {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub kind: TankKind,
    pub level_percent: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capacity_l: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temp_c: Option<f64>,
}

/// `tanks` subsystem payload (one node reports several tanks).
#[derive(Debug, Clone, Deserialize)]
pub struct TanksPayload {
    pub tanks: Vec<Tank>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TpmsSensor {
    pub position: String,
    pub pressure_kpa: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temp_c: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sensor_battery_percent: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alarm: Option<bool>,
}

/// `tpms` subsystem payload (one node reports all monitored wheels).
#[derive(Debug, Clone, Deserialize)]
pub struct TpmsPayload {
    pub sensors: Vec<TpmsSensor>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeStatus {
    Online,
    Degraded,
    Offline,
}

/// Node health (`smores.health.v1`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Health {
    pub schema: String,
    pub node_id: String,
    pub status: NodeStatus,
    pub firmware_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uptime_s: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supply_voltage_v: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rssi_dbm: Option<f64>,
    #[serde(default)]
    pub errors: Vec<String>,
    pub timestamp: Timestamp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

/// An alert (`smores.alert.v1`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub schema: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub severity: Severity,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subsystem: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    pub code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    pub timestamp: Timestamp,
}
