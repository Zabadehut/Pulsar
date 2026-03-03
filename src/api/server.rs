use crate::collectors::Snapshot;
use crate::engine::registry::CollectorHealth;
use crate::exporters::{prometheus::PrometheusExporter, Exporter};
use crate::platform::current;
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
        .route("/inventory", get(inventory_handler))
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

#[derive(Serialize)]
struct InventoryResponse {
    host: Option<InventoryHost>,
    disks: Vec<DiskInventoryView>,
    networks: Vec<NetworkInventoryView>,
}

#[derive(Serialize)]
struct InventoryHost {
    hostname: String,
    os_name: String,
    os_version: String,
    kernel_version: String,
    architecture: String,
}

#[derive(Serialize)]
struct DiskInventoryView {
    device: String,
    parent: String,
    structure: String,
    filesystem: String,
    label: String,
    uuid: String,
    part_uuid: String,
    model: String,
    serial: String,
    transport: String,
    reference: String,
    mount_point: String,
    mount_points: Vec<String>,
    children: Vec<String>,
    protocol: String,
    media: String,
    total_gb: f64,
    used_gb: f64,
    free_gb: f64,
    usage_pct: f64,
}

#[derive(Serialize)]
struct NetworkInventoryView {
    interface: String,
    topology: String,
    family: String,
    medium: String,
    connections_total: u32,
    connections_established: u32,
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

async fn inventory_handler(State(state): State<AppState>) -> impl IntoResponse {
    let snapshot = state.latest.read().await.clone();
    let disk_metrics_by_device: HashMap<String, _> = snapshot
        .disks
        .iter()
        .map(|disk| (disk.device.clone(), disk))
        .collect();
    let host = snapshot.system.as_ref().map(|system| InventoryHost {
        hostname: system.hostname.clone(),
        os_name: system.os_name.clone(),
        os_version: system.os_version.clone(),
        kernel_version: system.kernel_version.clone(),
        architecture: system.architecture.clone(),
    });

    let raw_inventory = current::read_disk_inventory().unwrap_or_default();
    let disks = if raw_inventory.is_empty() {
        snapshot
            .disks
            .iter()
            .map(|disk| DiskInventoryView {
                device: disk.device.clone(),
                parent: disk.parent.clone(),
                structure: if disk.structure.is_empty() {
                    disk.structure_hint.clone()
                } else {
                    disk.structure.clone()
                },
                filesystem: disk.filesystem.clone(),
                label: disk.label.clone(),
                uuid: disk.uuid.clone(),
                part_uuid: disk.part_uuid.clone(),
                model: disk.model.clone(),
                serial: disk.serial.clone(),
                transport: disk.protocol_hint.clone(),
                reference: disk.reference.clone(),
                mount_point: disk.mount_point.clone(),
                mount_points: disk.mount_points.clone(),
                children: disk.children.clone(),
                protocol: disk.protocol_hint.clone(),
                media: disk.media_hint.clone(),
                total_gb: disk.total_gb,
                used_gb: disk.used_gb,
                free_gb: disk.free_gb,
                usage_pct: disk.usage_pct,
            })
            .collect()
    } else {
        raw_inventory
            .iter()
            .map(|disk| {
                let metrics = disk_metrics_by_device.get(&disk.device).copied();
                DiskInventoryView {
                    device: disk.device.clone(),
                    parent: disk.parent.clone().unwrap_or_default(),
                    structure: if disk.structure.is_empty() {
                        metrics
                            .map(|item| item.structure_hint.clone())
                            .unwrap_or_default()
                    } else {
                        disk.structure.clone()
                    },
                    filesystem: disk.filesystem.clone(),
                    label: disk.label.clone(),
                    uuid: disk.uuid.clone(),
                    part_uuid: disk.part_uuid.clone(),
                    model: disk.model.clone(),
                    serial: disk.serial.clone(),
                    transport: disk.transport.clone(),
                    reference: disk.reference.clone(),
                    mount_point: metrics
                        .map(|item| item.mount_point.clone())
                        .filter(|value| !value.is_empty())
                        .or_else(|| disk.mount_points.first().cloned())
                        .unwrap_or_default(),
                    mount_points: if disk.mount_points.is_empty() {
                        metrics
                            .map(|item| item.mount_points.clone())
                            .unwrap_or_default()
                    } else {
                        disk.mount_points.clone()
                    },
                    children: disk.children.clone(),
                    protocol: metrics
                        .map(|item| item.protocol_hint.clone())
                        .unwrap_or_else(|| disk.transport.clone()),
                    media: metrics
                        .map(|item| item.media_hint.clone())
                        .unwrap_or_default(),
                    total_gb: metrics.map(|item| item.total_gb).unwrap_or(0.0),
                    used_gb: metrics.map(|item| item.used_gb).unwrap_or(0.0),
                    free_gb: metrics.map(|item| item.free_gb).unwrap_or(0.0),
                    usage_pct: metrics.map(|item| item.usage_pct).unwrap_or(0.0),
                }
            })
            .collect()
    };

    let networks = snapshot
        .networks
        .iter()
        .map(|net| NetworkInventoryView {
            interface: net.interface.clone(),
            topology: net.topology_hint.clone(),
            family: net.family_hint.clone(),
            medium: net.medium_hint.clone(),
            connections_total: net.connections_total,
            connections_established: net.connections_established,
        })
        .collect();

    (
        StatusCode::OK,
        Json(InventoryResponse {
            host,
            disks,
            networks,
        }),
    )
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
