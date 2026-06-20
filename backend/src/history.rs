//! SQLite telemetry history for trend charts. Numeric metrics are sampled
//! periodically into one table; the `/history` endpoint queries them back.
//!
//! Uses sqlx's runtime query API (not the compile-time `query!` macro) so the
//! crate builds with no database present.

use sqlx::{Row, SqlitePool};

#[derive(Clone, Debug)]
pub struct History {
    pool: SqlitePool,
}

/// One numeric sample to persist.
pub struct HistoryRow {
    pub ts: String,
    pub node_id: String,
    pub subsystem: String,
    pub metric: String,
    pub value: f64,
}

/// One point returned to the dashboard.
#[derive(serde::Serialize)]
pub struct HistoryPoint {
    pub ts: String,
    pub value: f64,
}

impl History {
    /// Open (creating if needed) the SQLite database and ensure the schema.
    pub async fn connect(path: &str) -> anyhow::Result<Self> {
        let url = format!("sqlite:{path}?mode=rwc");
        let pool = SqlitePool::connect(&url).await?;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS telemetry_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ts TEXT NOT NULL,
                node_id TEXT NOT NULL,
                subsystem TEXT NOT NULL,
                metric TEXT NOT NULL,
                value REAL NOT NULL
            )",
        )
        .execute(&pool)
        .await?;
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_hist ON telemetry_history (subsystem, metric, ts)",
        )
        .execute(&pool)
        .await?;
        Ok(Self { pool })
    }

    /// Insert a batch of samples. Failures are logged, not propagated — losing a
    /// history sample must never disrupt live telemetry.
    pub async fn insert(&self, rows: &[HistoryRow]) {
        for r in rows {
            let res = sqlx::query(
                "INSERT INTO telemetry_history (ts, node_id, subsystem, metric, value)
                 VALUES (?, ?, ?, ?, ?)",
            )
            .bind(r.ts.as_str())
            .bind(r.node_id.as_str())
            .bind(r.subsystem.as_str())
            .bind(r.metric.as_str())
            .bind(r.value)
            .execute(&self.pool)
            .await;
            if let Err(e) = res {
                tracing::warn!("history insert failed: {e}");
            }
        }
    }

    /// Query points for one subsystem+metric since a timestamp, oldest-first.
    pub async fn query(
        &self,
        subsystem: &str,
        metric: &str,
        since: &str,
        limit: i64,
    ) -> Vec<HistoryPoint> {
        let res = sqlx::query(
            "SELECT ts, value FROM telemetry_history
             WHERE subsystem = ? AND metric = ? AND ts >= ?
             ORDER BY ts ASC LIMIT ?",
        )
        .bind(subsystem)
        .bind(metric)
        .bind(since)
        .bind(limit)
        .fetch_all(&self.pool)
        .await;

        match res {
            Ok(rows) => rows
                .iter()
                .map(|r| HistoryPoint {
                    ts: r.get::<String, _>("ts"),
                    value: r.get::<f64, _>("value"),
                })
                .collect(),
            Err(e) => {
                tracing::warn!("history query failed: {e}");
                Vec::new()
            }
        }
    }
}
