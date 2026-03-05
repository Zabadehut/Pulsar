use crate::collectors::{async_trait, Collector, Snapshot};
use crate::platform::current;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct MemoryMetrics {
    pub timestamp: i64,
    pub total_kb: u64,
    pub used_kb: u64,
    pub free_kb: u64,
    pub available_kb: u64,
    pub cached_kb: u64,
    pub buffers_kb: u64,
    pub swap_total_kb: u64,
    pub swap_used_kb: u64,
    pub dirty_kb: u64,
    pub vm_pgfault: u64,
    pub vm_pgmajfault: u64,
    pub vm_pgpgin: u64,
    pub vm_pgpgout: u64,
    pub vm_pswpin: u64,
    pub vm_pswpout: u64,
    pub vm_pgscan: u64,
    pub vm_pgsteal: u64,
    pub usage_pct: f64,
    pub cached_supported: bool,
    pub buffers_supported: bool,
    pub dirty_supported: bool,
    pub vm_fault_counters_supported: bool,
    pub vm_scan_counters_supported: bool,
    pub vm_io_counters_supported: bool,
}

pub struct MemoryCollector;

impl MemoryCollector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MemoryCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Collector for MemoryCollector {
    fn name(&self) -> &'static str {
        "memory"
    }

    async fn collect(&mut self, snapshot: &mut Snapshot) -> Result<()> {
        snapshot.memory = Some(collect_memory()?);
        Ok(())
    }
}

fn collect_memory() -> Result<MemoryMetrics> {
    let memory = current::read_memory()?;
    Ok(MemoryMetrics {
        timestamp: chrono::Utc::now().timestamp(),
        total_kb: memory.total_kb,
        used_kb: memory.used_kb,
        free_kb: memory.free_kb,
        available_kb: memory.available_kb,
        cached_kb: memory.cached_kb,
        buffers_kb: memory.buffers_kb,
        swap_total_kb: memory.swap_total_kb,
        swap_used_kb: memory.swap_used_kb,
        dirty_kb: memory.dirty_kb,
        vm_pgfault: memory.vm_pgfault,
        vm_pgmajfault: memory.vm_pgmajfault,
        vm_pgpgin: memory.vm_pgpgin,
        vm_pgpgout: memory.vm_pgpgout,
        vm_pswpin: memory.vm_pswpin,
        vm_pswpout: memory.vm_pswpout,
        vm_pgscan: memory.vm_pgscan,
        vm_pgsteal: memory.vm_pgsteal,
        usage_pct: memory.usage_pct,
        cached_supported: memory.cached_supported,
        buffers_supported: memory.buffers_supported,
        dirty_supported: memory.dirty_supported,
        vm_fault_counters_supported: memory.vm_fault_counters_supported,
        vm_scan_counters_supported: memory.vm_scan_counters_supported,
        vm_io_counters_supported: memory.vm_io_counters_supported,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn memory_collector_populates_snapshot_on_linux() {
        let mut collector = MemoryCollector::new();
        let mut snapshot = Snapshot::default();

        collector.collect(&mut snapshot).await.unwrap();

        let memory = snapshot.memory.expect("memory metrics should be present");
        assert!(memory.total_kb > 0);
        assert!(memory.available_kb <= memory.total_kb);
        assert!(memory.used_kb <= memory.total_kb);
        assert!((0.0..=100.0).contains(&memory.usage_pct));
    }
}
