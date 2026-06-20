//! Runtime configuration. Loaded from an optional `smores.toml` (node registry,
//! alert thresholds, system/mqtt settings) with environment variables taking
//! precedence for connection settings. Everything has a sensible default, so the
//! file is optional and the dev container works with zero config.

use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Config {
    /// Address the HTTP API binds to (`SMORES_BIND`, default `0.0.0.0:8080`).
    pub bind_addr: SocketAddr,
    /// MQTT broker host (`SMORES_MQTT_HOST`, default `mqtt` — the dev-container service).
    pub mqtt_host: String,
    /// MQTT broker port (`SMORES_MQTT_PORT`, default `1883`).
    pub mqtt_port: u16,
    /// Topic prefix (`SMORES_BASE_TOPIC`, default `smores/v1`).
    pub base_topic: String,
    /// Run the built-in mock publisher (`SMORES_MOCK`, default on). Set `0`/`false` to disable.
    pub mock: bool,
    /// Default stale window for nodes/readings without a per-node override.
    pub stale_after_secs: i64,
    /// Display name for the system.
    pub system_name: String,
    /// Declared nodes (registry); drives per-node stale windows.
    pub nodes: Vec<NodeConfig>,
    /// Thresholds for the backend alert engine.
    pub thresholds: Thresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub subsystems: Vec<String>,
    #[serde(default = "default_stale")]
    pub stale_after_seconds: i64,
}

fn default_stale() -> i64 {
    15
}

/// Alert-engine thresholds. Missing fields fall back to [`Thresholds::default`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Thresholds {
    /// Raise LOW_SOC when battery SOC drops below this percent.
    pub low_soc_percent: f64,
    /// Raise HIGH_TANK when a gray/black tank rises above this percent.
    pub high_tank_percent: f64,
    /// Raise LOW_PRESSURE when a tire drops below this pressure (kPa).
    pub low_pressure_kpa: f64,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            low_soc_percent: 50.0,
            high_tank_percent: 85.0,
            low_pressure_kpa: 350.0,
        }
    }
}

#[derive(Debug, Default, Deserialize)]
struct FileConfig {
    system: Option<SystemSection>,
    mqtt: Option<MqttSection>,
    thresholds: Option<Thresholds>,
    #[serde(default)]
    nodes: Vec<NodeConfig>,
}

#[derive(Debug, Default, Deserialize)]
struct SystemSection {
    name: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct MqttSection {
    host: Option<String>,
    port: Option<u16>,
    base_topic: Option<String>,
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

impl Config {
    pub fn load() -> Self {
        let file = load_file();
        let mqtt = file.mqtt.unwrap_or_default();

        let bind = env_or("SMORES_BIND", "0.0.0.0:8080");
        let bind_addr = bind
            .parse()
            .unwrap_or_else(|_| panic!("invalid SMORES_BIND: {bind:?}"));

        let mqtt_host = std::env::var("SMORES_MQTT_HOST")
            .ok()
            .or(mqtt.host)
            .unwrap_or_else(|| "mqtt".to_string());
        let mqtt_port = std::env::var("SMORES_MQTT_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .or(mqtt.port)
            .unwrap_or(1883);
        let base_topic = std::env::var("SMORES_BASE_TOPIC")
            .ok()
            .or(mqtt.base_topic)
            .unwrap_or_else(|| "smores/v1".to_string());

        let mock = std::env::var("SMORES_MOCK")
            .map(|v| !(v == "0" || v.eq_ignore_ascii_case("false")))
            .unwrap_or(true);
        let stale_after_secs = std::env::var("SMORES_STALE_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(15);

        Self {
            bind_addr,
            mqtt_host,
            mqtt_port,
            base_topic,
            mock,
            stale_after_secs,
            system_name: file
                .system
                .and_then(|s| s.name)
                .unwrap_or_else(|| "S.M.O.R.E.S.".to_string()),
            nodes: file.nodes,
            thresholds: file.thresholds.unwrap_or_default(),
        }
    }
}

/// Read the config file from `SMORES_CONFIG`, or the first of `smores.toml` /
/// `../smores.toml` that exists (so it works whether run from the repo root or
/// from `backend/`). Missing or invalid files fall back to defaults.
fn load_file() -> FileConfig {
    let candidates: Vec<String> = match std::env::var("SMORES_CONFIG") {
        Ok(p) if !p.is_empty() => vec![p],
        _ => vec!["smores.toml".to_string(), "../smores.toml".to_string()],
    };
    let Some(path) = candidates
        .into_iter()
        .find(|p| std::path::Path::new(p).exists())
    else {
        tracing::info!("no smores.toml found; using env/defaults");
        return FileConfig::default();
    };

    match std::fs::read_to_string(&path).map(|s| toml::from_str::<FileConfig>(&s)) {
        Ok(Ok(f)) => {
            tracing::info!("loaded config from {path}");
            f
        }
        Ok(Err(e)) => {
            tracing::warn!("failed to parse {path}: {e}; using defaults");
            FileConfig::default()
        }
        Err(e) => {
            tracing::warn!("failed to read {path}: {e}; using defaults");
            FileConfig::default()
        }
    }
}
