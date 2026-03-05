use crate::platform::api::{
    RawCpuReading, RawDiskInventory, RawDiskSpace, RawDiskStat, RawLinuxMetrics,
    RawLoadAverageSource, RawMemoryInfo, RawNetConnections, RawNetStat, RawProcReading,
    RawSystemInfo,
};
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::mem::{size_of, MaybeUninit};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

unsafe extern "C" {
    fn proc_pid_rusage(pid: libc::c_int, flavor: libc::c_int, buffer: *mut c_void) -> libc::c_int;
}

pub fn read_cpu() -> Result<RawCpuReading> {
    let top = command_output("top", &["-l", "1", "-n", "0"]).unwrap_or_default();
    let usage = top
        .lines()
        .find_map(parse_top_cpu_usage_line)
        .unwrap_or((0.0, 0.0, 0.0));
    let load = command_output("sysctl", &["-n", "vm.loadavg"])
        .map(|value| parse_loadavg(&value))
        .unwrap_or((0.0, 0.0, 0.0));

    Ok(RawCpuReading {
        direct_global_usage_pct: Some((usage.0 + usage.1).clamp(0.0, 100.0)),
        direct_iowait_pct: None,
        direct_steal_pct: Some(0.0),
        direct_per_core_usage_pct: Vec::new(),
        load_avg_1: load.0,
        load_avg_5: load.1,
        load_avg_15: load.2,
        load_avg_source: RawLoadAverageSource::Native,
        ..RawCpuReading::default()
    })
}

pub fn read_mount_map() -> HashMap<String, String> {
    disk_rows()
        .into_iter()
        .map(|row| (row.device, row.mount_point))
        .collect()
}

pub fn read_disk_space(mount: &str) -> RawDiskSpace {
    let output = match command_output("df", &["-kP", mount]) {
        Some(output) => output,
        None => return RawDiskSpace::default(),
    };

    parse_df_space(&output).unwrap_or_default()
}

pub fn read_disks() -> Result<Vec<RawDiskStat>> {
    let mut seen = HashSet::new();
    let disks = disk_rows()
        .into_iter()
        .filter(|row| seen.insert(row.device.clone()))
        .map(|row| RawDiskStat {
            device: row.device,
            ..RawDiskStat::default()
        })
        .collect();
    Ok(disks)
}

pub fn read_disk_inventory() -> Result<Vec<RawDiskInventory>> {
    let identifiers = disk_identifiers();
    let mut inventory = Vec::new();

    for identifier in identifiers {
        let info = command_output("diskutil", &["info", &identifier]).unwrap_or_default();
        let parent =
            parse_diskutil_field(&info, "Part of Whole").or_else(|| parent_identifier(&identifier));
        let mount_point = parse_diskutil_field(&info, "Mount Point").unwrap_or_default();
        let filesystem = parse_diskutil_field(&info, "File System Personality")
            .or_else(|| parse_diskutil_field(&info, "Type (Bundle)"))
            .unwrap_or_default();
        let label = parse_diskutil_field(&info, "Volume Name").unwrap_or_default();
        let uuid = parse_diskutil_field(&info, "Volume UUID")
            .or_else(|| parse_diskutil_field(&info, "Disk / Partition UUID"))
            .unwrap_or_default();
        let part_uuid = parse_diskutil_field(&info, "Disk / Partition UUID").unwrap_or_default();
        let model = parse_diskutil_field(&info, "Device / Media Name").unwrap_or_default();
        let serial = parse_diskutil_field(&info, "Disk / Partition UUID").unwrap_or_default();
        let transport = parse_diskutil_field(&info, "Protocol").unwrap_or_default();
        let reference =
            parse_diskutil_field(&info, "Device Identifier").unwrap_or_else(|| identifier.clone());
        let structure = if identifier.contains('s') {
            "partition".to_string()
        } else if info.contains("APFS Container") {
            "apfs-container".to_string()
        } else {
            "whole-disk".to_string()
        };
        let volume_kind = if filesystem.is_empty() {
            "block-device".to_string()
        } else if structure == "apfs-container" {
            "apfs-container".to_string()
        } else {
            "filesystem-volume".to_string()
        };
        let filesystem_family = if filesystem.eq_ignore_ascii_case("apfs") {
            "apple".to_string()
        } else {
            filesystem.to_ascii_lowercase()
        };

        inventory.push(RawDiskInventory {
            device: identifier,
            parent,
            structure,
            volume_kind,
            filesystem,
            filesystem_family,
            label,
            uuid,
            part_uuid,
            model,
            serial,
            transport,
            reference,
            scheduler: String::new(),
            rotational: None,
            removable: None,
            read_only: None,
            mount_points: if mount_point.is_empty() {
                Vec::new()
            } else {
                vec![mount_point]
            },
            logical_stack: Vec::new(),
            slaves: Vec::new(),
            holders: Vec::new(),
            children: Vec::new(),
        });
    }

    inventory.extend(read_remote_mount_inventory());

    let children_by_parent = build_children_map(&inventory);
    let parent_map: HashMap<String, Option<String>> = inventory
        .iter()
        .map(|item| (item.device.clone(), item.parent.clone()))
        .collect();
    for item in &mut inventory {
        item.children = children_by_parent
            .get(&item.device)
            .cloned()
            .unwrap_or_default();
        item.logical_stack = logical_stack_from_map(&item.device, &parent_map);
    }

    Ok(inventory)
}

