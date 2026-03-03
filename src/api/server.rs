use crate::collectors::Snapshot;
use crate::engine::registry::CollectorHealth;
use crate::exporters::{prometheus::PrometheusExporter, Exporter};
use crate::reference::{self, Locale};
use anyhow::Result;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tracing::info;

#[derive(Clone)]
struct AppState {
    latest: Arc<RwLock<Snapshot>>,
    health: Arc<RwLock<HashMap<String, CollectorHealth>>>,
}

pub async fn run_server(
    bind: &str,
    port: u16,
    latest: Arc<RwLock<Snapshot>>,
    health: Arc<RwLock<HashMap<String, CollectorHealth>>>,
) -> Result<()> {
    let state = AppState { latest, health };
    let addr = format!("{}:{}", bind, port);

    let app = Router::new()
        .route("/metrics", get(metrics_handler))
        .route("/snapshot", get(snapshot_handler))
        .route("/health", get(health_handler))
        .route("/reference", get(reference_handler))
        .with_state(state);

    info!("API server listening on http://{}", addr);
    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let snapshot = state.latest.read().await.clone();
    let exporter = PrometheusExporter;
    info!(exporter = exporter.name(), "Serving /metrics");
    match exporter.export(&snapshot) {
        Ok(text) => (StatusCode::OK, text),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

async fn snapshot_handler(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.latest.read().await.clone())
}

#[derive(Debug, Deserialize)]
struct ReferenceQuery {
    q: Option<String>,
    lang: Option<String>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    collectors: HashMap<String, CollectorHealthView>,
}

#[derive(Serialize)]
struct CollectorHealthView {
    run_count: u64,
    error_count: u64,
    last_error: Option<String>,
    last_ok_ts: Option<i64>,
    healthy: bool,
}

async fn health_handler(State(state): State<AppState>) -> impl IntoResponse {
    let map = state.health.read().await;
    let collectors: HashMap<String, CollectorHealthView> = map
        .iter()
        .map(|(name, h)| {
            (
                name.clone(),
                CollectorHealthView {
                    run_count: h.run_count,
                    error_count: h.error_count,
                    last_error: h.last_error.clone(),
                    last_ok_ts: h.last_ok_ts,
                    healthy: h.last_error.is_none() && h.run_count > 0,
                },
            )
        })
        .collect();

    let all_healthy = collectors.values().all(|v| v.healthy);
    let status = if all_healthy { "ok" } else { "degraded" };
    let code = if all_healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (code, Json(HealthResponse { status, collectors }))
}

async fn reference_handler(Query(query): Query<ReferenceQuery>) -> impl IntoResponse {
    let locale = Locale::parse(query.lang.as_deref().unwrap_or("fr"));
    let body: Value = if let Some(q) = query.q.as_deref() {
        if q.trim().is_empty() {
            serde_json::to_value(reference::catalog_views(locale)).unwrap_or(Value::Null)
        } else {
            serde_json::to_value(reference::search(q, locale)).unwrap_or(Value::Null)
        }
    } else {
        serde_json::to_value(reference::catalog_views(locale)).unwrap_or(Value::Null)
    };
    (StatusCode::OK, Json(body))
}
