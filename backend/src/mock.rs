//! Built-in mock publisher. Publishes battery/tanks/tpms telemetry, periodic
//! node health, and the occasional alert — using the *exact same* envelopes and
//! topics a real node would. This is what keeps the whole stack demonstrable
//! with no hardware (Milestone 1). Disable with `SMORES_MOCK=0`.

use std::time::Duration;

use chrono::Utc;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use serde_json::json;

use crate::config::Config;

pub async fn run(cfg: Config) {
    loop {
        if let Err(e) = run_once(&cfg).await {
            tracing::warn!("mock publisher error: {e:#}; retrying in 3s");
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    }
}

async fn run_once(cfg: &Config) -> anyhow::Result<()> {
    let mut opts = MqttOptions::new("smores-mock", &cfg.mqtt_host, cfg.mqtt_port);
    opts.set_keep_alive(Duration::from_secs(30));
    let (client, mut eventloop) = AsyncClient::new(opts, 32);
    let base = cfg.base_topic.clone();

    // Drive the event loop so queued publishes are actually sent.
    tokio::spawn(async move {
        loop {
            if eventloop.poll().await.is_err() {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    });

    tracing::info!("mock publisher started (set SMORES_MOCK=0 to disable)");

    let mut ticker = tokio::time::interval(Duration::from_secs(3));
    let mut tick: u64 = 0;

    loop {
        ticker.tick().await;
        tick += 1;
        let t = tick as f64;
        let now = Utc::now().to_rfc3339();

        // ---- battery ----
        let soc = (72.0 + 12.0 * (t / 6.0).sin()).clamp(0.0, 100.0);
        let current = -6.0 + 4.0 * (t / 4.0).cos();
        let battery = json!({
            "schema": "smores.telemetry.v1",
            "node_id": "battery-node-01",
            "subsystem": "battery",
            "source": "mock",
            "timestamp": now.clone(),
            "data": {
                "id": "house-1",
                "name": "House Battery",
                "soc_percent": (soc * 10.0).round() / 10.0,
                "voltage_v": ((12.2 + soc / 100.0 * 1.4) * 100.0).round() / 100.0,
                "current_a": (current * 10.0).round() / 10.0,
                "temp_c": 28.0,
                "charging_state": if current >= 0.0 { "charging" } else { "discharging" }
            }
        });
        client
            .publish(
                format!("{base}/battery-node-01/telemetry/battery"),
                QoS::AtLeastOnce,
                false,
                battery.to_string(),
            )
            .await?;

        // ---- tanks ----
        let fresh = (90.0 - t).rem_euclid(100.0);
        let tanks = json!({
            "schema": "smores.telemetry.v1",
            "node_id": "tank-node-01",
            "subsystem": "tanks",
            "source": "mock",
            "timestamp": now.clone(),
            "data": { "tanks": [
                { "id": "fresh-1", "name": "Fresh", "kind": "fresh", "level_percent": (fresh * 10.0).round() / 10.0, "capacity_l": 150 },
                { "id": "gray-1",  "name": "Gray",  "kind": "gray",  "level_percent": (t * 2.0).rem_euclid(100.0), "capacity_l": 120 },
                { "id": "lp-1",    "name": "Propane","kind": "propane","level_percent": 41.0 }
            ]}
        });
        client
            .publish(
                format!("{base}/tank-node-01/telemetry/tanks"),
                QoS::AtLeastOnce,
                false,
                tanks.to_string(),
            )
            .await?;

        // ---- tpms ---- (one wheel periodically alarms)
        let rear_left = 448.0;
        let alarm = tick % 7 == 0;
        let tpms = json!({
            "schema": "smores.telemetry.v1",
            "node_id": "tpms-node-01",
            "subsystem": "tpms",
            "source": "mock",
            "timestamp": now.clone(),
            "data": { "sensors": [
                { "position": "front_left",  "pressure_kpa": 379.0, "temp_c": 34.0, "sensor_battery_percent": 88, "alarm": false },
                { "position": "front_right", "pressure_kpa": 372.0, "temp_c": 35.0, "sensor_battery_percent": 90, "alarm": false },
                { "position": "rear_left",   "pressure_kpa": rear_left, "temp_c": 41.0, "sensor_battery_percent": 76, "alarm": alarm },
                { "position": "rear_right",  "pressure_kpa": 451.0, "temp_c": 40.0, "sensor_battery_percent": 79, "alarm": false }
            ]}
        });
        client
            .publish(
                format!("{base}/tpms-node-01/telemetry/tpms"),
                QoS::AtLeastOnce,
                false,
                tpms.to_string(),
            )
            .await?;

        // ---- health (every ~9s) ----
        if tick % 3 == 0 {
            for node in ["battery-node-01", "tank-node-01", "tpms-node-01"] {
                let health = json!({
                    "schema": "smores.health.v1",
                    "node_id": node,
                    "status": "online",
                    "firmware_version": "0.1.0-mock",
                    "uptime_s": tick * 3,
                    "supply_voltage_v": 12.6,
                    "errors": [],
                    "timestamp": now.clone(),
                });
                client
                    .publish(
                        format!("{base}/{node}/health"),
                        QoS::AtLeastOnce,
                        false,
                        health.to_string(),
                    )
                    .await?;
            }
        }

        // ---- alert: raise/clear a low-SOC warning depending on SOC ----
        let alert = json!({
            "schema": "smores.alert.v1",
            "id": "mock-low-soc",
            "severity": "warning",
            "subsystem": "battery",
            "node_id": "battery-node-01",
            "code": "LOW_SOC",
            "message": "House battery SOC is below configured threshold",
            "active": soc < 64.0,
            "timestamp": now.clone(),
        });
        client
            .publish(
                format!("{base}/system/alerts"),
                QoS::AtLeastOnce,
                false,
                alert.to_string(),
            )
            .await?;
    }
}
