pub mod cpu;
pub mod disk;
pub mod linux;
pub mod logs;
pub mod memory;
pub mod network;
pub mod process;
pub mod system;

pub use cpu::{CpuCollector, CpuMetrics, LoadAverageSource};
pub use disk::{DiskCollector, DiskMetrics};
pub use linux::{LinuxCollector, LinuxMetrics};
pub use logs::LogsCollector;
pub use memory::{MemoryCollector, MemoryMetrics};
pub use network::{NetworkCollector, NetworkMetrics};
pub use process::{ProcessCollector, ProcessMetrics};
pub use system::{SystemCollector, SystemMetrics};

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ComputedMetrics {
    pub cpu_trend_p50: f64,
    pub cpu_trend_p95: f64,
    pub memory_pressure: f64,
    pub alerts_info: usize,
    pub alerts_warning: usize,
    pub alerts_critical: usize,
    pub alerts: Vec<Alert>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Alert {
    pub level: AlertLevel,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct LogEntry {
    pub timestamp: i64,
    pub level: AlertLevel,
    pub source: String,
    pub origin: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct LogsMetrics {
    pub timestamp: i64,
    pub system_events: Vec<LogEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum AlertLevel {
    #[default]
    Info,
    Warning,
    Critical,
}

/// Snapshot complet de toutes les métriques à un instant T.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Snapshot {
    pub timestamp: i64,
    pub cpu: Option<CpuMetrics>,
    pub memory: Option<MemoryMetrics>,
    pub disks: Vec<DiskMetrics>,
    pub networks: Vec<NetworkMetrics>,
    pub processes: Vec<ProcessMetrics>,
    pub system: Option<SystemMetrics>,
    pub linux: Option<LinuxMetrics>,
    pub logs: Option<LogsMetrics>,
    pub computed: ComputedMetrics,
}

/// Contrat de tout collector : collecter et écrire dans le snapshot.
/// L'intervalle d'exécution est géré par le Registry/Scheduler, pas ici.
#[async_trait::async_trait]
pub trait Collector: Send + Sync {
    fn name(&self) -> &'static str;
    async fn collect(&mut self, snapshot: &mut Snapshot) -> Result<()>;
}

pub use async_trait::async_trait;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_deserializes_when_computed_field_is_missing() {
        let json = r#"{
            "timestamp": 1,
            "cpu": null,
            "memory": null,
            "disks": [],
            "networks": [],
            "processes": [],
            "system": null
        }"#;

        let snapshot: Snapshot = serde_json::from_str(json).unwrap();
        assert_eq!(snapshot.timestamp, 1);
        assert_eq!(snapshot.computed.cpu_trend_p50, 0.0);
        assert_eq!(snapshot.computed.cpu_trend_p95, 0.0);
        assert_eq!(snapshot.computed.memory_pressure, 0.0);
        assert_eq!(snapshot.computed.alerts_info, 0);
        assert_eq!(snapshot.computed.alerts_warning, 0);
        assert_eq!(snapshot.computed.alerts_critical, 0);
        assert!(snapshot.linux.is_none());
        assert!(snapshot.logs.is_none());
        assert!(snapshot.computed.alerts.is_empty());
    }

    #[test]
    fn process_metrics_deserializes_when_new_fields_are_missing() {
        let json = r#"{
            "timestamp": 1,
            "pid": 42,
            "name": "java",
            "cmdline": "java -jar app.jar",
            "cpu_pct": 0.0,
            "mem_rss_kb": 128,
            "mem_vsz_kb": 256,
            "threads": 4,
            "fd_count": 10,
            "state": "Sleeping",
            "user": "app",
            "io_read_bytes": 0,
            "io_write_bytes": 0
        }"#;

        let process: ProcessMetrics = serde_json::from_str(json).unwrap();
        assert!(!process.is_jvm);
    }
}