pub fn read_net_connections() -> RawNetConnections {
    let output = command_output("netstat", &["-anp", "tcp"]).unwrap_or_default();
    let mut total = 0u32;
    let mut established = 0u32;

    for line in output.lines() {
        let trimmed = line.trim();
        if !(trimmed.starts_with("tcp")
            || trimmed.starts_with("tcp4")
            || trimmed.starts_with("tcp6"))
        {
            continue;
        }
        total += 1;
        if trimmed.contains("ESTABLISHED") {
            established += 1;
        }
    }

    RawNetConnections {
        total,
        established,
        tcp_state_breakdown_supported: false,
        udp_breakdown_supported: false,
        retrans_supported: false,
        ..RawNetConnections::default()
    }
}

pub fn read_network() -> Result<Vec<RawNetStat>> {
    let output = command_output("netstat", &["-ibn"]).unwrap_or_default();
    let mut interfaces: HashMap<String, RawNetStat> = HashMap::new();

    for line in output.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 12 {
            continue;
        }
        let interface = parts[0].to_string();
        if interface == "lo0" {
            continue;
        }

        let rx_packets = parts[4].parse::<u64>().ok();
        let rx_errors = parts[5].parse::<u64>().ok();
        let tx_packets = parts[6].parse::<u64>().ok();
        let tx_errors = parts[7].parse::<u64>().ok();
        let rx_bytes = parts[parts.len() - 2].parse::<u64>().ok();
        let tx_bytes = parts[parts.len() - 1].parse::<u64>().ok();

        let Some((rx_packets, rx_errors, tx_packets, tx_errors, rx_bytes, tx_bytes)) = rx_packets
            .zip(rx_errors)
            .zip(tx_packets)
            .zip(tx_errors)
            .zip(rx_bytes)
            .zip(tx_bytes)
            .map(
                |(((((rx_packets, rx_errors), tx_packets), tx_errors), rx_bytes), tx_bytes)| {
                    (
                        rx_packets, rx_errors, tx_packets, tx_errors, rx_bytes, tx_bytes,
                    )
                },
            )
        else {
            continue;
        };

        let entry = interfaces
            .entry(interface.clone())
            .or_insert_with(|| RawNetStat {
                interface,
                ..RawNetStat::default()
            });
        entry.rx_packets = entry.rx_packets.saturating_add(rx_packets);
        entry.rx_errors = entry.rx_errors.saturating_add(rx_errors);
        entry.tx_packets = entry.tx_packets.saturating_add(tx_packets);
        entry.tx_errors = entry.tx_errors.saturating_add(tx_errors);
        entry.rx_bytes = entry.rx_bytes.saturating_add(rx_bytes);
        entry.tx_bytes = entry.tx_bytes.saturating_add(tx_bytes);
    }

    Ok(interfaces.into_values().collect())
}

