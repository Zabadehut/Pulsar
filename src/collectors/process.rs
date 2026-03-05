use crate::collectors::{async_trait, Collector, Snapshot};
use crate::platform::{api::RawProcReading, current};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum ProcessState {
    Running,
    Sleeping,
    DiskSleep,
    Zombie,
    Stopped,
    TracingStop,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ProcessMetrics {
    pub timestamp: i64,
    pub pid: u32,
    pub name: String,
    pub cmdline: String,
    pub cpu_pct: f64,
    pub mem_rss_kb: u64,
    pub mem_vsz_kb: u64,
    pub threads: u32,
    pub fd_count: u32,
    pub state: ProcessState,
    pub user: String,
    pub io_read_bytes: u64,
    pub io_write_bytes: u64,
    pub is_jvm: bool,
    pub fd_count_supported: bool,
    pub io_bytes_supported: bool,
}

pub struct ProcessCollector {
    pub top_n: usize,
    pub jvm_detection: bool,
    watched_pid: Option<u32>,
    prev: std::collections::HashMap<u32, RawProcReading>,
    prev_at: Option<Instant>,
}

impl ProcessCollector {
    pub fn new(top_n: usize, jvm_detection: bool) -> Self {
        Self {
            top_n,
            jvm_detection,
            watched_pid: None,
            prev: std::collections::HashMap::new(),
            prev_at: None,
        }
    }

    pub fn new_watching(pid: u32, jvm_detection: bool) -> Self {
        Self {
            top_n: 1,
            jvm_detection,
            watched_pid: Some(pid),
            prev: std::collections::HashMap::new(),
            prev_at: None,
        }
    }
}

#[async_trait]
impl Collector for ProcessCollector {
    fn name(&self) -> &'static str {
        "process"
    }

    async fn collect(&mut self, snapshot: &mut Snapshot) -> Result<()> {
        snapshot.processes = collect_processes(self)?;
        Ok(())
    }
}

fn state_from_char(c: char) -> ProcessState {
    match c {
        'R' => ProcessState::Running,
        'S' => ProcessState::Sleeping,
        'D' => ProcessState::DiskSleep,
        'Z' => ProcessState::Zombie,
        'T' => ProcessState::Stopped,
        't' => ProcessState::TracingStop,
        _ => ProcessState::Unknown,
    }
}

fn collect_processes(c: &mut ProcessCollector) -> Result<Vec<ProcessMetrics>> {
    let page_size = current::page_size();
    let clock_ticks = current::clock_ticks();
    let num_cpus = current::num_cpus();
    let now = chrono::Utc::now().timestamp();
    let collected_at = Instant::now();
    let elapsed_secs = c
        .prev_at
        .map(|prev| collected_at.saturating_duration_since(prev).as_secs_f64())
        .filter(|secs| *secs > 0.0);
    let mut processes = Vec::new();

    for proc in current::read_processes()? {
        if c.watched_pid.is_some() && c.watched_pid != Some(proc.pid) {
            continue;
        }

        let cpu_pct = if let Some(prev) = c.prev.get(&proc.pid) {
            let delta = (proc.utime + proc.stime).saturating_sub(prev.utime + prev.stime);
            elapsed_secs
                .map(|secs| compute_cpu_pct(delta, clock_ticks, secs, num_cpus))
                .unwrap_or_else(|| proc.cpu_pct_hint.unwrap_or(0.0))
        } else {
            proc.cpu_pct_hint.unwrap_or(0.0)
        };

        let is_jvm = c.jvm_detection
            && (proc.name.contains("java")
                || proc.cmdline.contains("java")
                || proc.cmdline.contains("jvm"));

        processes.push(ProcessMetrics {
            timestamp: now,
            pid: proc.pid,
            name: proc.name.clone(),
            cmdline: proc.cmdline.clone(),
            cpu_pct,
            mem_rss_kb: proc.rss * page_size / 1024,
            mem_vsz_kb: proc.vsize / 1024,
            threads: proc.threads,
            fd_count: proc.fd_count,
            state: state_from_char(proc.state_char),
            user: proc.user.clone(),
            io_read_bytes: proc.io_read_bytes,
            io_write_bytes: proc.io_write_bytes,
            is_jvm,
            fd_count_supported: proc.fd_count_supported,
            io_bytes_supported: proc.io_bytes_supported,
        });

        c.prev.insert(proc.pid, proc);
    }

    c.prev_at = Some(collected_at);

    if c.watched_pid.is_none() {
        processes.sort_by(|a, b| {
            b.cpu_pct
                .partial_cmp(&a.cpu_pct)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        processes.truncate(c.top_n);
    }
    Ok(processes)
}

fn compute_cpu_pct(delta_ticks: u64, clock_ticks: f64, elapsed_secs: f64, num_cpus: f64) -> f64 {
    if clock_ticks <= 0.0 || elapsed_secs <= 0.0 {
        0.0
    } else {
        (delta_ticks as f64 / (clock_ticks * elapsed_secs) * 100.0).clamp(0.0, 100.0 * num_cpus)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_pct_scales_with_elapsed_time() {
        let cpu_pct = compute_cpu_pct(200, 100.0, 2.0, 8.0);
        assert_eq!(cpu_pct, 100.0);
    }

    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn process_collector_respects_limit_and_populates_processes_on_linux() {
        let limit = 5;
        let mut collector = ProcessCollector::new(limit, true);
        let mut snapshot = Snapshot::default();

        collector.collect(&mut snapshot).await.unwrap();

        assert!(snapshot.processes.len() <= limit);
        for process in snapshot.processes {
            assert!(process.pid > 0);
            assert!(!process.name.is_empty());
            assert!(process.cpu_pct.is_finite());
            assert!(process.mem_rss_kb <= process.mem_vsz_kb || process.mem_vsz_kb == 0);
        }
    }
}
