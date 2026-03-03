use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub collectors: CollectorsConfig,
    #[serde(default)]
    pub exporters: ExportersConfig,
    #[serde(default)]
    pub api: ApiConfig,
    #[serde(default)]
    pub tui: TuiConfig,
    #[serde(default)]
    pub pipeline: PipelineConfig,
    #[serde(default)]
    pub record: RecordConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    pub log_level: String,
    pub hostname_override: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CollectorsConfig {
    // Activation
    pub cpu: bool,
    pub memory: bool,
    pub disk: bool,
    pub network: bool,
    pub process: bool,
    pub containers: bool,

    // Intervalles d'exécution (en secondes) — toute valeur vient du config
    pub cpu_interval_secs: u64,
    pub memory_interval_secs: u64,
    pub disk_interval_secs: u64,
    pub network_interval_secs: u64,
    pub process_interval_secs: u64,
    pub system_interval_secs: u64,

    // Options process
    pub process_top_n: usize,
    pub jvm_detection: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ExportersConfig {
    pub json: bool,
    pub csv: bool,
    pub prometheus: bool,
    pub prometheus_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ApiConfig {
    pub enabled: bool,
    pub port: u16,
    pub bind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TuiConfig {
    pub enabled: bool,
    pub refresh_rate_ms: u64,
    pub theme: String,
    pub locale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PipelineConfig {
    pub cpu_trend: bool,
    pub mem_pressure: bool,
    pub alerts: bool,
    #[serde(default)]
    pub thresholds: AlertThresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AlertThresholds {
    pub cpu_warning_pct: f64,
    pub cpu_critical_pct: f64,
    pub mem_warning_pct: f64,
    pub mem_critical_pct: f64,
    pub psi_cpu_some_warning_pct: f64,
    pub psi_cpu_some_critical_pct: f64,
    pub psi_memory_some_warning_pct: f64,
    pub psi_memory_some_critical_pct: f64,
    pub psi_io_some_warning_pct: f64,
    pub psi_io_some_critical_pct: f64,
    pub cgroup_memory_warning_pct: f64,
    pub cgroup_memory_critical_pct: f64,
    pub cgroup_cpu_throttling_warning_pct: f64,
    pub cgroup_cpu_throttling_critical_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RecordConfig {
    pub interval_secs: u64,
    pub output: String,
    pub rotate: String,
    pub max_file_size_mb: Option<u64>,
    pub keep_files: Option<usize>,
    pub compress: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            hostname_override: String::new(),
        }
    }
}

impl Default for CollectorsConfig {
    fn default() -> Self {
        Self {
            cpu: true,
            memory: true,
            disk: true,
            network: true,
            process: true,
            containers: false,

            cpu_interval_secs: 1,
            memory_interval_secs: 2,
            disk_interval_secs: 2,
            network_interval_secs: 2,
            process_interval_secs: 2,
            system_interval_secs: 30,

            process_top_n: 20,
            jvm_detection: true,
        }
    }
}

impl Default for ExportersConfig {
    fn default() -> Self {
        Self {
            json: true,
            csv: false,
            prometheus: false,
            prometheus_port: 9090,
        }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            port: 8080,
            bind: "127.0.0.1".to_string(),
        }
    }
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            refresh_rate_ms: 500,
            theme: "dark".to_string(),
            locale: "fr".to_string(),
        }
    }
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            cpu_trend: true,
            mem_pressure: true,
            alerts: true,
            thresholds: AlertThresholds::default(),
        }
    }
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            cpu_warning_pct: 75.0,
            cpu_critical_pct: 90.0,
            mem_warning_pct: 80.0,
            mem_critical_pct: 95.0,
            psi_cpu_some_warning_pct: 10.0,
            psi_cpu_some_critical_pct: 20.0,
            psi_memory_some_warning_pct: 5.0,
            psi_memory_some_critical_pct: 10.0,
            psi_io_some_warning_pct: 10.0,
            psi_io_some_critical_pct: 20.0,
            cgroup_memory_warning_pct: 80.0,
            cgroup_memory_critical_pct: 95.0,
            cgroup_cpu_throttling_warning_pct: 10.0,
            cgroup_cpu_throttling_critical_pct: 25.0,
        }
    }
}

impl Default for RecordConfig {
    fn default() -> Self {
        Self {
            interval_secs: 5,
            output: ".".to_string(),
            rotate: "never".to_string(),
            max_file_size_mb: None,
            keep_files: None,
            compress: "none".to_string(),
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config: {}", path.display()))?;
        toml::from_str(&content)
            .with_context(|| format!("Failed to parse config: {}", path.display()))
    }

    pub fn load_or_default(path: &Path) -> Self {
        if path.exists() {
            match Self::load(path) {
                Ok(cfg) => cfg,
                Err(e) => {
                    tracing::warn!("Failed to load config, using defaults: {}", e);
                    Self::default()
                }
            }
        } else {
            Self::default()
        }
    }
}