pub fn read_memory() -> Result<RawMemoryInfo> {
    let total_bytes = command_output("sysctl", &["-n", "hw.memsize"])
        .and_then(|value| value.trim().parse::<u64>().ok())
        .unwrap_or(0);
    let page_size = command_output("sysctl", &["-n", "hw.pagesize"])
        .and_then(|value| value.trim().parse::<u64>().ok())
        .unwrap_or(4096);

    let mut free_pages = 0u64;
    let mut inactive_pages = 0u64;
    let mut speculative_pages = 0u64;
    let mut wired_pages = 0u64;
    let mut active_pages = 0u64;
    let mut compressed_pages = 0u64;

    if let Some(vm_stat) = command_output("vm_stat", &[]) {
        for line in vm_stat.lines() {
            if let Some(value) = parse_vm_stat_value(line, "Pages free") {
                free_pages = value;
            } else if let Some(value) = parse_vm_stat_value(line, "Pages inactive") {
                inactive_pages = value;
            } else if let Some(value) = parse_vm_stat_value(line, "Pages speculative") {
                speculative_pages = value;
            } else if let Some(value) = parse_vm_stat_value(line, "Pages wired down") {
                wired_pages = value;
            } else if let Some(value) = parse_vm_stat_value(line, "Pages occupied by compressor") {
                compressed_pages = value;
            } else if let Some(value) = parse_vm_stat_value(line, "Pages active") {
                active_pages = value;
            }
        }
    }

    let available_bytes = (free_pages + inactive_pages + speculative_pages) * page_size;
    let used_bytes = if total_bytes > 0 {
        total_bytes.saturating_sub(available_bytes)
    } else {
        (active_pages + wired_pages + compressed_pages) * page_size
    };

    let (swap_total_kb, swap_used_kb) = command_output("sysctl", &["vm.swapusage"])
        .map(|value| parse_swapusage(&value))
        .unwrap_or((0, 0));

    let total_kb = total_bytes / 1024;
    let available_kb = available_bytes / 1024;
    let used_kb = used_bytes / 1024;
    let free_kb = free_pages * page_size / 1024;
    let usage_pct = if total_kb > 0 {
        (used_kb as f64 / total_kb as f64 * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };

    Ok(RawMemoryInfo {
        total_kb,
        used_kb,
        free_kb,
        available_kb,
        cached_kb: inactive_pages * page_size / 1024,
        buffers_kb: 0,
        swap_total_kb,
        swap_used_kb,
        dirty_kb: 0,
        vm_pgfault: 0,
        vm_pgmajfault: 0,
        vm_pgpgin: 0,
        vm_pgpgout: 0,
        vm_pswpin: 0,
        vm_pswpout: 0,
        vm_pgscan: 0,
        vm_pgsteal: 0,
        usage_pct,
        cached_supported: true,
        buffers_supported: false,
        dirty_supported: false,
        vm_fault_counters_supported: false,
        vm_scan_counters_supported: false,
        vm_io_counters_supported: false,
    })
}

pub fn read_system() -> Result<RawSystemInfo> {
    let os_name = command_output("sw_vers", &["-productName"]).unwrap_or_else(|| "macOS".into());
    let product_version = command_output("sw_vers", &["-productVersion"]).unwrap_or_default();
    let build_version = command_output("sw_vers", &["-buildVersion"]).unwrap_or_default();

    Ok(RawSystemInfo {
        hostname: command_output("scutil", &["--get", "ComputerName"])
            .or_else(|| command_output("hostname", &[]))
            .unwrap_or_default(),
        os_name,
        os_version: format_os_version(&product_version, &build_version),
        kernel_version: command_output("uname", &["-r"]).unwrap_or_default(),
        uptime_seconds: read_uptime_seconds(),
        architecture: std::env::consts::ARCH.to_string(),
        cpu_count: std::thread::available_parallelism()
            .map(|n| n.get() as u32)
            .unwrap_or(1),
    })
}

pub fn page_size() -> u64 {
    1024
}

pub fn clock_ticks() -> f64 {
    1_000_000_000.0
}

pub fn num_cpus() -> f64 {
    std::thread::available_parallelism()
        .map(|n| n.get() as f64)
        .unwrap_or(1.0)
}

pub fn read_processes() -> Result<Vec<RawProcReading>> {
    let output = command_output(
        "ps",
        &[
            "-axo",
            "pid=,comm=,%cpu=,time=,rss=,vsz=,thcount=,state=,user=,command=",
        ],
    )
    .unwrap_or_default();

    let mut processes = Vec::new();
    for line in output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 9 {
            continue;
        }

        let pid = match parts[0].parse::<u32>() {
            Ok(pid) => pid,
            Err(_) => continue,
        };
        let cpu_pct_hint = parts[2].parse::<f64>().ok();
        let rss_kb = parts[4].parse::<u64>().unwrap_or(0);
        let vsz_kb = parts[5].parse::<u64>().unwrap_or(0);
        let threads = parts[6].parse::<u32>().unwrap_or(0);
        let state_char = parts[7].chars().next().unwrap_or('?');
        let user = parts[8].to_string();
        let cmdline = if parts.len() > 9 {
            parts[9..].join(" ")
        } else {
            parts[1].to_string()
        };
        let fallback_cpu_time_ns = parse_ps_cpu_time_ns(parts[3]).unwrap_or(0);
        let rusage = read_process_rusage(pid);
        let fd_count = read_fd_count(pid);

        processes.push(RawProcReading {
            pid,
            name: parts[1].to_string(),
            cmdline,
            utime: rusage
                .as_ref()
                .map(|usage| usage.ri_user_time)
                .unwrap_or(fallback_cpu_time_ns),
            stime: rusage
                .as_ref()
                .map(|usage| usage.ri_system_time)
                .unwrap_or(0),
            vsize: vsz_kb * 1024,
            rss: rss_kb,
            threads,
            fd_count,
            state_char,
            user,
            io_read_bytes: rusage
                .as_ref()
                .map(|usage| usage.ri_diskio_bytesread)
                .unwrap_or(0),
            io_write_bytes: rusage
                .as_ref()
                .map(|usage| usage.ri_diskio_byteswritten)
                .unwrap_or(0),
            cpu_pct_hint,
            fd_count_supported: true,
            io_bytes_supported: rusage.is_some(),
        });
    }

    Ok(processes)
}

