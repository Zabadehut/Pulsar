use anyhow::Result;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RawLoadAverageSource {
    #[default]
    Unknown,
    Native,
    DerivedDemand,
}

/// Raw CPU counter snapshot for delta computation.
/// All platforms must produce this shape; zero-filled on unsupported OS.
#[derive(Debug, Clone, Default)]
pub struct RawCpuStat {
    pub user: u64,
    pub nice: u64,
    pub system: u64,
    pub idle: u64,
    pub iowait: u64,
    pub irq: u64,
    pub softirq: u64,
    pub steal: u64,
}

impl RawCpuStat {
    pub fn total(&self) -> u64 {
        self.user
            + self.nice
            + self.system
            + self.idle
            + self.iowait
            + self.irq
            + self.softirq
            + self.steal
    }
}

/// Full CPU reading from one poll — global + per-core + load + misc counters.
#[derive(Debug, Default)]
pub struct RawCpuReading {
    pub global: RawCpuStat,
    pub cores: Vec<RawCpuStat>,
    pub load_avg_1: f64,
    pub load_avg_5: f64,
    pub load_avg_15: f64,
    pub load_avg_source: RawLoadAverageSource,
    pub context_switches: u64,
    pub interrupts: u64,
    pub direct_global_usage_pct: Option<f64>,
    pub direct_iowait_pct: Option<f64>,
    pub direct_steal_pct: Option<f64>,
    pub direct_per_core_usage_pct: Vec<f64>,
}

/// One disk device reading (counter snapshot, not rate).
#[derive(Debug, Clone, Default)]
pub struct RawDiskStat {
    pub device: String,
    pub reads_completed: u64,
    pub reads_merged: u64,
    pub reads_sectors: u64,
    pub reads_time_ms: u64,
    pub writes_completed: u64,
    pub writes_merged: u64,
    pub writes_sectors: u64,
    pub writes_time_ms: u64,
    pub io_ticks: u64,
    pub weighted_io_time_ms: u64,
}

#[derive(Debug, Clone, Default)]
pub struct RawDiskInventory {
    pub device: String,
    pub parent: Option<String>,
    pub structure: String,
    pub volume_kind: String,
    pub filesystem: String,
    pub filesystem_family: String,
    pub label: String,
    pub uuid: String,
    pub part_uuid: String,
    pub model: String,
    pub serial: String,
    pub transport: String,
    pub reference: String,
    pub scheduler: String,
    pub rotational: Option<bool>,
    pub removable: Option<bool>,
    pub read_only: Option<bool>,
    pub mount_points: Vec<String>,
    pub logical_stack: Vec<String>,
    pub slaves: Vec<String>,
    pub holders: Vec<String>,
    pub children: Vec<String>,
}

/// One network interface reading (counter snapshot, not rate).
#[derive(Debug, Clone, Default)]
pub struct RawNetStat {
    pub interface: String,
    pub rx_bytes: u64,
    pub rx_packets: u64,
    pub rx_errors: u64,
    pub rx_dropped: u64,
    pub tx_bytes: u64,
    pub tx_packets: u64,
    pub tx_errors: u64,
    pub tx_dropped: u64,
}

/// Connection counts (all TCP sockets + established ones).
#[derive(Debug, Default)]
pub struct RawNetConnections {
    pub total: u32,
    pub established: u32,
    pub tcp_syn_sent: u32,
    pub tcp_syn_recv: u32,
    pub tcp_fin_wait1: u32,
    pub tcp_fin_wait2: u32,
    pub tcp_time_wait: u32,
    pub tcp_close: u32,
    pub tcp_close_wait: u32,
    pub tcp_last_ack: u32,
    pub tcp_listen: u32,
    pub tcp_closing: u32,
    pub tcp_other: u32,
    pub udp_total: u32,
    pub udp_established: u32,
    pub udp_close: u32,
    pub udp_other: u32,
    pub retrans_segs: u64,
    pub tcp_state_breakdown_supported: bool,
    pub udp_breakdown_supported: bool,
    pub retrans_supported: bool,
}

/// Disk space from statvfs/equivalent.
#[derive(Debug, Default)]
pub struct RawDiskSpace {
    pub total_gb: f64,
    pub used_gb: f64,
    pub free_gb: f64,
    pub usage_pct: f64,
}

#[derive(Debug, Clone, Default)]
pub struct RawMemoryInfo {
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

#[derive(Debug, Clone, Default)]
pub struct RawSystemInfo {
    pub hostname: String,
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub uptime_seconds: u64,
    pub cpu_count: u32,
    pub architecture: String,
}

#[derive(Debug, Clone, Default)]
pub struct RawPressureWindow {
    pub avg10: f64,
    pub avg60: f64,
    pub avg300: f64,
    pub total: u64,
}

#[derive(Debug, Clone, Default)]
pub struct RawPressureMetric {
    pub some: Option<RawPressureWindow>,
    pub full: Option<RawPressureWindow>,
}

#[derive(Debug, Clone, Default)]
pub struct RawPsiMetrics {
    pub cpu: RawPressureMetric,
    pub memory: RawPressureMetric,
    pub io: RawPressureMetric,
}

#[derive(Debug, Clone, Default)]
pub struct RawCgroupMetrics {
    pub version: u8,
    pub path: String,
    pub memory_current_bytes: u64,
    pub memory_max_bytes: Option<u64>,
    pub memory_swap_current_bytes: u64,
    pub memory_swap_max_bytes: Option<u64>,
    pub memory_usage_pct: f64,
    pub pids_current: u64,
    pub pids_max: Option<u64>,
    pub cpu_usage_usec: u64,
    pub cpu_user_usec: u64,
    pub cpu_system_usec: u64,
    pub cpu_nr_periods: u64,
    pub cpu_nr_throttled: u64,
    pub cpu_throttled_usec: u64,
    pub cpu_quota_usec: Option<u64>,
    pub cpu_period_usec: Option<u64>,
}

#[derive(Debug, Clone, Default)]
pub struct RawLinuxMetrics {
    pub cgroup: Option<RawCgroupMetrics>,
    pub psi: Option<RawPsiMetrics>,
}

/// One process reading (counter snapshot for CPU delta, not rate).
#[derive(Debug, Clone, Default)]
pub struct RawProcReading {
    pub pid: u32,
    pub name: String,
    pub cmdline: String,
    pub utime: u64,
    pub stime: u64,
    pub vsize: u64, // bytes
    pub rss: u64,   // pages
    pub threads: u32,
    pub fd_count: u32,
    pub state_char: char,
    pub user: String,
    pub io_read_bytes: u64,
    pub io_write_bytes: u64,
    pub cpu_pct_hint: Option<f64>,
    pub fd_count_supported: bool,
    pub io_bytes_supported: bool,
}

#[allow(dead_code)]
pub trait PlatformCollect {
    fn read_cpu() -> Result<RawCpuReading>;
    fn read_disks() -> Result<Vec<RawDiskStat>>;
    fn read_disk_inventory() -> Result<Vec<RawDiskInventory>>;
    fn read_network() -> Result<Vec<RawNetStat>>;
    fn read_memory() -> Result<RawMemoryInfo>;
    fn read_system() -> Result<RawSystemInfo>;
    fn read_processes() -> Result<Vec<RawProcReading>>;
    fn read_linux_metrics() -> Result<RawLinuxMetrics>;
}
