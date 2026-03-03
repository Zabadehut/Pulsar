use crate::collectors::Snapshot;
use crate::exporters::Exporter;
use anyhow::Result;

pub struct CsvExporter;

impl Exporter for CsvExporter {
    fn name(&self) -> &'static str {
        "csv"
    }

    fn export(&self, snapshot: &Snapshot) -> Result<String> {
        let mut out = String::new();
        out.push_str("timestamp,metric,value\n");

        if let Some(cpu) = &snapshot.cpu {
            out.push_str(&format!(
                "{},cpu.global_usage_pct,{:.2}\n",
                snapshot.timestamp, cpu.global_usage_pct
            ));
            out.push_str(&format!(
                "{},cpu.load_avg_1,{:.2}\n",
                snapshot.timestamp, cpu.load_avg_1
            ));
            out.push_str(&format!(
                "{},cpu.load_avg_5,{:.2}\n",
                snapshot.timestamp, cpu.load_avg_5
            ));
            out.push_str(&format!(
                "{},cpu.load_avg_15,{:.2}\n",
                snapshot.timestamp, cpu.load_avg_15
            ));
            out.push_str(&format!(
                "{},cpu.iowait_pct,{:.2}\n",
                snapshot.timestamp, cpu.iowait_pct
            ));
            out.push_str(&format!(
                "{},cpu.steal_pct,{:.2}\n",
                snapshot.timestamp, cpu.steal_pct
            ));
            out.push_str(&format!(
                "{},cpu.mode.user_pct,{:.2}\n",
                snapshot.timestamp, cpu.modes.user_pct
            ));
            out.push_str(&format!(
                "{},cpu.mode.nice_pct,{:.2}\n",
                snapshot.timestamp, cpu.modes.nice_pct
            ));
            out.push_str(&format!(
                "{},cpu.mode.system_pct,{:.2}\n",
                snapshot.timestamp, cpu.modes.system_pct
            ));
            out.push_str(&format!(
                "{},cpu.mode.idle_pct,{:.2}\n",
                snapshot.timestamp, cpu.modes.idle_pct
            ));
            out.push_str(&format!(
                "{},cpu.mode.irq_pct,{:.2}\n",
                snapshot.timestamp, cpu.modes.irq_pct
            ));
            out.push_str(&format!(
                "{},cpu.mode.softirq_pct,{:.2}\n",
                snapshot.timestamp, cpu.modes.softirq_pct
            ));
            out.push_str(&format!(
                "{},cpu.context_switches,{}\n",
                snapshot.timestamp, cpu.context_switches
            ));
            out.push_str(&format!(
                "{},cpu.interrupts,{}\n",
                snapshot.timestamp, cpu.interrupts
            ));
            for core in &cpu.per_core {
                out.push_str(&format!(
                    "{},cpu.core_{}.usage_pct,{:.2}\n",
                    snapshot.timestamp, core.id, core.usage_pct
                ));
            }
        }

        if let Some(mem) = &snapshot.memory {
            out.push_str(&format!(
                "{},mem.total_kb,{}\n",
                snapshot.timestamp, mem.total_kb
            ));
            out.push_str(&format!(
                "{},mem.used_kb,{}\n",
                snapshot.timestamp, mem.used_kb
            ));
            out.push_str(&format!(
                "{},mem.usage_pct,{:.2}\n",
                snapshot.timestamp, mem.usage_pct
            ));
            out.push_str(&format!(
                "{},mem.available_kb,{}\n",
                snapshot.timestamp, mem.available_kb
            ));
            out.push_str(&format!(
                "{},mem.cached_kb,{}\n",
                snapshot.timestamp, mem.cached_kb
            ));
            out.push_str(&format!(
                "{},mem.buffers_kb,{}\n",
                snapshot.timestamp, mem.buffers_kb
            ));
            out.push_str(&format!(
                "{},mem.swap_used_kb,{}\n",
                snapshot.timestamp, mem.swap_used_kb
            ));
            out.push_str(&format!(
                "{},mem.dirty_kb,{}\n",
                snapshot.timestamp, mem.dirty_kb
            ));
            out.push_str(&format!(
                "{},mem.vm_pgfault,{}\n",
                snapshot.timestamp, mem.vm_pgfault
            ));
            out.push_str(&format!(
                "{},mem.vm_pgmajfault,{}\n",
                snapshot.timestamp, mem.vm_pgmajfault
            ));
            out.push_str(&format!(
                "{},mem.vm_pgpgin,{}\n",
                snapshot.timestamp, mem.vm_pgpgin
            ));
            out.push_str(&format!(
                "{},mem.vm_pgpgout,{}\n",
                snapshot.timestamp, mem.vm_pgpgout
            ));
            out.push_str(&format!(
                "{},mem.vm_pswpin,{}\n",
                snapshot.timestamp, mem.vm_pswpin
            ));
            out.push_str(&format!(
                "{},mem.vm_pswpout,{}\n",
                snapshot.timestamp, mem.vm_pswpout
            ));
            out.push_str(&format!(
                "{},mem.vm_pgscan,{}\n",
                snapshot.timestamp, mem.vm_pgscan
            ));
            out.push_str(&format!(
                "{},mem.vm_pgsteal,{}\n",
                snapshot.timestamp, mem.vm_pgsteal
            ));
        }

        for disk in &snapshot.disks {
            let device = metric_label(&disk.device);
            out.push_str(&format!(
                "{},disk.{}.usage_pct,{:.2}\n",
                snapshot.timestamp, device, disk.usage_pct
            ));
            out.push_str(&format!(
                "{},disk.{}.read_iops,{}\n",
                snapshot.timestamp, device, disk.read_iops
            ));
            out.push_str(&format!(
                "{},disk.{}.write_iops,{}\n",
                snapshot.timestamp, device, disk.write_iops
            ));
            out.push_str(&format!(
                "{},disk.{}.read_kb_sec,{}\n",
                snapshot.timestamp, device, disk.read_throughput_kb
            ));
            out.push_str(&format!(
                "{},disk.{}.write_kb_sec,{}\n",
                snapshot.timestamp, device, disk.write_throughput_kb
            ));
            out.push_str(&format!(
                "{},disk.{}.await_ms,{:.2}\n",
                snapshot.timestamp, device, disk.await_ms
            ));
            out.push_str(&format!(
                "{},disk.{}.service_time_ms,{:.2}\n",
                snapshot.timestamp, device, disk.service_time_ms
            ));
            out.push_str(&format!(
                "{},disk.{}.queue_depth,{:.4}\n",
                snapshot.timestamp, device, disk.queue_depth
            ));
            out.push_str(&format!(
                "{},disk.{}.util_pct,{:.2}\n",
                snapshot.timestamp, device, disk.util_pct
            ));
            out.push_str(&format!(
                "{},disk.{}.read_merged_ops_sec,{}\n",
                snapshot.timestamp, device, disk.read_merged_ops_sec
            ));
            out.push_str(&format!(
                "{},disk.{}.write_merged_ops_sec,{}\n",
                snapshot.timestamp, device, disk.write_merged_ops_sec
            ));
        }

        for net in &snapshot.networks {
            let iface = metric_label(&net.interface);
            out.push_str(&format!(
                "{},net.{}.topology.{},1\n",
                snapshot.timestamp,
                iface,
                metric_label(&net.topology_hint)
            ));
            out.push_str(&format!(
                "{},net.{}.family.{},1\n",
                snapshot.timestamp,
                iface,
                metric_label(&net.family_hint)
            ));
            out.push_str(&format!(
                "{},net.{}.medium.{},1\n",
                snapshot.timestamp,
                iface,
                metric_label(&net.medium_hint)
            ));
            out.push_str(&format!(
                "{},net.{}.rx_bytes_sec,{}\n",
                snapshot.timestamp, iface, net.rx_bytes_sec
            ));
            out.push_str(&format!(
                "{},net.{}.tx_bytes_sec,{}\n",
                snapshot.timestamp, iface, net.tx_bytes_sec
            ));
            out.push_str(&format!(
                "{},net.{}.rx_packets_sec,{}\n",
                snapshot.timestamp, iface, net.rx_packets_sec
            ));
            out.push_str(&format!(
                "{},net.{}.tx_packets_sec,{}\n",
                snapshot.timestamp, iface, net.tx_packets_sec
            ));
            out.push_str(&format!(
                "{},net.{}.errors,{}\n",
                snapshot.timestamp,
                iface,
                net.rx_errors + net.tx_errors
            ));
            out.push_str(&format!(
                "{},net.{}.drops,{}\n",
                snapshot.timestamp,
                iface,
                net.rx_dropped + net.tx_dropped
            ));
            out.push_str(&format!(
                "{},net.{}.connections_total,{}\n",
                snapshot.timestamp, iface, net.connections_total
            ));
            out.push_str(&format!(
                "{},net.{}.connections_established,{}\n",
                snapshot.timestamp, iface, net.connections_established
            ));
            out.push_str(&format!(
                "{},net.{}.tcp_listen,{}\n",
                snapshot.timestamp, iface, net.tcp_listen
            ));
            out.push_str(&format!(
                "{},net.{}.tcp_time_wait,{}\n",
                snapshot.timestamp, iface, net.tcp_time_wait
            ));
            out.push_str(&format!(
                "{},net.{}.tcp_close_wait,{}\n",
                snapshot.timestamp, iface, net.tcp_close_wait
            ));
            out.push_str(&format!(
                "{},net.{}.tcp_syn_sent,{}\n",
                snapshot.timestamp, iface, net.tcp_syn_sent
            ));
            out.push_str(&format!(
                "{},net.{}.tcp_syn_recv,{}\n",
                snapshot.timestamp, iface, net.tcp_syn_recv
            ));
            out.push_str(&format!(
                "{},net.{}.tcp_fin_wait1,{}\n",
                snapshot.timestamp, iface, net.tcp_fin_wait1
            ));
            out.push_str(&format!(
                "{},net.{}.tcp_fin_wait2,{}\n",
                snapshot.timestamp, iface, net.tcp_fin_wait2
            ));
            out.push_str(&format!(
                "{},net.{}.tcp_last_ack,{}\n",
                snapshot.timestamp, iface, net.tcp_last_ack
            ));
            out.push_str(&format!(
                "{},net.{}.tcp_closing,{}\n",
                snapshot.timestamp, iface, net.tcp_closing
            ));
            out.push_str(&format!(
                "{},net.{}.tcp_close,{}\n",
                snapshot.timestamp, iface, net.tcp_close
            ));
            out.push_str(&format!(
                "{},net.{}.tcp_other,{}\n",
                snapshot.timestamp, iface, net.tcp_other
            ));
            out.push_str(&format!(
                "{},net.{}.udp_total,{}\n",
                snapshot.timestamp, iface, net.udp_total
            ));
            out.push_str(&format!(
                "{},net.{}.udp_established,{}\n",
                snapshot.timestamp, iface, net.udp_established
            ));
            out.push_str(&format!(
                "{},net.{}.udp_close,{}\n",
                snapshot.timestamp, iface, net.udp_close
            ));
            out.push_str(&format!(
                "{},net.{}.udp_other,{}\n",
                snapshot.timestamp, iface, net.udp_other
            ));
            out.push_str(&format!(
                "{},net.{}.retrans_segs,{}\n",
                snapshot.timestamp, iface, net.retrans_segs
            ));
        }

        for process in &snapshot.processes {
            out.push_str(&format!(
                "{},process.{}.cpu_pct,{:.2}\n",
                snapshot.timestamp, process.pid, process.cpu_pct
            ));
            out.push_str(&format!(
                "{},process.{}.rss_kb,{}\n",
                snapshot.timestamp, process.pid, process.mem_rss_kb
            ));
            out.push_str(&format!(
                "{},process.{}.threads,{}\n",
                snapshot.timestamp, process.pid, process.threads
            ));
            out.push_str(&format!(
                "{},process.{}.fds,{}\n",
                snapshot.timestamp, process.pid, process.fd_count
            ));
            out.push_str(&format!(
                "{},process.{}.io_read_bytes,{}\n",
                snapshot.timestamp, process.pid, process.io_read_bytes
            ));
            out.push_str(&format!(
                "{},process.{}.io_write_bytes,{}\n",
                snapshot.timestamp, process.pid, process.io_write_bytes
            ));
        }

        if let Some(system) = &snapshot.system {
            out.push_str(&format!(
                "{},system.uptime_seconds,{}\n",
                snapshot.timestamp, system.uptime_seconds
            ));
            out.push_str(&format!(
                "{},system.cpu_count,{}\n",
                snapshot.timestamp, system.cpu_count
            ));
        }

        if let Some(linux) = &snapshot.linux {
            if let Some(cgroup) = &linux.cgroup {
                out.push_str(&format!(
                    "{},linux.cgroup.memory_current_bytes,{}\n",
                    snapshot.timestamp, cgroup.memory_current_bytes
                ));
                if let Some(memory_max) = cgroup.memory_max_bytes {
                    out.push_str(&format!(
                        "{},linux.cgroup.memory_max_bytes,{}\n",
                        snapshot.timestamp, memory_max
                    ));
                }
                out.push_str(&format!(
                    "{},linux.cgroup.memory_usage_pct,{:.2}\n",
                    snapshot.timestamp, cgroup.memory_usage_pct
                ));
                out.push_str(&format!(
                    "{},linux.cgroup.pids_current,{}\n",
                    snapshot.timestamp, cgroup.pids_current
                ));
                out.push_str(&format!(
                    "{},linux.cgroup.cpu_usage_usec,{}\n",
                    snapshot.timestamp, cgroup.cpu_usage_usec
                ));
                out.push_str(&format!(
                    "{},linux.cgroup.cpu_nr_throttled,{}\n",
                    snapshot.timestamp, cgroup.cpu_nr_throttled
                ));
                out.push_str(&format!(
                    "{},linux.cgroup.cpu_throttled_usec,{}\n",
                    snapshot.timestamp, cgroup.cpu_throttled_usec
                ));
            }

            if let Some(psi) = &linux.psi {
                append_psi_csv(&mut out, snapshot.timestamp, "cpu", &psi.cpu);
                append_psi_csv(&mut out, snapshot.timestamp, "memory", &psi.memory);
                append_psi_csv(&mut out, snapshot.timestamp, "io", &psi.io);
            }
        }

        out.push_str(&format!(
            "{},computed.cpu_trend_p50,{:.2}\n",
            snapshot.timestamp, snapshot.computed.cpu_trend_p50
        ));
        out.push_str(&format!(
            "{},computed.cpu_trend_p95,{:.2}\n",
            snapshot.timestamp, snapshot.computed.cpu_trend_p95
        ));
        out.push_str(&format!(
            "{},computed.memory_pressure,{:.4}\n",
            snapshot.timestamp, snapshot.computed.memory_pressure
        ));
        out.push_str(&format!(
            "{},computed.alert_count,{}\n",
            snapshot.timestamp,
            snapshot.computed.alerts.len()
        ));
        out.push_str(&format!(
            "{},computed.alerts_info,{}\n",
            snapshot.timestamp, snapshot.computed.alerts_info
        ));
        out.push_str(&format!(
            "{},computed.alerts_warning,{}\n",
            snapshot.timestamp, snapshot.computed.alerts_warning
        ));
        out.push_str(&format!(
            "{},computed.alerts_critical,{}\n",
            snapshot.timestamp, snapshot.computed.alerts_critical
        ));

        Ok(out)
    }
}

fn metric_label(value: &str) -> String {
    value
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

fn append_psi_csv(
    out: &mut String,
    timestamp: i64,
    resource: &str,
    metric: &crate::collectors::linux::PressureMetric,
) {
    if let Some(window) = &metric.some {
        append_psi_window_csv(out, timestamp, resource, "some", window);
    }
    if let Some(window) = &metric.full {
        append_psi_window_csv(out, timestamp, resource, "full", window);
    }
}

fn append_psi_window_csv(
    out: &mut String,
    timestamp: i64,
    resource: &str,
    scope: &str,
    window: &crate::collectors::linux::PressureWindow,
) {
    out.push_str(&format!(
        "{},linux.psi.{}.{}.avg10,{:.2}\n",
        timestamp, resource, scope, window.avg10
    ));
    out.push_str(&format!(
        "{},linux.psi.{}.{}.avg60,{:.2}\n",
        timestamp, resource, scope, window.avg60
    ));
    out.push_str(&format!(
        "{},linux.psi.{}.{}.avg300,{:.2}\n",
        timestamp, resource, scope, window.avg300
    ));
    out.push_str(&format!(
        "{},linux.psi.{}.{}.total,{}\n",
        timestamp, resource, scope, window.total
    ));
}
