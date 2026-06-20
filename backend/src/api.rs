//! REST API. Every handler reads the latest state and returns a typed view
//! (no loosely-typed JSON blobs). Each subsystem view carries `node_id`,
//! `source`, `timestamp`, and a computed `stale` flag.

use std::convert::Infallible;
use std::sync::Arc;

use axum::{
    extract::{Query, State},
    response::sse::{Event as SseEvent, KeepAlive, Sse},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio_stream::{wrappers::BroadcastStream, Stream, StreamExt};
use tower_http::cors::{Any, CorsLayer};

use crate::history::HistoryPoint;
use crate::models::*;
use crate::state::AppState;

pub fn router(state: Arc<AppState>) -> Router {
    // Permissive CORS: this is a local, offline device on the rig's own network.
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/v1/health", get(health))
        .route("/api/v1/battery", get(battery))
        .route("/api/v1/tanks", get(tanks))
        .route("/api/v1/tpms", get(tpms))
        .route("/api/v1/nodes", get(nodes))
        .route("/api/v1/alerts", get(alerts))
        .route("/api/v1/history", get(history))
        .route("/api/v1/stream", get(stream))
        .with_state(state)
        .layer(cors)
}

#[derive(Serialize)]
struct SystemHealth {
    status: &'static str,
    node_count: usize,
    online_nodes: usize,
    alert_count: usize,
    subsystems_present: Vec<String>,
}

async fn health(State(state): State<Arc<AppState>>) -> Json<SystemHealth> {
    let store = state.inner.read().await;
    let online_nodes = store
        .nodes
        .values()
        .filter(|n| !state.is_node_stale(&n.node_id, n.last_seen))
        .count();

    let mut subsystems_present = Vec::new();
    if store.battery.is_some() {
        subsystems_present.push("battery".to_string());
    }
    if store.tanks.is_some() {
        subsystems_present.push("tanks".to_string());
    }
    if store.tpms.is_some() {
        subsystems_present.push("tpms".to_string());
    }
    subsystems_present.extend(store.unknown.keys().cloned());

    Json(SystemHealth {
        status: "ok",
        node_count: store.nodes.len(),
        online_nodes,
        alert_count: store.alerts.len(),
        subsystems_present,
    })
}

#[derive(Serialize)]
struct BatteryView {
    #[serde(flatten)]
    data: Battery,
    node_id: String,
    source: Source,
    timestamp: Timestamp,
    stale: bool,
}

#[derive(Serialize)]
struct BatteryResponse {
    battery: Option<BatteryView>,
}

async fn battery(State(state): State<Arc<AppState>>) -> Json<BatteryResponse> {
    let store = state.inner.read().await;
    let battery = store.battery.as_ref().map(|s| BatteryView {
        data: s.data.clone(),
        node_id: s.node_id.clone(),
        source: s.source,
        timestamp: s.timestamp,
        stale: state.is_stale(s.received_at),
    });
    Json(BatteryResponse { battery })
}

#[derive(Serialize)]
struct TanksResponse {
    tanks: Vec<Tank>,
    node_id: Option<String>,
    timestamp: Option<Timestamp>,
    stale: bool,
}

async fn tanks(State(state): State<Arc<AppState>>) -> Json<TanksResponse> {
    let store = state.inner.read().await;
    match store.tanks.as_ref() {
        Some(s) => Json(TanksResponse {
            tanks: s.data.clone(),
            node_id: Some(s.node_id.clone()),
            timestamp: Some(s.timestamp),
            stale: state.is_stale(s.received_at),
        }),
        None => Json(TanksResponse {
            tanks: Vec::new(),
            node_id: None,
            timestamp: None,
            stale: false,
        }),
    }
}

#[derive(Serialize)]
struct TpmsResponse {
    sensors: Vec<TpmsSensor>,
    node_id: Option<String>,
    timestamp: Option<Timestamp>,
    stale: bool,
}

async fn tpms(State(state): State<Arc<AppState>>) -> Json<TpmsResponse> {
    let store = state.inner.read().await;
    match store.tpms.as_ref() {
        Some(s) => Json(TpmsResponse {
            sensors: s.data.clone(),
            node_id: Some(s.node_id.clone()),
            timestamp: Some(s.timestamp),
            stale: state.is_stale(s.received_at),
        }),
        None => Json(TpmsResponse {
            sensors: Vec::new(),
            node_id: None,
            timestamp: None,
            stale: false,
        }),
    }
}

#[derive(Serialize)]
struct NodeView {
    node_id: String,
    last_seen: Timestamp,
    online: bool,
    subsystems: Vec<String>,
    firmware_version: Option<String>,
    status: Option<NodeStatus>,
}

async fn nodes(State(state): State<Arc<AppState>>) -> Json<Vec<NodeView>> {
    let store = state.inner.read().await;
    let mut out: Vec<NodeView> = store
        .nodes
        .values()
        .map(|n| NodeView {
            node_id: n.node_id.clone(),
            last_seen: n.last_seen,
            online: !state.is_node_stale(&n.node_id, n.last_seen),
            subsystems: n.subsystems.clone(),
            firmware_version: n.health.as_ref().map(|h| h.firmware_version.clone()),
            status: n.health.as_ref().map(|h| h.status),
        })
        .collect();
    out.sort_by(|a, b| a.node_id.cmp(&b.node_id));
    Json(out)
}

async fn alerts(State(state): State<Arc<AppState>>) -> Json<Vec<Alert>> {
    let store = state.inner.read().await;
    Json(store.alerts.clone())
}

#[derive(Deserialize)]
struct HistoryQuery {
    subsystem: Option<String>,
    metric: Option<String>,
    since: Option<String>,
    limit: Option<i64>,
}

/// Trend history for one subsystem+metric, e.g.
/// `/history?subsystem=battery&metric=soc_percent&limit=200`.
/// Returns `[]` if history persistence is disabled.
async fn history(
    State(state): State<Arc<AppState>>,
    Query(q): Query<HistoryQuery>,
) -> Json<Vec<HistoryPoint>> {
    let Some(h) = &state.history else {
        return Json(Vec::new());
    };
    let subsystem = q.subsystem.unwrap_or_else(|| "battery".to_string());
    let metric = q.metric.unwrap_or_else(|| "soc_percent".to_string());
    let since = q.since.unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string());
    let limit = q.limit.unwrap_or(500).clamp(1, 5000);
    Json(h.query(&subsystem, &metric, &since, limit).await)
}

/// Server-sent events. Emits an `update` event whose data is the key of the
/// subsystem that changed (`battery`/`tanks`/`tpms`/`nodes`/`alerts`); the
/// dashboard refetches that data on receipt. A polling fallback covers any gaps.
async fn stream(
    State(state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<SseEvent, Infallible>>> {
    let rx = state.events.subscribe();
    let events = BroadcastStream::new(rx).filter_map(|msg| {
        msg.ok()
            .map(|key| Ok::<_, Infallible>(SseEvent::default().event("update").data(key)))
    });
    Sse::new(events).keep_alive(KeepAlive::default())
}
