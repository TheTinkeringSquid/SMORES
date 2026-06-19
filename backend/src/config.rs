//! Runtime configuration, sourced from environment variables with sensible
//! defaults for the dev container. A TOML node-registry config arrives in
//! Milestone 2 (see docs roadmap); env vars are enough for Milestone 1.

use std::net::SocketAddr;

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
    /// A reading/node older than this is considered stale (`SMORES_STALE_SECS`, default 15).
    pub stale_after_secs: i64,
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

impl Config {
    pub fn from_env() -> Self {
        let bind = env_or("SMORES_BIND", "0.0.0.0:8080");
        let bind_addr = bind
            .parse()
            .unwrap_or_else(|_| panic!("invalid SMORES_BIND: {bind:?}"));

        let mqtt_port = std::env::var("SMORES_MQTT_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1883);

        let mock = std::env::var("SMORES_MOCK")
            .map(|v| !(v == "0" || v.eq_ignore_ascii_case("false")))
            .unwrap_or(true);

        let stale_after_secs = std::env::var("SMORES_STALE_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(15);

        Self {
            bind_addr,
            mqtt_host: env_or("SMORES_MQTT_HOST", "mqtt"),
            mqtt_port,
            base_topic: env_or("SMORES_BASE_TOPIC", "smores/v1"),
            mock,
            stale_after_secs,
        }
    }
}
