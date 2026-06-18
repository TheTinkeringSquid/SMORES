//! In-memory latest-state store. Holds the most recent reading per subsystem
//! plus a `received_at` wall-clock stamp (for staleness), a per-node registry,
//! and the active alert list. Persistence (SQLite) arrives in Milestone 2.

use std::collections::HashMap;

use chrono::Utc;
use serde_json::Value;
use tokio::sync::RwLock;

use crate::models::*;

/// A stored reading plus where/when it came from.
#[derive(Debug, Clone)]
pub struct Stamped<T> {
    pub data: T,
    pub node_id: String,
    pub source: Source,
    pub timestamp: Timestamp,
    /// When the backend received it (used for staleness; not the node's clock).
    pub received_at: Timestamp,
}

impl<T> Stamped<T> {
    fn new(data: T, node_id: String, source: Source, timestamp: Timestamp) -> Self {
        Self {
            data,
            node_id,
            source,
            timestamp,
            received_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NodeRecord {
    pub node_id: String,
    pub last_seen: Timestamp,
    pub subsystems: Vec<String>,
    pub health: Option<Health>,
}

/// The latest known state of the whole rig.
#[derive(Debug, Default)]
pub struct Store {
    pub battery: Option<Stamped<Battery>>,
    pub tanks: Option<Stamped<Vec<Tank>>>,
    pub tpms: Option<Stamped<Vec<TpmsSensor>>>,
    pub nodes: HashMap<String, NodeRecord>,
    pub alerts: Vec<Alert>,
    /// Telemetry for subsystems we don't model yet — never dropped (schema §1).
    pub unknown: HashMap<String, Stamped<Value>>,
}

#[derive(Debug)]
pub struct AppState {
    pub inner: RwLock<Store>,
    /// A reading/node older than this many seconds is reported as stale.
    pub stale_after_secs: i64,
}

const MAX_ALERTS: usize = 200;

impl AppState {
    pub fn new(stale_after_secs: i64) -> Self {
        Self {
            inner: RwLock::default(),
            stale_after_secs,
        }
    }

    /// True if `received_at` is older than the configured stale window.
    pub fn is_stale(&self, received_at: Timestamp) -> bool {
        (Utc::now() - received_at).num_seconds() > self.stale_after_secs
    }

    /// Record that a node was just heard from, tracking which subsystems it reports.
    fn touch_node(store: &mut Store, node_id: &str, subsystem: Option<&str>, seen: Timestamp) {
        let rec = store
            .nodes
            .entry(node_id.to_string())
            .or_insert_with(|| NodeRecord {
                node_id: node_id.to_string(),
                last_seen: seen,
                subsystems: Vec::new(),
                health: None,
            });
        rec.last_seen = seen;
        if let Some(sub) = subsystem {
            if !rec.subsystems.iter().any(|s| s == sub) {
                rec.subsystems.push(sub.to_string());
            }
        }
    }

    /// Parse and apply a telemetry envelope. Unrecognized subsystems are stored
    /// raw rather than dropped, so new node types work without a backend release.
    pub async fn apply_envelope(&self, env: TelemetryEnvelope) {
        let mut store = self.inner.write().await;
        Self::touch_node(&mut store, &env.node_id, Some(&env.subsystem), env.timestamp);

        // Destructure so subsystem-specific arms can move fields out freely.
        let TelemetryEnvelope {
            node_id,
            subsystem,
            source,
            timestamp,
            data,
            ..
        } = env;

        match subsystem.as_str() {
            "battery" => match serde_json::from_value::<Battery>(data) {
                Ok(b) => store.battery = Some(Stamped::new(b, node_id, source, timestamp)),
                Err(e) => tracing::warn!("bad battery payload from {node_id}: {e}"),
            },
            "tanks" => match serde_json::from_value::<TanksPayload>(data) {
                Ok(p) => store.tanks = Some(Stamped::new(p.tanks, node_id, source, timestamp)),
                Err(e) => tracing::warn!("bad tanks payload from {node_id}: {e}"),
            },
            "tpms" => match serde_json::from_value::<TpmsPayload>(data) {
                Ok(p) => store.tpms = Some(Stamped::new(p.sensors, node_id, source, timestamp)),
                Err(e) => tracing::warn!("bad tpms payload from {node_id}: {e}"),
            },
            other => {
                store
                    .unknown
                    .insert(other.to_string(), Stamped::new(data, node_id, source, timestamp));
            }
        }
    }

    pub async fn apply_health(&self, health: Health) {
        let mut store = self.inner.write().await;
        let node_id = health.node_id.clone();
        let seen = health.timestamp;
        Self::touch_node(&mut store, &node_id, None, seen);
        if let Some(rec) = store.nodes.get_mut(&node_id) {
            rec.health = Some(health);
        }
    }

    /// Apply an alert. `active: false` clears matching alerts (by `id`, else `code`);
    /// otherwise it is raised (deduped on the same key).
    pub async fn apply_alert(&self, alert: Alert) {
        let mut store = self.inner.write().await;
        // Capture the dedupe key by value so the closure doesn't borrow `alert`
        // (which we move into the list below).
        let id = alert.id.clone();
        let code = alert.code.clone();
        let subsystem = alert.subsystem.clone();
        let key_matches = |a: &Alert| match (&id, &a.id) {
            (Some(x), Some(y)) => x == y,
            _ => a.code == code && a.subsystem == subsystem,
        };

        if alert.active == Some(false) {
            store.alerts.retain(|a| !key_matches(a));
            return;
        }
        store.alerts.retain(|a| !key_matches(a));
        store.alerts.push(alert);
        if store.alerts.len() > MAX_ALERTS {
            let overflow = store.alerts.len() - MAX_ALERTS;
            store.alerts.drain(0..overflow);
        }
    }
}