pub fn read_linux_metrics() -> Result<RawLinuxMetrics> {
    Ok(RawLinuxMetrics::default())
}

#[derive(Debug, Clone)]
struct DiskRow {
    device: String,
    mount_point: String,
}

fn disk_identifiers() -> Vec<String> {
    let output = match command_output("diskutil", &["list"]) {
        Some(output) => output,
        None => return Vec::new(),
    };

    output
        .lines()
        .filter_map(|line| line.split_whitespace().last())
        .filter(|token| token.starts_with("disk"))
        .map(ToString::to_string)
        .collect()
}

fn parse_diskutil_field(output: &str, key: &str) -> Option<String> {
    output.lines().find_map(|line| {
        let trimmed = line.trim();
        let (field, value) = trimmed.split_once(':')?;
        if field.trim() == key {
            let value = value.trim();
            if value.is_empty() || value == "Not applicable (no file system)" {
                None
            } else {
                Some(value.to_string())
            }
        } else {
            None
        }
    })
}

fn parent_identifier(identifier: &str) -> Option<String> {
    identifier.rsplit_once('s').and_then(|(base, suffix)| {
        if suffix.chars().all(|c| c.is_ascii_digit()) {
            Some(base.to_string())
        } else {
            None
        }
    })
}

fn build_children_map(items: &[RawDiskInventory]) -> HashMap<String, Vec<String>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for item in items {
        if let Some(parent) = &item.parent {
            map.entry(parent.clone())
                .or_default()
                .push(item.device.clone());
        }
    }
    map
}

