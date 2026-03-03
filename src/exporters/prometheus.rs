use crate::collectors::Snapshot;
use crate::exporters::Exporter;
use anyhow::Result;

pub struct PrometheusExporter;

impl Exporter for PrometheusExporter {
    fn name(&self) -> &'static str {
        "prometheus"
    }

    fn export(&self, snapshot: &Snapshot) -> Result<String> {
        let mut out = String::new();

        if let Some(cpu) = &snapshot.cpu {
            out.push_str("# HELP sysray_cpu_usage_percent CPU global usage percentage\n");
            out.push_str("# TYPE sysray_cpu_usage_percent gauge\n");
            out.push_str(&format!(
                "sysray_cpu_usage_percent {:.2}\n",
                cpu.global_usage_pct
            ));

            out.push_str("# HELP sysray_load_avg_1m Load average 1 minute\n");
            out.push_str("# TYPE sysray_load_avg_1m gauge\n");
            out.push_str(&format!("sysray_load_avg_1m {:.2}\n", cpu.load_avg_1));

            out.push_str("# HELP sysray_load_avg_5m Load average 5 minutes\n");
            out.push_str("# TYPE sysray_load_avg_5m gauge\n");
            out.push_str(&format!("sysray_load_avg_5m {:.2}\n", cpu.load_avg_5));

            out.push_str("# HELP sysray_load_avg_15m Load average 15 minutes\n");
            out.push_str("# TYPE sysray_load_avg_15m gauge\n");
            out.push_str(&format!("sysray_load_avg_15m {:.2}\n", cpu.load_avg_15));

            out.push_str("# HELP sysray_cpu_iowait_percent CPU iowait percentage\n");
            out.push_str("# TYPE sysray_cpu_iowait_percent gauge\n");
            out.push_str(&format!(
                "sysray_cpu_iowait_percent {:.2}\n",
                cpu.iowait_pct
            ));

            out.push_str("# HELP sysray_cpu_steal_percent CPU steal percentage\n");
            out.push_str("# TYPE sysray_cpu_steal_percent gauge\n");
            out.push_str(&format!("sysray_cpu_steal_percent {:.2}\n", cpu.steal_pct));

            out.push_str("# HELP sysray_cpu_mode_percent CPU percentage by scheduler mode\n");
            out.push_str("# TYPE sysray_cpu_mode_percent gauge\n");
            out.push_str(&format!(
                "sysray_cpu_mode_percent{{mode=\"user\"}} {:.2}\n",
                cpu.modes.user_pct
            ));
            out.push_str(&format!(
                "sysray_cpu_mode_percent{{mode=\"nice\"}} {:.2}\n",
                cpu.modes.nice_pct
            ));
            out.push_str(&format!(
                "sysray_cpu_mode_percent{{mode=\"system\"}} {:.2}\n",
                cpu.modes.system_pct
            ));
            out.push_str(&format!(
                "sysray_cpu_mode_percent{{mode=\"idle\"}} {:.2}\n",
                cpu.modes.idle_pct
            ));
            out.push_str(&format!(
                "sysray_cpu_mode_percent{{mode=\"iowait\"}} {:.2}\n",
                cpu.modes.iowait_pct
            ));
            out.push_str(&format!(
                "sysray_cpu_mode_percent{{mode=\"irq\"}} {:.2}\n",
                cpu.modes.irq_pct
            ));
            out.push_str(&format!(
                "sysray_cpu_mode_percent{{mode=\"softirq\"}} {:.2}\n",
                cpu.modes.softirq_pct
            ));
            out.push_str(&format!(
                "sysray_cpu_mode_percent{{mode=\"steal\"}} {:.2}\n",
                cpu.modes.steal_pct
            ));

            out.push_str(
                "# HELP sysray_cpu_context_switches_total Context switch counter snapshot\n",
            );
            out.push_str("# TYPE sysray_cpu_context_switches_total gauge\n");
            out.push_str(&format!(
                "sysray_cpu_context_switches_total {}\n",
                cpu.context_switches
            ));

            out.push_str("# HELP sysray_cpu_interrupts_total Interrupt counter snapshot\n");
            out.push_str("# TYPE sysray_cpu_interrupts_total gauge\n");
            out.push_str(&format!("sysray_cpu_interrupts_total {}\n", cpu.interrupts));

            out.push_str("# HELP sysray_cpu_core_usage_percent Per-core CPU usage percentage\n");
            out.push_str("# TYPE sysray_cpu_core_usage_percent gauge\n");
            for core in &cpu.per_core {
                out.push_str(&format!(
                    "sysray_cpu_core_usage_percent{{core=\"{}\"}} {:.2}\n",
                    core.id, core.usage_pct
                ));
            }
        }

        if let Some(mem) = &snapshot.memory {
            out.push_str("# HELP sysray_memory_total_bytes Total memory in bytes\n");
            out.push_str("# TYPE sysray_memory_total_bytes gauge\n");
            out.push_str(&format!(
                "sysray_memory_total_bytes {}\n",
                mem.total_kb * 1024
            ));

            out.push_str("# HELP sysray_memory_used_bytes Used memory in bytes\n");
            out.push_str("# TYPE sysray_memory_used_bytes gauge\n");
            out.push_str(&format!(
                "sysray_memory_used_bytes {}\n",
                mem.used_kb * 1024
            ));

            out.push_str("# HELP sysray_memory_usage_percent Memory usage percentage\n");
            out.push_str("# TYPE sysray_memory_usage_percent gauge\n");
            out.push_str(&format!(
                "sysray_memory_usage_percent {:.2}\n",
                mem.usage_pct
            ));

            out.push_str("# HELP sysray_memory_available_bytes Available memory in bytes\n");
            out.push_str("# TYPE sysray_memory_available_bytes gauge\n");
            out.push_str(&format!(
                "sysray_memory_available_bytes {}\n",
                mem.available_kb * 1024
            ));

            out.push_str("# HELP sysray_memory_cached_bytes Cached memory in bytes\n");
            out.push_str("# TYPE sysray_memory_cached_bytes gauge\n");
            out.push_str(&format!(
                "sysray_memory_cached_bytes {}\n",
                mem.cached_kb * 1024
            ));

            out.push_str("# HELP sysray_memory_buffers_bytes Buffer memory in bytes\n");
            out.push_str("# TYPE sysray_memory_buffers_bytes gauge\n");
            out.push_str(&format!(
                "sysray_memory_buffers_bytes {}\n",
                mem.buffers_kb * 1024
            ));

            out.push_str("# HELP sysray_memory_dirty_bytes Dirty memory in bytes\n");
            out.push_str("# TYPE sysray_memory_dirty_bytes gauge\n");
            out.push_str(&format!(
                "sysray_memory_dirty_bytes {}\n",
                mem.dirty_kb * 1024
            ));

            out.push_str("# HELP sysray_swap_used_bytes Swap used in bytes\n");
            out.push_str("# TYPE sysray_swap_used_bytes gauge\n");
            out.push_str(&format!(
                "sysray_swap_used_bytes {}\n",
                mem.swap_used_kb * 1024
            ));

            out.push_str("# HELP sysray_memory_vm_counters Linux VM paging counter snapshots\n");
            out.push_str("# TYPE sysray_memory_vm_counters gauge\n");
            out.push_str(&format!(
                "sysray_memory_vm_counters{{counter=\"pgfault\"}} {}\n",
                mem.vm_pgfault
            ));
            out.push_str(&format!(
                "sysray_memory_vm_counters{{counter=\"pgmajfault\"}} {}\n",
                mem.vm_pgmajfault
            ));
            out.push_str(&format!(
                "sysray_memory_vm_counters{{counter=\"pgpgin\"}} {}\n",
                mem.vm_pgpgin
            ));
            out.push_str(&format!(
                "sysray_memory_vm_counters{{counter=\"pgpgout\"}} {}\n",
                mem.vm_pgpgout
            ));
            out.push_str(&format!(
                "sysray_memory_vm_counters{{counter=\"pswpin\"}} {}\n",
                mem.vm_pswpin
            ));
            out.push_str(&format!(
                "sysray_memory_vm_counters{{counter=\"pswpout\"}} {}\n",
                mem.vm_pswpout
            ));
            out.push_str(&format!(
                "sysray_memory_vm_counters{{counter=\"pgscan\"}} {}\n",
                mem.vm_pgscan
            ));
            out.push_str(&format!(
                "sysray_memory_vm_counters{{counter=\"pgsteal\"}} {}\n",
                mem.vm_pgsteal
            ));
        }

        if !snapshot.disks.is_empty() {
            out.push_str("# HELP sysray_disk_used_bytes Used disk space in bytes\n");
            out.push_str("# TYPE sysray_disk_used_bytes gauge\n");
            out.push_str("# HELP sysray_disk_free_bytes Free disk space in bytes\n");
            out.push_str("# TYPE sysray_disk_free_bytes gauge\n");
            out.push_str("# HELP sysray_disk_usage_percent Disk usage percentage\n");
            out.push_str("# TYPE sysray_disk_usage_percent gauge\n");
            out.push_str("# HELP sysray_disk_read_iops Read IOPS\n");
            out.push_str("# TYPE sysray_disk_read_iops gauge\n");
            out.push_str("# HELP sysray_disk_write_iops Write IOPS\n");
            out.push_str("# TYPE sysray_disk_write_iops gauge\n");
            out.push_str("# HELP sysray_disk_read_throughput_kb Read throughput in KB/s\n");
            out.push_str("# TYPE sysray_disk_read_throughput_kb gauge\n");
            out.push_str("# HELP sysray_disk_write_throughput_kb Write throughput in KB/s\n");
            out.push_str("# TYPE sysray_disk_write_throughput_kb gauge\n");
            out.push_str(
                "# HELP sysray_disk_await_milliseconds Average disk await in milliseconds\n",
            );
            out.push_str("# TYPE sysray_disk_await_milliseconds gauge\n");
            out.push_str("# HELP sysray_disk_service_time_milliseconds Average disk service time in milliseconds\n");
            out.push_str("# TYPE sysray_disk_service_time_milliseconds gauge\n");
            out.push_str("# HELP sysray_disk_queue_depth Average disk queue depth\n");
            out.push_str("# TYPE sysray_disk_queue_depth gauge\n");
            out.push_str("# HELP sysray_disk_util_percent Disk utilization percentage\n");
            out.push_str("# TYPE sysray_disk_util_percent gauge\n");
            out.push_str(
                "# HELP sysray_disk_merged_ops_per_sec Merged disk operations per second\n",
            );
            out.push_str("# TYPE sysray_disk_merged_ops_per_sec gauge\n");
            out.push_str("# HELP sysray_disk_info Portable disk classification hints\n");
            out.push_str("# TYPE sysray_disk_info gauge\n");
            out.push_str("# HELP sysray_disk_inventory_info Rich disk inventory labels\n");
            out.push_str("# TYPE sysray_disk_inventory_info gauge\n");
            out.push_str("# HELP sysray_disk_stack_depth Disk logical stack depth\n");
            out.push_str("# TYPE sysray_disk_stack_depth gauge\n");
            out.push_str("# HELP sysray_disk_relation_count Disk relation counts\n");
            out.push_str("# TYPE sysray_disk_relation_count gauge\n");
            out.push_str("# HELP sysray_disk_flag Disk inventory boolean flags\n");
            out.push_str("# TYPE sysray_disk_flag gauge\n");
        }
        for disk in &snapshot.disks {
            let lbl = format!(
                r#"device="{}",mount="{}",structure="{}",protocol="{}",media="{}""#,
                escape_label(&disk.device),
                escape_label(&disk.mount_point),
                escape_label(&disk.structure_hint),
                escape_label(&disk.protocol_hint),
                escape_label(&disk.media_hint)
            );
            out.push_str(&format!("sysray_disk_info{{{}}} 1\n", lbl));
            let inventory_lbl = format!(
                r#"device="{}",mount="{}",parent="{}",structure="{}",volume_kind="{}",filesystem="{}",filesystem_family="{}",protocol="{}",media="{}",transport="{}",reference="{}",scheduler="{}""#,
                escape_label(&disk.device),
                escape_label(&disk.mount_point),
                escape_label(&disk.parent),
                escape_label(&disk.structure),
                escape_label(&disk.volume_kind),
                escape_label(&disk.filesystem),
                escape_label(&disk.filesystem_family),
                escape_label(&disk.protocol_hint),
                escape_label(&disk.media_hint),
                escape_label(&disk.protocol_hint),
                escape_label(&disk.reference),
                escape_label(&disk.scheduler)
            );
            out.push_str(&format!(
                "sysray_disk_inventory_info{{{}}} 1\n",
                inventory_lbl
            ));
            out.push_str(&format!(
                "sysray_disk_stack_depth{{device=\"{}\"}} {}\n",
                escape_label(&disk.device),
                disk.logical_stack.len()
            ));
            out.push_str(&format!(
                "sysray_disk_relation_count{{device=\"{}\",kind=\"children\"}} {}\n",
                escape_label(&disk.device),
                disk.children.len()
            ));
            out.push_str(&format!(
                "sysray_disk_relation_count{{device=\"{}\",kind=\"holders\"}} {}\n",
                escape_label(&disk.device),
                disk.holders.len()
            ));
            out.push_str(&format!(
                "sysray_disk_relation_count{{device=\"{}\",kind=\"slaves\"}} {}\n",
                escape_label(&disk.device),
                disk.slaves.len()
            ));
            out.push_str(&format!(
                "sysray_disk_flag{{device=\"{}\",flag=\"rotational\"}} {}\n",
                escape_label(&disk.device),
                u8::from(disk.rotational)
            ));
            out.push_str(&format!(
                "sysray_disk_flag{{device=\"{}\",flag=\"removable\"}} {}\n",
                escape_label(&disk.device),
                u8::from(disk.removable)
            ));
            out.push_str(&format!(
                "sysray_disk_flag{{device=\"{}\",flag=\"read_only\"}} {}\n",
                escape_label(&disk.device),
                u8::from(disk.read_only)
            ));
            out.push_str(&format!(
                "sysray_disk_used_bytes{{{}}} {:.0}\n",
                lbl,
                disk.used_gb * 1_000_000_000.0
            ));
            out.push_str(&format!(
                "sysray_disk_free_bytes{{{}}} {:.0}\n",
                lbl,
                disk.free_gb * 1_000_000_000.0
            ));
            out.push_str(&format!(
                "sysray_disk_usage_percent{{{}}} {:.2}\n",
                lbl, disk.usage_pct
            ));
            out.push_str(&format!(
                "sysray_disk_read_iops{{{}}} {}\n",
                lbl, disk.read_iops
            ));
            out.push_str(&format!(
                "sysray_disk_write_iops{{{}}} {}\n",
                lbl, disk.write_iops
            ));
            out.push_str(&format!(
                "sysray_disk_read_throughput_kb{{{}}} {}\n",
                lbl, disk.read_throughput_kb
            ));
            out.push_str(&format!(
                "sysray_disk_write_throughput_kb{{{}}} {}\n",
                lbl, disk.write_throughput_kb
            ));
            out.push_str(&format!(
                "sysray_disk_await_milliseconds{{{}}} {:.2}\n",
                lbl, disk.await_ms
            ));
            out.push_str(&format!(
                "sysray_disk_service_time_milliseconds{{{}}} {:.2}\n",
                lbl, disk.service_time_ms
            ));
            out.push_str(&format!(
                "sysray_disk_queue_depth{{{}}} {:.4}\n",
                lbl, disk.queue_depth
            ));
            out.push_str(&format!(
                "sysray_disk_util_percent{{{}}} {:.2}\n",
                lbl, disk.util_pct
            ));
            out.push_str(&format!(
                "sysray_disk_merged_ops_per_sec{{{},direction=\"read\"}} {}\n",
                lbl, disk.read_merged_ops_sec
            ));
            out.push_str(&format!(
                "sysray_disk_merged_ops_per_sec{{{},direction=\"write\"}} {}\n",
                lbl, disk.write_merged_ops_sec
            ));
        }

        if !snapshot.networks.is_empty() {
            out.push_str(
                "# HELP sysray_network_rx_bytes_sec Network receive throughput in bytes/s\n",
            );
            out.push_str("# TYPE sysray_network_rx_bytes_sec gauge\n");
            out.push_str(
                "# HELP sysray_network_tx_bytes_sec Network transmit throughput in bytes/s\n",
            );
            out.push_str("# TYPE sysray_network_tx_bytes_sec gauge\n");
            out.push_str(
                "# HELP sysray_network_rx_packets_sec Network receive packets per second\n",
            );
            out.push_str("# TYPE sysray_network_rx_packets_sec gauge\n");
            out.push_str(
                "# HELP sysray_network_tx_packets_sec Network transmit packets per second\n",
            );
            out.push_str("# TYPE sysray_network_tx_packets_sec gauge\n");
            out.push_str("# HELP sysray_network_errors_total Network interface errors\n");
            out.push_str("# TYPE sysray_network_errors_total gauge\n");
            out.push_str("# HELP sysray_network_drops_total Network interface drops\n");
            out.push_str("# TYPE sysray_network_drops_total gauge\n");
            out.push_str("# HELP sysray_network_connections_total TCP connection count\n");
            out.push_str("# TYPE sysray_network_connections_total gauge\n");
            out.push_str(
                "# HELP sysray_network_connections_established TCP established connection count\n",
            );
            out.push_str("# TYPE sysray_network_connections_established gauge\n");
            out.push_str("# HELP sysray_network_tcp_state_count TCP socket states\n");
            out.push_str("# TYPE sysray_network_tcp_state_count gauge\n");
            out.push_str("# HELP sysray_network_udp_state_count UDP socket states\n");
            out.push_str("# TYPE sysray_network_udp_state_count gauge\n");
            out.push_str("# HELP sysray_network_tcp_retrans_segments TCP retransmitted segments\n");
            out.push_str("# TYPE sysray_network_tcp_retrans_segments gauge\n");
            out.push_str(
                "# HELP sysray_network_interface_info Portable interface classification hints\n",
            );
            out.push_str("# TYPE sysray_network_interface_info gauge\n");
            out.push_str(
                "# HELP sysray_network_inventory_flag Derived interface inventory flags\n",
            );
            out.push_str("# TYPE sysray_network_inventory_flag gauge\n");
        }
        for net in &snapshot.networks {
            let lbl = format!(
                r#"interface="{}",topology="{}",family="{}",medium="{}""#,
                escape_label(&net.interface),
                escape_label(&net.topology_hint),
                escape_label(&net.family_hint),
                escape_label(&net.medium_hint)
            );
            out.push_str(&format!("sysray_network_interface_info{{{}}} 1\n", lbl));
            out.push_str(&format!(
                "sysray_network_inventory_flag{{interface=\"{}\",flag=\"loopback\"}} {}\n",
                escape_label(&net.interface),
                u8::from(net.topology_hint == "loopback")
            ));
            out.push_str(&format!(
                "sysray_network_inventory_flag{{interface=\"{}\",flag=\"virtual\"}} {}\n",
                escape_label(&net.interface),
                u8::from(net.medium_hint == "virtual" || net.medium_hint == "software")
            ));
            out.push_str(&format!(
                "sysray_network_inventory_flag{{interface=\"{}\",flag=\"wireless\"}} {}\n",
                escape_label(&net.interface),
                u8::from(net.family_hint == "wireless")
            ));
            out.push_str(&format!(
                "sysray_network_inventory_flag{{interface=\"{}\",flag=\"overlay\"}} {}\n",
                escape_label(&net.interface),
                u8::from(net.medium_hint == "overlay")
            ));
            out.push_str(&format!(
                "sysray_network_rx_bytes_sec{{{}}} {}\n",
                lbl, net.rx_bytes_sec
            ));
            out.push_str(&format!(
                "sysray_network_tx_bytes_sec{{{}}} {}\n",
                lbl, net.tx_bytes_sec
            ));
            out.push_str(&format!(
                "sysray_network_rx_packets_sec{{{}}} {}\n",
                lbl, net.rx_packets_sec
            ));
            out.push_str(&format!(
                "sysray_network_tx_packets_sec{{{}}} {}\n",
                lbl, net.tx_packets_sec
            ));
            out.push_str(&format!(
                "sysray_network_errors_total{{{}}} {}\n",
                lbl,
                net.rx_errors + net.tx_errors
            ));
            out.push_str(&format!(
                "sysray_network_drops_total{{{}}} {}\n",
                lbl,
                net.rx_dropped + net.tx_dropped
            ));
            out.push_str(&format!(
                "sysray_network_connections_total{{{}}} {}\n",
                lbl, net.connections_total
            ));
            out.push_str(&format!(
                "sysray_network_connections_established{{{}}} {}\n",
                lbl, net.connections_established
            ));
            out.push_str(&format!(
                "sysray_network_tcp_state_count{{{},state=\"established\"}} {}\n",
                lbl, net.connections_established
            ));
            out.push_str(&format!(
                "sysray_network_tcp_state_count{{{},state=\"listen\"}} {}\n",
                lbl, net.tcp_listen
            ));
            out.push_str(&format!(
                "sysray_network_tcp_state_count{{{},state=\"time_wait\"}} {}\n",
                lbl, net.tcp_time_wait
            ));
            out.push_str(&format!(
                "sysray_network_tcp_state_count{{{},state=\"close_wait\"}} {}\n",
                lbl, net.tcp_close_wait
            ));
            out.push_str(&format!(
                "sysray_network_tcp_state_count{{{},state=\"syn_sent\"}} {}\n",
                lbl, net.tcp_syn_sent
            ));
            out.push_str(&format!(
                "sysray_network_tcp_state_count{{{},state=\"syn_recv\"}} {}\n",
                lbl, net.tcp_syn_recv
            ));
            out.push_str(&format!(
                "sysray_network_tcp_state_count{{{},state=\"fin_wait1\"}} {}\n",
                lbl, net.tcp_fin_wait1
            ));
            out.push_str(&format!(
                "sysray_network_tcp_state_count{{{},state=\"fin_wait2\"}} {}\n",
                lbl, net.tcp_fin_wait2
            ));
            out.push_str(&format!(
                "sysray_network_tcp_state_count{{{},state=\"last_ack\"}} {}\n",
                lbl, net.tcp_last_ack
            ));
            out.push_str(&format!(
                "sysray_network_tcp_state_count{{{},state=\"closing\"}} {}\n",
                lbl, net.tcp_closing
            ));
            out.push_str(&format!(
                "sysray_network_tcp_state_count{{{},state=\"close\"}} {}\n",
                lbl, net.tcp_close
            ));
            out.push_str(&format!(
                "sysray_network_tcp_state_count{{{},state=\"other\"}} {}\n",
                lbl, net.tcp_other
            ));
            out.push_str(&format!(
                "sysray_network_udp_state_count{{{},state=\"total\"}} {}\n",
                lbl, net.udp_total
            ));
            out.push_str(&format!(
                "sysray_network_udp_state_count{{{},state=\"established\"}} {}\n",
                lbl, net.udp_established
            ));
            out.push_str(&format!(
                "sysray_network_udp_state_count{{{},state=\"close\"}} {}\n",
                lbl, net.udp_close
            ));
            out.push_str(&format!(
                "sysray_network_udp_state_count{{{},state=\"other\"}} {}\n",
                lbl, net.udp_other
            ));
            out.push_str(&format!(
                "sysray_network_tcp_retrans_segments{{{}}} {}\n",
                lbl, net.retrans_segs
            ));
        }

        if let Some(system) = &snapshot.system {
            out.push_str("# HELP sysray_system_uptime_seconds System uptime in seconds\n");
            out.push_str("# TYPE sysray_system_uptime_seconds gauge\n");
            out.push_str(&format!(
                "sysray_system_uptime_seconds {}\n",
                system.uptime_seconds
            ));
            out.push_str("# HELP sysray_system_cpu_count System CPU count\n");
            out.push_str("# TYPE sysray_system_cpu_count gauge\n");
            out.push_str(&format!("sysray_system_cpu_count {}\n", system.cpu_count));
            out.push_str("# HELP sysray_system_info System metadata labels\n");
            out.push_str("# TYPE sysray_system_info gauge\n");
            out.push_str(&format!(
                "sysray_system_info{{hostname=\"{}\",os=\"{}\",os_version=\"{}\",kernel=\"{}\",arch=\"{}\"}} 1\n",
                escape_label(&system.hostname),
                escape_label(&system.os_name),
                escape_label(&system.os_version),
                escape_label(&system.kernel_version),
                escape_label(&system.architecture)
            ));
        }

        if let Some(linux) = &snapshot.linux {
            if let Some(cgroup) = &linux.cgroup {
                out.push_str("# HELP sysray_linux_cgroup_info Linux cgroup metadata labels\n");
                out.push_str("# TYPE sysray_linux_cgroup_info gauge\n");
                out.push_str(&format!(
                    "sysray_linux_cgroup_info{{version=\"{}\",path=\"{}\"}} 1\n",
                    cgroup.version,
                    escape_label(&cgroup.path)
                ));
                out.push_str("# HELP sysray_linux_cgroup_memory_current_bytes Current cgroup memory usage in bytes\n");
                out.push_str("# TYPE sysray_linux_cgroup_memory_current_bytes gauge\n");
                out.push_str(&format!(
                    "sysray_linux_cgroup_memory_current_bytes {}\n",
                    cgroup.memory_current_bytes
                ));
                if let Some(memory_max) = cgroup.memory_max_bytes {
                    out.push_str("# HELP sysray_linux_cgroup_memory_max_bytes Maximum cgroup memory in bytes\n");
                    out.push_str("# TYPE sysray_linux_cgroup_memory_max_bytes gauge\n");
                    out.push_str(&format!(
                        "sysray_linux_cgroup_memory_max_bytes {}\n",
                        memory_max
                    ));
                }
                out.push_str("# HELP sysray_linux_cgroup_memory_usage_percent Cgroup memory usage percentage against the configured limit\n");
                out.push_str("# TYPE sysray_linux_cgroup_memory_usage_percent gauge\n");
                out.push_str(&format!(
                    "sysray_linux_cgroup_memory_usage_percent {:.2}\n",
                    cgroup.memory_usage_pct
                ));
                out.push_str("# HELP sysray_linux_cgroup_pids_current Current cgroup pid usage\n");
                out.push_str("# TYPE sysray_linux_cgroup_pids_current gauge\n");
                out.push_str(&format!(
                    "sysray_linux_cgroup_pids_current {}\n",
                    cgroup.pids_current
                ));
                if let Some(pids_max) = cgroup.pids_max {
                    out.push_str("# HELP sysray_linux_cgroup_pids_max Maximum cgroup pid limit\n");
                    out.push_str("# TYPE sysray_linux_cgroup_pids_max gauge\n");
                    out.push_str(&format!("sysray_linux_cgroup_pids_max {}\n", pids_max));
                }
                out.push_str("# HELP sysray_linux_cgroup_cpu_usage_usec Total cgroup CPU usage in microseconds\n");
                out.push_str("# TYPE sysray_linux_cgroup_cpu_usage_usec gauge\n");
                out.push_str(&format!(
                    "sysray_linux_cgroup_cpu_usage_usec {}\n",
                    cgroup.cpu_usage_usec
                ));
                out.push_str("# HELP sysray_linux_cgroup_cpu_nr_throttled Number of throttled CPU periods in the cgroup\n");
                out.push_str("# TYPE sysray_linux_cgroup_cpu_nr_throttled gauge\n");
                out.push_str(&format!(
                    "sysray_linux_cgroup_cpu_nr_throttled {}\n",
                    cgroup.cpu_nr_throttled
                ));
                out.push_str("# HELP sysray_linux_cgroup_cpu_throttled_usec Total throttled CPU time in microseconds\n");
                out.push_str("# TYPE sysray_linux_cgroup_cpu_throttled_usec gauge\n");
                out.push_str(&format!(
                    "sysray_linux_cgroup_cpu_throttled_usec {}\n",
                    cgroup.cpu_throttled_usec
                ));
            }

            append_pressure_metric(&mut out, "cpu", &linux.psi.as_ref().map(|psi| &psi.cpu));
            append_pressure_metric(
                &mut out,
                "memory",
                &linux.psi.as_ref().map(|psi| &psi.memory),
            );
            append_pressure_metric(&mut out, "io", &linux.psi.as_ref().map(|psi| &psi.io));
        }

        out.push_str("# HELP sysray_computed_cpu_trend_p50 CPU usage rolling p50\n");
        out.push_str("# TYPE sysray_computed_cpu_trend_p50 gauge\n");
        out.push_str(&format!(
            "sysray_computed_cpu_trend_p50 {:.2}\n",
            snapshot.computed.cpu_trend_p50
        ));
        out.push_str("# HELP sysray_computed_cpu_trend_p95 CPU usage rolling p95\n");
        out.push_str("# TYPE sysray_computed_cpu_trend_p95 gauge\n");
        out.push_str(&format!(
            "sysray_computed_cpu_trend_p95 {:.2}\n",
            snapshot.computed.cpu_trend_p95
        ));
        out.push_str("# HELP sysray_computed_memory_pressure Memory pressure score\n");
        out.push_str("# TYPE sysray_computed_memory_pressure gauge\n");
        out.push_str(&format!(
            "sysray_computed_memory_pressure {:.4}\n",
            snapshot.computed.memory_pressure
        ));
        out.push_str("# HELP sysray_alerts_total Active alert count\n");
        out.push_str("# TYPE sysray_alerts_total gauge\n");
        out.push_str(&format!(
            "sysray_alerts_total {}\n",
            snapshot.computed.alerts.len()
        ));
        out.push_str("# HELP sysray_alerts_info Active info alert count\n");
        out.push_str("# TYPE sysray_alerts_info gauge\n");
        out.push_str(&format!(
            "sysray_alerts_info {}\n",
            snapshot.computed.alerts_info
        ));
        out.push_str("# HELP sysray_alerts_warning Active warning alert count\n");
        out.push_str("# TYPE sysray_alerts_warning gauge\n");
        out.push_str(&format!(
            "sysray_alerts_warning {}\n",
            snapshot.computed.alerts_warning
        ));
        out.push_str("# HELP sysray_alerts_critical Active critical alert count\n");
        out.push_str("# TYPE sysray_alerts_critical gauge\n");
        out.push_str(&format!(
            "sysray_alerts_critical {}\n",
            snapshot.computed.alerts_critical
        ));

        Ok(out)
    }
}

