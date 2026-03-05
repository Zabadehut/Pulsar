use crate::collectors::{async_trait, Collector, Snapshot};
use crate::platform::{
    api::{RawCpuStat, RawLoadAverageSource},
    current,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CoreMetrics {
    pub id: usize,
    pub usage_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CpuModeMetrics {
    pub user_pct: f64,
    pub nice_pct: f64,
    pub system_pct: f64,
    pub idle_pct: f64,
    pub iowait_pct: f64,
    pub irq_pct: f64,
    pub softirq_pct: f64,
    pub steal_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CpuMetrics {
    pub timestamp: i64,
    pub global_usage_pct: f64,
    pub per_core: Vec<CoreMetrics>,
    pub load_avg_1: f64,
    pub load_avg_5: f64,
    pub load_avg_15: f64,
    pub load_avg_source: LoadAverageSource,
    pub mode_breakdown_supported: bool,
    pub iowait_supported: bool,
    pub steal_supported: bool,
    pub context_switches: u64,
    pub interrupts: u64,
    pub steal_pct: f64,
    pub iowait_pct: f64,
    pub modes: CpuModeMetrics,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LoadAverageSource {
    #[default]
    Unknown,
    Native,
    DerivedDemand,
}

pub struct CpuCollector {
    prev_global: Option<RawCpuStat>,
    prev_cores: Vec<RawCpuStat>,
    prev_poll: Option<Instant>,
    derived_load: Option<[f64; 3]>,
}

impl CpuCollector {
    pub fn new() -> Self {
        Self {
            prev_global: None,
            prev_cores: Vec::new(),
            prev_poll: None,
            derived_load: None,
        }
    }
}

impl Default for CpuCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Collector for CpuCollector {
    fn name(&self) -> &'static str {
        "cpu"
    }

    async fn collect(&mut self, snapshot: &mut Snapshot) -> Result<()> {
        snapshot.cpu = Some(collect_cpu(self)?);
        Ok(())
    }
}

fn delta_modes(cur: &RawCpuStat, prev: &RawCpuStat) -> CpuModeMetrics {
    let dt = cur.total().saturating_sub(prev.total());
    let pct = |n: u64| {
        if dt == 0 {
            0.0
        } else {
            (n as f64 / dt as f64 * 100.0).clamp(0.0, 100.0)
        }
    };
    CpuModeMetrics {
        user_pct: pct(cur.user.saturating_sub(prev.user)),
        nice_pct: pct(cur.nice.saturating_sub(prev.nice)),
        system_pct: pct(cur.system.saturating_sub(prev.system)),
        idle_pct: pct(cur.idle.saturating_sub(prev.idle)),
        iowait_pct: pct(cur.iowait.saturating_sub(prev.iowait)),
        irq_pct: pct(cur.irq.saturating_sub(prev.irq)),
        softirq_pct: pct(cur.softirq.saturating_sub(prev.softirq)),
        steal_pct: pct(cur.steal.saturating_sub(prev.steal)),
    }
}

fn usage_pct_from_modes(modes: &CpuModeMetrics) -> f64 {
    (modes.user_pct
        + modes.nice_pct
        + modes.system_pct
        + modes.irq_pct
        + modes.softirq_pct
        + modes.steal_pct)
        .clamp(0.0, 100.0)
}

fn map_load_avg_source(source: RawLoadAverageSource) -> LoadAverageSource {
    match source {
        RawLoadAverageSource::Unknown => LoadAverageSource::Unknown,
        RawLoadAverageSource::Native => LoadAverageSource::Native,
        RawLoadAverageSource::DerivedDemand => LoadAverageSource::DerivedDemand,
    }
}

fn smoothing_factor(elapsed_secs: f64, window_secs: f64) -> f64 {
    if elapsed_secs <= 0.0 {
        1.0
    } else {
        (1.0 - (-elapsed_secs / window_secs).exp()).clamp(0.0, 1.0)
    }
}

fn derive_load_averages(
    previous: Option<[f64; 3]>,
    instant_demand: f64,
    elapsed_secs: Option<f64>,
) -> [f64; 3] {
    let Some(previous) = previous else {
        return [instant_demand; 3];
    };

    let elapsed_secs = elapsed_secs.unwrap_or(5.0).max(0.001);
    let windows = [60.0, 300.0, 900.0];
    let mut next = previous;
    for (index, window_secs) in windows.into_iter().enumerate() {
        let alpha = smoothing_factor(elapsed_secs, window_secs);
        next[index] += alpha * (instant_demand - next[index]);
    }
    next
}

fn collect_cpu(c: &mut CpuCollector) -> Result<CpuMetrics> {
    let reading = current::read_cpu()?;
    let now = Instant::now();

    let modes = if let Some(prev) = c.prev_global.as_ref() {
        delta_modes(&reading.global, prev)
    } else {
        CpuModeMetrics::default()
    };
    let (global_usage_pct, steal_pct, iowait_pct) =
        if let Some(direct_global_usage_pct) = reading.direct_global_usage_pct {
            (
                direct_global_usage_pct,
                reading.direct_steal_pct.unwrap_or(modes.steal_pct),
                reading.direct_iowait_pct.unwrap_or(modes.iowait_pct),
            )
        } else {
            (
                usage_pct_from_modes(&modes),
                modes.steal_pct,
                modes.iowait_pct,
            )
        };

    let per_core: Vec<CoreMetrics> = if !reading.direct_per_core_usage_pct.is_empty() {
        reading
            .direct_per_core_usage_pct
            .iter()
            .enumerate()
            .map(|(id, usage_pct)| CoreMetrics {
                id,
                usage_pct: usage_pct.clamp(0.0, 100.0),
            })
            .collect()
    } else {
        reading
            .cores
            .iter()
            .enumerate()
            .map(|(id, stat)| CoreMetrics {
                id,
                usage_pct: c
                    .prev_cores
                    .get(id)
                    .map(|prev| usage_pct_from_modes(&delta_modes(stat, prev)))
                    .unwrap_or(0.0),
            })
            .collect()
    };

    let load_avg_source = map_load_avg_source(reading.load_avg_source);
    let mode_breakdown_supported = reading.global.total() > 0;
    let iowait_supported = reading.direct_iowait_pct.is_some() || mode_breakdown_supported;
    let steal_supported = reading.direct_steal_pct.is_some() || mode_breakdown_supported;
    let core_count = per_core.len().max(1) as f64;
    let elapsed_secs = c
        .prev_poll
        .map(|previous| now.duration_since(previous).as_secs_f64());
    let derived_load = if load_avg_source == LoadAverageSource::DerivedDemand {
        let instant_demand = (global_usage_pct / 100.0) * core_count;
        let next = derive_load_averages(c.derived_load, instant_demand, elapsed_secs);
        c.derived_load = Some(next);
        next
    } else {
        c.derived_load = None;
        [reading.load_avg_1, reading.load_avg_5, reading.load_avg_15]
    };

    c.prev_global = Some(reading.global.clone());
    c.prev_cores = reading.cores;
    c.prev_poll = Some(now);

    Ok(CpuMetrics {
        timestamp: chrono::Utc::now().timestamp(),
        global_usage_pct,
        per_core,
        load_avg_1: derived_load[0],
        load_avg_5: derived_load[1],
        load_avg_15: derived_load[2],
        load_avg_source,
        mode_breakdown_supported,
        iowait_supported,
        steal_supported,
        context_switches: reading.context_switches,
        interrupts: reading.interrupts,
        steal_pct,
        iowait_pct,
        modes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn cpu_collector_populates_snapshot_on_linux() {
        let mut collector = CpuCollector::new();
        let mut snapshot = Snapshot::default();

        collector.collect(&mut snapshot).await.unwrap();

        let cpu = snapshot.cpu.expect("cpu metrics should be present");
        assert!(cpu.global_usage_pct.is_finite());
        assert!(cpu.steal_pct.is_finite());
        assert!(cpu.iowait_pct.is_finite());
        assert!(cpu.modes.user_pct.is_finite());
        assert!(cpu.modes.system_pct.is_finite());
        assert!(cpu.modes.idle_pct.is_finite());
        assert!(cpu.load_avg_1.is_finite());
        assert!(cpu.load_avg_5.is_finite());
        assert!(cpu.load_avg_15.is_finite());

        for (index, core) in cpu.per_core.iter().enumerate() {
            assert_eq!(core.id, index);
            assert!(core.usage_pct.is_finite());
            assert!((0.0..=100.0).contains(&core.usage_pct));
        }
    }

    #[test]
    fn derived_load_averages_initialize_from_instant_demand() {
        let derived = derive_load_averages(None, 2.5, None);
        assert_eq!(derived, [2.5, 2.5, 2.5]);
    }

    #[test]
    fn derived_load_averages_smooth_over_time() {
        let derived = derive_load_averages(Some([1.0, 1.0, 1.0]), 3.0, Some(5.0));
        assert!(derived[0] > 1.0);
        assert!(derived[1] > 1.0);
        assert!(derived[2] > 1.0);
        assert!(derived[0] > derived[1]);
        assert!(derived[1] > derived[2]);
        assert!(derived[0] < 3.0);
    }
}
