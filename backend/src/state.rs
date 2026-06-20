//! In-memory latest-state store plus the alert-threshold engine. Holds the most
//! recent reading per subsystem (with a `received_at` stamp for staleness), a
//! per-node registry, and the active alert list. Persistence (SQLite) is a
//! later Milestone-2 slice.

use std::collections::HashMap;

use chrono::Utc;
use serde_json::Value;
use tokio::sync::RwLock;

use crate::config::Thresholds;
use crate::history::{History, HistoryRow};
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
    /// Default stale window for readings and for nodes without an override.
    pub stale_after_secs: i64,
    /// Alert-engine thresholds.
    pub thresholds: Thresholds,
    /// Per-node stale overrides from the config registry.
    pub node_stale: HashMap<String, i64>,
    /// Optional SQLite history (None if the DB failed to open).
    pub history: Option<History>,
}

const MAX_ALERTS: usize = 200;

impl AppState {
    pub fn new(
        stale_after_secs: i64,
        thresholds: Thresholds,
        node_stale: HashMap<String, i64>,
        history: Option<History>,
    ) -> Self {
        Self {
            inner: RwLock::default(),
            stale_after_secs,
            thresholds,
            node_stale,
            history,
        }
    }

    /// Snapshot current numeric readings into the history DB. Called on a timer;
    /// builds rows under a read lock, then writes outside the lock (no I/O while
    /// the store is locked).
    pub async fn sample(&self) {
        let Some(history) = &self.history else {
            return;
        };
        let rows = {
            let store = self.inner.read().await;
            let now = Utc::now().to_rfc3339();
            let mut rows: Vec<HistoryRow> = Vec::new();

            if let Some(b) = &store.battery {
                let push = |rows: &mut Vec<HistoryRow>, metric: &str, value: f64| {
                    rows.push(HistoryRow {
                        ts: now.clone(),
                        node_id: b.node_id.clone(),
                        subsystem: "battery".to_string(),
                        metric: metric.to_string(),
                        value,
                    });
                };
                if let Some(v) = b.data.soc_percent {
                    push(&mut rows, "soc_percent", v);
                }
                push(&mut rows, "voltage_v", b.data.voltage_v);
                if let Some(v) = b.data.current_a {
                    push(&mut rows, "current_a", v);
                }
            }

            if let Some(t) = &store.tanks {
                for tank in &t.data {
                    rows.push(HistoryRow {
                        ts: now.clone(),
                        node_id: t.node_id.clone(),
                        subsystem: "tanks".to_string(),
                        metric: format!("level_percent:{}", tank.id),
                        value: tank.level_percent,
                    });
                }
            }

            if let Some(tp) = &store.tpms {
                for s in &tp.data {
                    rows.push(HistoryRow {
                        ts: now.clone(),
                        node_id: tp.node_id.clone(),
                        subsystem: "tpms".to_string(),
                        metric: format!("pressure_kpa:{}", s.position),
                        value: s.pressure_kpa,
                    });
                }
            }
            rows
        };
        history.insert(&rows).await;
    }

    /// True if a reading's `received_at` is older than the default stale window.
    pub fn is_stale(&self, received_at: Timestamp) -> bool {
        (Utc::now() - received_at).num_seconds() > self.stale_after_secs
    }

    /// True if a node hasn't been heard from within its (possibly per-node) window.
    pub fn is_node_stale(&self, node_id: &str, last_seen: Timestamp) -> bool {
        let limit = self
            .node_stale
            .get(node_id)
            .copied()
            .unwrap_or(self.stale_after_secs);
        (Utc::now() - last_seen).num_seconds() > limit
    }

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