fn read_remote_mount_inventory() -> Vec<RawDiskInventory> {
    let output = match command_output("mount", &[]) {
        Some(output) => output,
        None => return Vec::new(),
    };

    output.lines().filter_map(parse_remote_mount_line).collect()
}

fn parse_remote_mount_line(line: &str) -> Option<RawDiskInventory> {
    let (source, rest) = line.split_once(" on ")?;
    let (mount_point, meta) = rest.split_once(" (")?;
    let fstype = meta
        .trim_end_matches(')')
        .split(',')
        .next()
        .map(str::trim)
        .unwrap_or_default();

    if !is_remote_mount(source, fstype) {
        return None;
    }

    Some(RawDiskInventory {
        device: source.to_string(),
        parent: None,
        structure: "remote-mount".to_string(),
        volume_kind: remote_volume_kind(fstype),
        filesystem: fstype.to_string(),
        filesystem_family: "remote".to_string(),
        label: mount_point.to_string(),
        uuid: String::new(),
        part_uuid: String::new(),
        model: String::new(),
        serial: String::new(),
        transport: remote_transport(fstype).to_string(),
        reference: source.to_string(),
        scheduler: String::new(),
        rotational: None,
        removable: None,
        read_only: None,
        mount_points: vec![mount_point.to_string()],
        logical_stack: vec![source.to_string()],
        slaves: Vec::new(),
        holders: Vec::new(),
        children: Vec::new(),
    })
}

fn is_remote_mount(source: &str, fstype: &str) -> bool {
    matches!(
        fstype.to_ascii_lowercase().as_str(),
        "nfs" | "smbfs" | "webdav" | "webdavs" | "afpfs"
    ) || source.starts_with("//")
        || source.contains(':')
}

fn remote_transport(fstype: &str) -> &'static str {
    match fstype.to_ascii_lowercase().as_str() {
        "nfs" => "nfs",
        "smbfs" => "smb",
        "afpfs" => "afp",
        "webdav" | "webdavs" => "webdav",
        _ => "remote",
    }
}

fn remote_volume_kind(fstype: &str) -> String {
    match fstype.to_ascii_lowercase().as_str() {
        "nfs" => "nfs-share".to_string(),
        "smbfs" => "smb-share".to_string(),
        "afpfs" => "afp-share".to_string(),
        "webdav" | "webdavs" => "webdav-share".to_string(),
        other => format!("{other}-mount"),
    }
}

fn logical_stack_from_map(
    device: &str,
    parent_map: &HashMap<String, Option<String>>,
) -> Vec<String> {
    let mut stack = Vec::new();
    let mut current = Some(device.to_string());
    while let Some(name) = current {
        stack.push(name.clone());
        current = parent_map.get(&name).cloned().flatten();
    }
    stack.reverse();
    stack
}

