use axum::{routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BatteryTelemetry {
    soc_percent: Option<f32>,
    voltage_v: f32,
    current_a: Option<f32>,
    temp_c: Option<f32>,
    timestamp: String,
}

async fn get_battery() -> Json<serde_json::Value> {
    let sample = BatteryTelemetry {
        soc_percent: Some(82.5),
        voltage_v: 13.2,
        current_a: Some(-4.1),
        temp_c: Some(28.0),
        timestamp: "2025-08-13T00:00:00Z".to_string(),
    };
    Json(serde_json::json!({ "battery": sample }))
}

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);
    let app = Router::new().route("/api/v1/battery", get(get_battery)).layer(cors);
    let addr: SocketAddr = ([0, 0, 0, 0], 8080).into();
    println!("Backend on http://{addr}");
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app).await.unwrap();
}
