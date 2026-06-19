//! MQTT ingress. Subscribes to telemetry/health/alert topics, parses each
//! payload against the canonical schemas, and updates the state store.
//! Resilient by design: on any connection error it backs off and reconnects,
//! and (re)subscribes on every ConnAck so subscriptions survive reconnects.

use std::sync::Arc;
use std::time::Duration;

use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};

use crate::config::Config;
use crate::models::{Alert, Health, TelemetryEnvelope};
use crate::state::AppState;

pub async fn run(cfg: Config, state: Arc<AppState>) {
    loop {
        if let Err(e) = run_once(&cfg, &state).await {
            tracing::warn!("mqtt subscriber error: {e:#}; reconnecting in 2s");
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }
}

async fn run_once(cfg: &Config, state: &Arc<AppState>) -> anyhow::Result<()> {
    let mut opts = MqttOptions::new("smores-backend", &cfg.mqtt_host, cfg.mqtt_port);
    opts.set_keep_alive(Duration::from_secs(30));
    let (client, mut eventloop) = AsyncClient::new(opts, 64);
    let base = cfg.base_topic.clone();

    loop {
        match eventloop.poll().await? {
            Event::Incoming(Packet::ConnAck(_)) => {
                client
                    .subscribe(format!("{base}/+/telemetry/+"), QoS::AtLeastOnce)
                    .await?;
                client
                    .subscribe(format!("{base}/+/health"), QoS::AtLeastOnce)
                    .await?;
                client
                    .subscribe(format!("{base}/system/alerts"), QoS::AtLeastOnce)
                    .await?;
                tracing::info!(
                    "mqtt connected to {}:{} (base topic {base})",
                    cfg.mqtt_host,
                    cfg.mqtt_port
                );
            }
            Event::Incoming(Packet::Publish(p)) => {
                handle_publish(&p.topic, &p.payload, state, &base).await;
            }
            _ => {}
        }
    }
}

async fn handle_publish(topic: &str, payload: &[u8], state: &Arc<AppState>, base: &str) {
    if topic == format!("{base}/system/alerts") {
        match serde_json::from_slice::<Alert>(payload) {
            Ok(alert) => state.apply_alert(alert).await,
            Err(e) => tracing::warn!("bad alert payload on {topic}: {e}"),
        }
        return;
    }

    let rest = topic.strip_prefix(&format!("{base}/")).unwrap_or(topic);
    let parts: Vec<&str> = rest.split('/').collect();

    match parts.as_slice() {
        [_node, "health"] => match serde_json::from_slice::<Health>(payload) {
            Ok(h) => state.apply_health(h).await,
            Err(e) => tracing::warn!("bad health payload on {topic}: {e}"),
        },
        [_node, "telemetry", _sub] => match serde_json::from_slice::<TelemetryEnvelope>(payload) {
            Ok(env) => state.apply_envelope(env).await,
            Err(e) => tracing::warn!("bad telemetry envelope on {topic}: {e}"),
        },
        _ => tracing::debug!("ignoring message on unrecognized topic {topic}"),
    }
}