fn command_output(program: &str, args: &[&str]) -> Option<String> {
    for candidate in command_candidates(program) {
        let output = match Command::new(&candidate)
            .args(args)
            .env("LC_ALL", "C")
            .env("LANG", "C")
            .output()
        {
            Ok(output) => output,
            Err(_) => continue,
        };
        if !output.status.success() {
            continue;
        }
        let text = String::from_utf8(output.stdout).ok()?;
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    None
}

fn command_candidates(program: &str) -> Vec<PathBuf> {
    let path = Path::new(program);
    if path.is_absolute() || program.contains('/') {
        return vec![path.to_path_buf()];
    }

    vec![
        PathBuf::from(program),
        PathBuf::from("/usr/bin").join(program),
        PathBuf::from("/usr/sbin").join(program),
        PathBuf::from("/bin").join(program),
        PathBuf::from("/sbin").join(program),
    ]
}

fn parse_top_cpu_usage_line(line: &str) -> Option<(f64, f64, f64)> {
    let trimmed = line.trim();
    if !trimmed.starts_with("CPU usage:") {
        return None;
    }

    let mut user = 0.0;
    let mut system = 0.0;
    let mut idle = 0.0;
    for segment in trimmed.trim_start_matches("CPU usage:").split(',') {
        let segment = segment.trim();
        if let Some(value) = segment.strip_suffix("% user") {
            user = value.trim().parse().ok()?;
        } else if let Some(value) = segment.strip_suffix("% sys") {
            system = value.trim().parse().ok()?;
        } else if let Some(value) = segment.strip_suffix("% idle") {
            idle = value.trim().parse().ok()?;
        }
    }
    Some((user, system, idle))
}

fn parse_loadavg(value: &str) -> (f64, f64, f64) {
    let cleaned = value.trim().trim_start_matches('{').trim_end_matches('}');
    let nums: Vec<f64> = cleaned
        .split_whitespace()
        .filter_map(|part| part.parse::<f64>().ok())
        .collect();
    (
        nums.first().copied().unwrap_or(0.0),
        nums.get(1).copied().unwrap_or(0.0),
        nums.get(2).copied().unwrap_or(0.0),
    )
}

fn disk_rows() -> Vec<DiskRow> {
    let output = match command_output("df", &["-kP"]) {
        Some(output) => output,
        None => return Vec::new(),
    };

    output
        .lines()
        .skip(1)
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 6 {
                return None;
            }
            let device = parts[0];
            if !device.starts_with("/dev/") {
                return None;
            }
            Some(DiskRow {
                device: device.trim_start_matches("/dev/").to_string(),
                mount_point: parts[5].to_string(),
            })
        })
        .collect()
}

fn parse_df_space(output: &str) -> Option<RawDiskSpace> {
    let line = output.lines().nth(1)?;
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 6 {
        return None;
    }

    let total_kb = parts[1].parse::<f64>().ok()?;
    let used_kb = parts[2].parse::<f64>().ok()?;
    let free_kb = parts[3].parse::<f64>().ok()?;
    let usage_pct = parts[4].trim_end_matches('%').parse::<f64>().ok()?;

    Some(RawDiskSpace {
        total_gb: total_kb / 1_048_576.0,
        used_gb: used_kb / 1_048_576.0,
        free_gb: free_kb / 1_048_576.0,
        usage_pct,
    })
}

fn parse_vm_stat_value(line: &str, key: &str) -> Option<u64> {
    let trimmed = line.trim();
    if !trimmed.starts_with(key) {
        return None;
    }
    trimmed
        .split(':')
        .nth(1)?
        .trim()
        .trim_end_matches('.')
        .replace('.', "")
        .parse()
        .ok()
}

fn parse_swapusage(value: &str) -> (u64, u64) {
    let mut total_kb = 0u64;
    let mut used_kb = 0u64;

    for segment in value.split_whitespace().collect::<Vec<_>>().windows(3) {
        if segment[0] == "total" && segment[1] == "=" {
            total_kb = parse_size_to_kb(segment[2]).unwrap_or(0);
        } else if segment[0] == "used" && segment[1] == "=" {
            used_kb = parse_size_to_kb(segment[2]).unwrap_or(0);
        }
    }

    (total_kb, used_kb)
}

