//! S.M.O.R.E.S. backend entrypoint. Wires config, the in-memory state store,
//! the MQTT subscriber, the optional mock publisher, and the REST API.

mod api;
mod config;
mod mock;
mod models;
mod mqtt;
mod state;

use std::sync::Arc;

use config::Config;
use state::AppState;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let cfg = Config::load();
    let node_stale = cfg
        .nodes
        .iter()
        .map(|n| (n.id.clone(), n.stale_after_seconds))
        .collect();
    let state = Arc::new(AppState::new(
        cfg.stale_after_secs,
        cfg.thresholds.clone(),
        node_stale,
    ));

    // MQTT ingress: keeps the state store fresh from telemetry/health/alerts.
    tokio::spawn(mqtt::run(cfg.clone(), state.clone()));

    // Mock publisher: makes the stack demonstrable without hardware.
    if cfg.mock {
        tokio::spawn(mock::run(cfg.clone()));
    }

    // Alert engine tick: re-evaluate so node-offline alerts fire without telemetry.
    {
        let state = state.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
            loop {
                interval.tick().await;
                state.tick().await;
            }
        });
    }

    let app = api::router(state.clone());
    let listener = tokio::net::TcpListener::bind(cfg.bind_addr).await?;
    tracing::info!(
        "{} backend listening on http://{}",
        cfg.system_name,
        cfg.bind_addr
    );
    axum::serve(listener, app).await?;
    Ok(())
}