    /// Parse and apply a telemetry envelope, then re-run the alert engine.
    /// Unrecognized subsystems are stored raw rather than dropped (schema §1).
    pub async fn apply_envelope(&self, env: TelemetryEnvelope) {
        let mut store = self.inner.write().await;
        Self::touch_node(&mut store, &env.node_id, Some(&env.subsystem), env.timestamp);

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

        Self::evaluate(&mut store, &self.thresholds, &self.node_stale, self.stale_after_secs);
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

    /// Apply an externally-published alert (from a node or `system/alerts`).
    pub async fn apply_alert(&self, alert: Alert) {
        let mut store = self.inner.write().await;
        Self::upsert(&mut store, alert);
    }

    /// Periodic re-evaluation so node-offline alerts fire even when no telemetry
    /// is arriving.
    pub async fn tick(&self) {
        let mut store = self.inner.write().await;
        Self::evaluate(&mut store, &self.thresholds, &self.node_stale, self.stale_after_secs);
    }

    /// Raise or clear an alert. `active: false` clears matching alerts (by `id`,
    /// else `code`+`subsystem`); otherwise it is raised, deduped on the same key.
    fn upsert(store: &mut Store, alert: Alert) {
        // Capture the dedupe key by value so the closure doesn't borrow `alert`,
        // which we move into the list below.
        let id = alert.id.clone();
        let code = alert.code.clone();
        let subsystem = alert.subsystem.clone();
        let key_matches = |a: &Alert| match (&id, &a.id) {
            (Some(x), Some(y)) => x == y,
            _ => a.code == code && a.subsystem == subsystem,
        };

        store.alerts.retain(|a| !key_matches(a));
        if alert.active == Some(false) {
            return;
        }
        store.alerts.push(alert);
        if store.alerts.len() > MAX_ALERTS {
            let overflow = store.alerts.len() - MAX_ALERTS;
            store.alerts.drain(0..overflow);
        }
    }

    fn derived(
        id: &str,
        severity: Severity,
        subsystem: Option<&str>,
        code: &str,
        message: String,
        active: bool,
    ) -> Alert {
        Alert {
            schema: "smores.alert.v1".to_string(),
            id: Some(id.to_string()),
            severity,
            subsystem: subsystem.map(|s| s.to_string()),
            node_id: None,
            code: code.to_string(),
            message,
            active: Some(active),
            timestamp: Utc::now(),
        }
    }

    /// The alert engine: derive alerts from current readings + node liveness.
    /// Each rule upserts with `active = condition`, so alerts self-clear once the
    /// condition resolves.
    fn evaluate(
        store: &mut Store,
        thresholds: &Thresholds,
        node_stale: &HashMap<String, i64>,
        default_stale: i64,
    ) {
        // Battery: low state of charge.
        let soc = store.battery.as_ref().and_then(|b| b.data.soc_percent);
        if let Some(soc) = soc {
            Self::upsert(
                store,
                Self::derived(
                    "thr-low-soc",
                    Severity::Warning,
                    Some("battery"),
                    "LOW_SOC",
                    format!(
                        "House battery SOC {soc:.0}% below {:.0}%",
                        thresholds.low_soc_percent
                    ),
                    soc < thresholds.low_soc_percent,
                ),
            );
        }

        // Tanks: gray/black above the high mark.
        let tanks = store.tanks.as_ref().map(|s| s.data.clone());
        if let Some(tanks) = tanks {
            for t in tanks {
                if matches!(t.kind, TankKind::Gray | TankKind::Black) {
                    let label = t.name.clone().unwrap_or_else(|| format!("{:?}", t.kind));
                    Self::upsert(
                        store,
                        Self::derived(
                            &format!("thr-high-tank-{}", t.id),
                            Severity::Warning,
                            Some("tanks"),
                            "HIGH_TANK",
                            format!("{label} tank {:.0}% full", t.level_percent),
                            t.level_percent > thresholds.high_tank_percent,
                        ),
                    );
                }
            }
        }

        // TPMS: low pressure or a node-reported alarm.
        let sensors = store.tpms.as_ref().map(|s| s.data.clone());
        if let Some(sensors) = sensors {
            for s in sensors {
                let active = s.alarm.unwrap_or(false) || s.pressure_kpa < thresholds.low_pressure_kpa;
                let pos = s.position.replace('_', " ");
                Self::upsert(
                    store,
                    Self::derived(
                        &format!("thr-low-pressure-{}", s.position),
                        Severity::Critical,
                        Some("tpms"),
                        "LOW_PRESSURE",
                        format!("{pos} tire pressure {:.0} kPa low", s.pressure_kpa),
                        active,
                    ),
                );
            }
        }

        // Nodes: offline past their (possibly per-node) stale window.
        let now = Utc::now();
        let offline: Vec<(String, bool)> = store
            .nodes
            .values()
            .map(|n| {
                let limit = node_stale.get(&n.node_id).copied().unwrap_or(default_stale);
                (n.node_id.clone(), (now - n.last_seen).num_seconds() > limit)
            })
            .collect();
        for (id, off) in offline {
            Self::upsert(
                store,
                Self::derived(
                    &format!("node-offline-{id}"),
                    Severity::Warning,
                    None,
                    "NODE_OFFLINE",
                    format!("Node {id} is offline"),
                    off,
                ),
            );
        }
    }
}