fn escape_label(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn append_pressure_metric(
    out: &mut String,
    resource: &str,
    metric: &Option<&crate::collectors::linux::PressureMetric>,
) {
    let Some(metric) = metric else {
        return;
    };

    if let Some(some) = &metric.some {
        append_pressure_window(out, resource, "some", some);
    }
    if let Some(full) = &metric.full {
        append_pressure_window(out, resource, "full", full);
    }
}

fn append_pressure_window(
    out: &mut String,
    resource: &str,
    scope: &str,
    window: &crate::collectors::linux::PressureWindow,
) {
    out.push_str("# HELP sysray_linux_psi_avg10 Pressure stall avg10 percentage\n");
    out.push_str("# TYPE sysray_linux_psi_avg10 gauge\n");
    out.push_str(&format!(
        "sysray_linux_psi_avg10{{resource=\"{}\",scope=\"{}\"}} {:.2}\n",
        resource, scope, window.avg10
    ));
    out.push_str("# HELP sysray_linux_psi_avg60 Pressure stall avg60 percentage\n");
    out.push_str("# TYPE sysray_linux_psi_avg60 gauge\n");
    out.push_str(&format!(
        "sysray_linux_psi_avg60{{resource=\"{}\",scope=\"{}\"}} {:.2}\n",
        resource, scope, window.avg60
    ));
    out.push_str("# HELP sysray_linux_psi_avg300 Pressure stall avg300 percentage\n");
    out.push_str("# TYPE sysray_linux_psi_avg300 gauge\n");
    out.push_str(&format!(
        "sysray_linux_psi_avg300{{resource=\"{}\",scope=\"{}\"}} {:.2}\n",
        resource, scope, window.avg300
    ));
    out.push_str("# HELP sysray_linux_psi_total Pressure stall total time in microseconds\n");
    out.push_str("# TYPE sysray_linux_psi_total gauge\n");
    out.push_str(&format!(
        "sysray_linux_psi_total{{resource=\"{}\",scope=\"{}\"}} {}\n",
        resource, scope, window.total
    ));
}