fn parse_size_to_kb(value: &str) -> Option<u64> {
    let trimmed = value.trim_end_matches(',');
    let unit = trimmed.chars().last()?;
    let number = trimmed[..trimmed.len().saturating_sub(1)]
        .parse::<f64>()
        .ok()?;
    let multiplier = match unit {
        'K' => 1.0,
        'M' => 1024.0,
        'G' => 1024.0 * 1024.0,
        'T' => 1024.0 * 1024.0 * 1024.0,
        _ => return None,
    };
    Some((number * multiplier).round() as u64)
}

fn format_os_version(product_version: &str, build_version: &str) -> String {
    match (product_version.trim(), build_version.trim()) {
        ("", "") => String::new(),
        (version, "") => version.to_string(),
        ("", build) => build.to_string(),
        (version, build) => format!("{} ({})", version, build),
    }
}

fn parse_ps_cpu_time_ns(value: &str) -> Option<u64> {
    let mut days = 0u64;
    let mut time_part = value.trim();

    if let Some((day_part, rest)) = time_part.split_once('-') {
        days = day_part.parse().ok()?;
        time_part = rest;
    }

    let segments: Vec<u64> = time_part
        .split(':')
        .map(|part| part.parse::<u64>().ok())
        .collect::<Option<Vec<_>>>()?;

    let seconds = match segments.as_slice() {
        [minutes, seconds] => minutes.saturating_mul(60).saturating_add(*seconds),
        [hours, minutes, seconds] => hours
            .saturating_mul(3600)
            .saturating_add(minutes.saturating_mul(60))
            .saturating_add(*seconds),
        _ => return None,
    };

    days.saturating_mul(86_400)
        .saturating_add(seconds)
        .checked_mul(1_000_000_000)
}

fn read_process_rusage(pid: u32) -> Option<libc::rusage_info_v2> {
    let mut usage = MaybeUninit::<libc::rusage_info_v2>::zeroed();
    let result = unsafe {
        proc_pid_rusage(
            pid as libc::c_int,
            libc::RUSAGE_INFO_V2,
            usage.as_mut_ptr().cast(),
        )
    };
    if result == 0 {
        Some(unsafe { usage.assume_init() })
    } else {
        None
    }
}

fn read_fd_count(pid: u32) -> u32 {
    let mut capacity = 64usize;

    loop {
        let mut buffer = Vec::with_capacity(capacity);
        buffer.resize_with(capacity, || libc::proc_fdinfo {
            proc_fd: 0,
            proc_fdtype: 0,
        });
        let buffer_size = (buffer.len() * size_of::<libc::proc_fdinfo>()) as libc::c_int;
        let bytes = unsafe {
            libc::proc_pidinfo(
                pid as libc::c_int,
                libc::PROC_PIDLISTFDS,
                0,
                buffer.as_mut_ptr().cast(),
                buffer_size,
            )
        };

        if bytes <= 0 {
            return 0;
        }

        if bytes < buffer_size || capacity >= 16_384 {
            return (bytes as usize / size_of::<libc::proc_fdinfo>()) as u32;
        }

        capacity *= 2;
    }
}

fn read_uptime_seconds() -> u64 {
    let Some(boottime) = command_output("sysctl", &["-n", "kern.boottime"]) else {
        return 0;
    };
    let Some(sec_pos) = boottime.find("sec = ") else {
        return 0;
    };
    let rest = &boottime[sec_pos + 6..];
    let secs = rest
        .split(',')
        .next()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .unwrap_or(0);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    now.saturating_sub(secs)
}

#[cfg(test)]
mod tests {
    use super::parse_ps_cpu_time_ns;

    #[test]
    fn parse_ps_cpu_time_without_days() {
        assert_eq!(parse_ps_cpu_time_ns("01:02"), Some(62_000_000_000));
        assert_eq!(parse_ps_cpu_time_ns("01:02:03"), Some(3_723_000_000_000));
    }

    #[test]
    fn parse_ps_cpu_time_with_days() {
        assert_eq!(
            parse_ps_cpu_time_ns("2-00:00:01"),
            Some(172_801_000_000_000)
        );
    }
}
