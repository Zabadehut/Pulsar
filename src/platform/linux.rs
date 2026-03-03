use crate::platform::api::{
    RawCgroupMetrics, RawCpuReading, RawCpuStat, RawDiskInventory, RawDiskSpace, RawDiskStat,
    RawLinuxMetrics, RawMemoryInfo, RawNetConnections, RawNetStat, RawPressureMetric,
    RawPressureWindow, RawProcReading, RawPsiMetrics, RawSystemInfo,
};
use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

// ─── CPU ─────────────────────────────────────────────────────────────────────

fn parse_cpu_line(line: &str) -> Option<RawCpuStat> {
    let mut parts = line.split_whitespace();
    parts.next()?; // skip label
    let user = parts.next()?.parse().ok()?;
    let nice = parts.next()?.parse().ok()?;
    let system = parts.next()?.parse().ok()?;
    let idle = parts.next()?.parse().ok()?;
    let iowait = parts.next()?.parse().unwrap_or(0);
    let irq = parts.next()?.parse().unwrap_or(0);
    let softirq = parts.next()?.parse().unwrap_or(0);
    let steal = parts.next()?.parse().unwrap_or(0);
    Some(RawCpuStat {
        user,
        nice,
        system,
        idle,
        iowait,
        irq,
        softirq,
        steal,
    })
}

pub fn read_cpu() -> Result<RawCpuReading> {
    let content = fs::read_to_string("/proc/stat")?;
    let mut lines = content.lines();

    let global_line = lines
        .next()
        .ok_or_else(|| anyhow::anyhow!("/proc/stat empty"))?;
    let global = parse_cpu_line(global_line)
        .ok_or_else(|| anyhow::anyhow!("Failed to parse global CPU line"))?;

    let mut cores: Vec<RawCpuStat> = Vec::new();
    for line in &mut lines {
        if !line.starts_with("cpu") || line.starts_with("cpu ") {
            break;
        }
        if let Some(stat) = parse_cpu_line(line) {
            cores.push(stat);
        }
    }

    let loadavg = fs::read_to_string("/proc/loadavg")?;
    let mut lp = loadavg.split_whitespace();
    let load_avg_1 = lp.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
    let load_avg_5 = lp.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
    let load_avg_15 = lp.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);

    let mut context_switches = 0u64;
    let mut interrupts = 0u64;
    for line in content.lines() {
        if let Some(val) = line.strip_prefix("ctxt ") {
            context_switches = val.trim().parse().unwrap_or(0);
        } else if let Some(rest) = line.strip_prefix("intr ") {
            interrupts = rest
                .split_whitespace()
                .next()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);
        }
    }

    Ok(RawCpuReading {
        global,
        cores,
        load_avg_1,
        load_avg_5,
        load_avg_15,
        context_switches,
        interrupts,
        ..RawCpuReading::default()
    })
}

// ─── Disk ─────────────────────────────────────────────────────────────────────

fn strip_partition_suffix(dev: &str) -> &str {
    if let Some(pos) = dev.rfind('p') {
        let suffix = &dev[pos + 1..];
        if !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit()) {
            return &dev[..pos];
        }
    }
    let trimmed = dev.trim_end_matches(|c: char| c.is_ascii_digit());
    if trimmed != dev && !trimmed.is_empty() {
        return trimmed;
    }
    dev
}

pub fn read_mount_map() -> HashMap<String, String> {
    let content = match fs::read_to_string("/proc/mounts") {
        Ok(c) => c,
        Err(_) => return HashMap::new(),
    };
    let mut map: HashMap<String, String> = HashMap::new();
    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }
        if let Some(dev) = parts[0].strip_prefix("/dev/") {
            let mount = parts[1];
            map.entry(dev.to_string())
                .or_insert_with(|| mount.to_string());
            let parent = strip_partition_suffix(dev);
            if parent != dev {
                map.entry(parent.to_string())
                    .or_insert_with(|| mount.to_string());
            }
        }
    }
    map
}

pub fn read_disk_space(mount: &str) -> RawDiskSpace {
    use std::ffi::CString;
    use std::mem::MaybeUninit;
    let path = match CString::new(mount) {
        Ok(p) => p,
        Err(_) => return RawDiskSpace::default(),
    };
    let mut stat: MaybeUninit<libc::statvfs> = MaybeUninit::uninit();
    if unsafe { libc::statvfs(path.as_ptr(), stat.as_mut_ptr()) } != 0 {
        return RawDiskSpace::default();
    }
    let stat = unsafe { stat.assume_init() };
    let block = stat.f_frsize as f64;
    let total_gb = stat.f_blocks as f64 * block / 1e9;
    let free_gb = stat.f_bfree as f64 * block / 1e9;
    let used_gb = total_gb - free_gb;
    let usage_pct = if total_gb > 0.0 {
        (used_gb / total_gb * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };
    RawDiskSpace {
        total_gb,
        used_gb,
        free_gb,
        usage_pct,
    }
}

pub fn read_disks() -> Result<Vec<RawDiskStat>> {
    let diskstats = fs::read_to_string("/proc/diskstats")?;
    let mut results = Vec::new();

    for line in diskstats.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 14 {
            continue;
        }
        let device = parts[2].to_string();
        if device.starts_with("loop") || device.starts_with("ram") || device.starts_with("zram") {
            continue;
        }
        if device != strip_partition_suffix(&device) {
            continue;
        }

        results.push(RawDiskStat {
            device,
            reads_completed: parts[3].parse().unwrap_or(0),
            reads_merged: parts[4].parse().unwrap_or(0),
            reads_sectors: parts[5].parse().unwrap_or(0),
            reads_time_ms: parts[6].parse().unwrap_or(0),
            writes_completed: parts[7].parse().unwrap_or(0),
            writes_merged: parts[8].parse().unwrap_or(0),
            writes_sectors: parts[9].parse().unwrap_or(0),
            writes_time_ms: parts[10].parse().unwrap_or(0),
            io_ticks: parts[12].parse().unwrap_or(0),
            weighted_io_time_ms: parts[13].parse().unwrap_or(0),
        });
    }
    Ok(results)
}

pub fn read_disk_inventory() -> Result<Vec<RawDiskInventory>> {
    let Some(output) = command_output(
        "lsblk",
        &[
            "-J",
            "-o",
            "NAME,KNAME,PKNAME,TYPE,FSTYPE,LABEL,UUID,PARTUUID,MOUNTPOINTS,MODEL,SERIAL,TRAN,WWN",
        ],
    ) else {
        return Ok(Vec::new());
    };

    let value: Value = serde_json::from_str(&output)?;
    let mut inventory = Vec::new();
    for device in value
        .get("blockdevices")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        flatten_lsblk_node(device, &mut inventory);
    }
    inventory.extend(read_remote_mount_inventory());
    enrich_linux_disk_inventory(&mut inventory);
    Ok(inventory)
}

// ─── Network ──────────────────────────────────────────────────────────────────

pub fn read_net_connections() -> RawNetConnections {
    let mut connections = RawNetConnections::default();

    for file in &["/proc/net/tcp", "/proc/net/tcp6"] {
        if let Ok(content) = fs::read_to_string(file) {
            parse_socket_table(&content, false, &mut connections);
        }
    }

    for file in &["/proc/net/udp", "/proc/net/udp6"] {
        if let Ok(content) = fs::read_to_string(file) {
            parse_socket_table(&content, true, &mut connections);
        }
    }

    connections.total = connections.total.max(
        connections.established
            + connections.tcp_syn_sent
            + connections.tcp_syn_recv
            + connections.tcp_fin_wait1
            + connections.tcp_fin_wait2
            + connections.tcp_time_wait
            + connections.tcp_close
            + connections.tcp_close_wait
            + connections.tcp_last_ack
            + connections.tcp_listen
            + connections.tcp_closing
            + connections.tcp_other,
    );
    connections.retrans_segs =
        read_snmp_counter("/proc/net/snmp", "Tcp", "RetransSegs").unwrap_or_default();
    connections
}

pub fn read_network() -> Result<Vec<RawNetStat>> {
    let content = fs::read_to_string("/proc/net/dev")?;
    let mut results = Vec::new();

    for line in content.lines().skip(2) {
        let line = line.trim();
        let colon = match line.find(':') {
            Some(p) => p,
            None => continue,
        };
        let interface = line[..colon].trim().to_string();
        if interface == "lo" {
            continue;
        }
        let nums: Vec<u64> = line[colon + 1..]
            .split_whitespace()
            .filter_map(|s| s.parse().ok())
            .collect();
        if nums.len() < 16 {
            continue;
        }
        results.push(RawNetStat {
            interface,
            rx_bytes: nums[0],
            rx_packets: nums[1],
            rx_errors: nums[2],
            rx_dropped: nums[3],
            tx_bytes: nums[8],
            tx_packets: nums[9],
            tx_errors: nums[10],
            tx_dropped: nums[11],
        });
    }
    Ok(results)
}

fn command_output(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn flatten_lsblk_node(node: &Value, out: &mut Vec<RawDiskInventory>) {
    let device = json_string(node, "kname")
        .or_else(|| json_string(node, "name"))
        .unwrap_or_default();
    if device.is_empty() {
        return;
    }

    let children = node
        .get("children")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| json_string(item, "kname").or_else(|| json_string(item, "name")))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    out.push(RawDiskInventory {
        device,
        parent: json_string(node, "pkname"),
        structure: json_string(node, "type").unwrap_or_default(),
        volume_kind: String::new(),
        filesystem: json_string(node, "fstype").unwrap_or_default(),
        filesystem_family: String::new(),
        label: json_string(node, "label").unwrap_or_default(),
        uuid: json_string(node, "uuid").unwrap_or_default(),
        part_uuid: json_string(node, "partuuid").unwrap_or_default(),
        model: json_string(node, "model").unwrap_or_default(),
        serial: json_string(node, "serial").unwrap_or_default(),
        transport: json_string(node, "tran").unwrap_or_default(),
        reference: json_string(node, "wwn").unwrap_or_default(),
        scheduler: String::new(),
        rotational: None,
        removable: None,
        read_only: None,
        mount_points: json_string_list(node, "mountpoints"),
        logical_stack: Vec::new(),
        slaves: Vec::new(),
        holders: Vec::new(),
        children,
    });

    for child in node
        .get("children")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        flatten_lsblk_node(child, out);
    }
}

fn enrich_linux_disk_inventory(items: &mut [RawDiskInventory]) {
    let lvm_vgs = read_lvm_pv_map();
    let lvs = read_lvm_lv_map();
    let dmsetup = read_dmsetup_map();
    let mdadm = read_mdadm_map();
    let btrfs = read_btrfs_map();
    let findmnt = read_findmnt_map();
    let zfs_mounts = read_zfs_mount_map();
    let zpool = read_zpool_device_map();
    let crypttab = read_crypttab_map();
    let parent_map: HashMap<String, Option<String>> = items
        .iter()
        .map(|item| (item.device.clone(), item.parent.clone()))
        .collect();

    for item in items.iter_mut() {
        let sys_path = sys_block_path(&item.device);
        let dm_name = read_string_file(&sys_path.join("dm/name"));
        let dm_uuid = read_string_file(&sys_path.join("dm/uuid"));
        let md_level = read_string_file(&sys_path.join("md/level"));
        let dmsetup_info = dmsetup.get(&item.device);
        let mdadm_info = mdadm.get(&item.device);
        let btrfs_info = btrfs.get(&item.device);
        let findmnt_info = findmnt.get(&item.device);
        let zpool_name = zpool.get(&item.device);
        let crypt_name = crypttab.get(&item.device);

        if item.structure == "remote-mount" {
            item.volume_kind = remote_volume_kind(&item.filesystem);
            item.filesystem_family = filesystem_family(&item.filesystem);
            item.logical_stack = vec![item.device.clone()];
            continue;
        }

        if item.structure.is_empty() {
            item.structure = infer_linux_structure(&item.device, &sys_path);
        }
        if let Some((mounts, fstype, source)) = findmnt_info {
            if item.mount_points.is_empty() {
                item.mount_points = mounts.clone();
            }
            if item.filesystem.is_empty() {
                item.filesystem = fstype.clone();
            }
            if item.reference.is_empty() {
                item.reference = source.clone();
            }
        }
        if item.transport.is_empty() {
            item.transport = read_string_file(&sys_path.join("device/transport"))
                .or_else(|| read_string_file(&sys_path.join("device/protocol")))
                .unwrap_or_else(|| infer_linux_transport(&item.device).to_string());
        }
        if item.reference.is_empty() {
            item.reference = read_string_file(&sys_path.join("device/wwid"))
                .or_else(|| dm_uuid.clone())
                .or_else(|| {
                    dmsetup_info
                        .map(|info| info.uuid.clone())
                        .filter(|value| !value.is_empty())
                })
                .or_else(|| {
                    mdadm_info
                        .map(|info| info.uuid.clone())
                        .filter(|value| !value.is_empty())
                })
                .or_else(|| read_string_file(&sys_path.join("device/serial")))
                .unwrap_or_default();
        }
        if item.label.is_empty() {
            item.label = dm_name
                .clone()
                .or_else(|| {
                    dmsetup_info
                        .map(|info| info.name.clone())
                        .filter(|value| !value.is_empty())
                })
                .or_else(|| {
                    mdadm_info
                        .map(|info| info.name.clone())
                        .filter(|value| !value.is_empty())
                })
                .or_else(|| {
                    btrfs_info
                        .map(|info| info.label.clone())
                        .filter(|value| !value.is_empty())
                })
                .unwrap_or_default();
        }
        if let Some(vg_name) = lvm_vgs.get(&item.device) {
            if item.volume_kind.is_empty() || item.volume_kind == "partition" {
                item.volume_kind = "lvm-member".to_string();
            }
            if item.label.is_empty() {
                item.label = vg_name.clone();
            }
        }
        if let Some((vg_name, lv_name, lv_path)) = dm_name
            .as_deref()
            .and_then(|name| lvs.get(name))
            .or_else(|| lvs.get(&item.device))
        {
            item.volume_kind = "lvm-lv".to_string();
            if item.label.is_empty() {
                item.label = format!("{vg_name}/{lv_name}");
            }
            if item.reference.is_empty() {
                item.reference = lv_path.clone();
            }
        }
        if let Some(info) = mdadm_info {
            if item.volume_kind.is_empty() || item.volume_kind == "raid-array" {
                item.volume_kind = format!("md-{}", info.level);
            }
            if item.label.is_empty() {
                item.label = info.name.clone();
            }
        }
        if let Some(info) = btrfs_info {
            if item.filesystem.is_empty() {
                item.filesystem = "btrfs".to_string();
            }
            item.filesystem_family = "btrfs".to_string();
            if item.label.is_empty() && !info.label.is_empty() {
                item.label = info.label.clone();
            }
            if item.reference.is_empty() && !info.uuid.is_empty() {
                item.reference = info.uuid.clone();
            }
            if item.volume_kind == "partition"
                || item.volume_kind == "filesystem-volume"
                || item.volume_kind.is_empty()
            {
                item.volume_kind = "btrfs-member".to_string();
            }
        }
        if let Some(name) = crypt_name {
            if item.volume_kind == "partition" || item.volume_kind == "block-device" {
                item.volume_kind = "luks-member".to_string();
            }
            if item.label.is_empty() {
                item.label = name.clone();
            }
        }
        if let Some(pool) = zpool_name {
            if item.volume_kind == "block-device" || item.volume_kind == "filesystem-volume" {
                item.volume_kind = "zfs-member".to_string();
            }
            if item.label.is_empty() {
                item.label = pool.clone();
            }
        }
        if let Some(dataset) = zfs_mounts
            .iter()
            .find(|(mount, _)| item.mount_points.iter().any(|point| point == mount))
            .map(|(_, dataset)| dataset)
        {
            item.filesystem = "zfs".to_string();
            item.filesystem_family = "zfs".to_string();
            item.volume_kind = "zfs-dataset".to_string();
            if item.reference.is_empty() {
                item.reference = dataset.clone();
            }
            if item.label.is_empty() {
                item.label = dataset.clone();
            }
        }
        item.scheduler = read_string_file(&sys_path.join("queue/scheduler"))
            .map(|value| value.replace(['[', ']'], ""))
            .unwrap_or_default();
        item.rotational = read_bool_file(&sys_path.join("queue/rotational"));
        item.removable = read_bool_file(&sys_path.join("removable"));
        item.read_only = read_bool_file(&sys_path.join("ro"));
        item.slaves = read_dir_names(&sys_path.join("slaves"));
        item.holders = read_dir_names(&sys_path.join("holders"));
        item.volume_kind = infer_linux_volume_kind(
            item,
            dm_name.as_deref(),
            dm_uuid.as_deref(),
            md_level.as_deref(),
        );
        item.filesystem_family = filesystem_family(&item.filesystem);
        item.logical_stack = logical_stack_for(&item.device, &parent_map);
    }
}

fn read_remote_mount_inventory() -> Vec<RawDiskInventory> {
    let content = match fs::read_to_string("/proc/mounts") {
        Ok(content) => content,
        Err(_) => return Vec::new(),
    };

    let mut items = Vec::new();
    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }
        let source = parts[0];
        let mount_point = parts[1];
        let filesystem = parts[2];
        if !is_remote_mount(source, filesystem) {
            continue;
        }

        items.push(RawDiskInventory {
            device: source.to_string(),
            parent: None,
            structure: "remote-mount".to_string(),
            volume_kind: remote_volume_kind(filesystem),
            filesystem: filesystem.to_string(),
            filesystem_family: filesystem_family(filesystem),
            label: mount_point.to_string(),
            uuid: String::new(),
            part_uuid: String::new(),
            model: String::new(),
            serial: String::new(),
            transport: remote_transport(filesystem).to_string(),
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
        });
    }
    items
}

fn sys_block_path(device: &str) -> PathBuf {
    Path::new("/sys/class/block").join(device)
}

fn infer_linux_structure(device: &str, sys_path: &Path) -> String {
    if sys_path.join("partition").exists() {
        "partition".to_string()
    } else if device.starts_with("dm-") {
        "mapper".to_string()
    } else if device.starts_with("md") {
        "raid".to_string()
    } else if device.starts_with("loop") {
        "loop".to_string()
    } else if device.starts_with("sr") {
        "optical".to_string()
    } else if device.starts_with("nvme") {
        "namespace".to_string()
    } else if device.starts_with("zd") {
        "zvol".to_string()
    } else {
        "disk".to_string()
    }
}

fn infer_linux_transport(device: &str) -> &'static str {
    if device.starts_with("nvme") {
        "nvme"
    } else if device.starts_with("vd") {
        "virtio"
    } else if device.starts_with("xvd") {
        "xen"
    } else if device.starts_with("sd") {
        "scsi/sata"
    } else if device.starts_with("dm-") {
        "device-mapper"
    } else if device.starts_with("md") {
        "mdraid"
    } else if device.starts_with("loop") {
        "loop"
    } else {
        "block"
    }
}

fn remote_transport(filesystem: &str) -> &'static str {
    match filesystem.to_ascii_lowercase().as_str() {
        "nfs" | "nfs4" => "nfs",
        "cifs" | "smb3" | "smbfs" => "smb",
        "sshfs" => "ssh",
        "9p" => "9p",
        other => {
            if other.contains("fuse") {
                "fuse"
            } else {
                "remote"
            }
        }
    }
}

fn infer_linux_volume_kind(
    item: &RawDiskInventory,
    dm_name: Option<&str>,
    dm_uuid: Option<&str>,
    md_level: Option<&str>,
) -> String {
    let fs = item.filesystem.to_ascii_lowercase();
    let dm_uuid = dm_uuid.unwrap_or_default().to_ascii_lowercase();
    let dm_name = dm_name.unwrap_or_default().to_ascii_lowercase();

    if item.structure == "raid" || item.device.starts_with("md") {
        if let Some(level) = md_level {
            format!("md-{level}")
        } else {
            "raid-array".to_string()
        }
    } else if item.structure == "mapper" || item.device.starts_with("dm-") {
        if dm_uuid.starts_with("lvm-") || dm_name.contains('-') {
            "lvm-lv".to_string()
        } else if dm_uuid.starts_with("crypt-")
            || dm_name.contains("crypt")
            || dm_name.contains("luks")
        {
            "luks-mapper".to_string()
        } else if dm_uuid.starts_with("mpath-") || dm_name.contains("mpath") {
            "multipath".to_string()
        } else if fs == "lvm2_member" {
            "lvm-pv".to_string()
        } else {
            "mapped-volume".to_string()
        }
    } else if item.structure == "part" || item.structure == "partition" {
        if fs == "lvm2_member" {
            "lvm-member".to_string()
        } else if fs == "linux_raid_member" {
            "raid-member".to_string()
        } else if fs == "crypto_luks" {
            "luks-member".to_string()
        } else if fs == "btrfs" {
            "btrfs-partition".to_string()
        } else {
            "partition".to_string()
        }
    } else if item.device.starts_with("loop") {
        "loop-device".to_string()
    } else if item.device.starts_with("zram") {
        "zram-device".to_string()
    } else if fs == "btrfs" {
        "btrfs-volume".to_string()
    } else if fs == "zfs_member" {
        "zfs-member".to_string()
    } else if fs == "xfs"
        || fs == "ext4"
        || fs == "ext3"
        || fs == "ext2"
        || fs == "ntfs"
        || fs == "refs"
        || fs == "apfs"
        || fs == "vfat"
        || fs == "fat32"
        || fs == "exfat"
    {
        "filesystem-volume".to_string()
    } else if item.filesystem.is_empty() {
        "block-device".to_string()
    } else {
        "filesystem-volume".to_string()
    }
}

fn remote_volume_kind(filesystem: &str) -> String {
    match filesystem.to_ascii_lowercase().as_str() {
        "nfs" | "nfs4" => "nfs-share".to_string(),
        "cifs" | "smb3" | "smbfs" => "smb-share".to_string(),
        "sshfs" => "sshfs-share".to_string(),
        "9p" => "9p-share".to_string(),
        other => {
            if other.contains("fuse") {
                "fuse-remote".to_string()
            } else {
                format!("{other}-mount")
            }
        }
    }
}

fn filesystem_family(filesystem: &str) -> String {
    let fs = filesystem.to_ascii_lowercase();
    if fs.is_empty() {
        String::new()
    } else if matches!(fs.as_str(), "ext2" | "ext3" | "ext4") {
        "ext".to_string()
    } else if fs.starts_with("fat") || fs == "vfat" || fs == "exfat" {
        "fat".to_string()
    } else if fs == "ntfs" || fs == "refs" {
        "windows".to_string()
    } else if fs == "xfs" {
        "xfs".to_string()
    } else if fs == "btrfs" {
        "btrfs".to_string()
    } else if matches!(
        fs.as_str(),
        "nfs" | "nfs4" | "cifs" | "smb3" | "smbfs" | "sshfs" | "9p"
    ) {
        "remote".to_string()
    } else if fs == "apfs" || fs == "hfs" || fs == "hfs+" {
        "apple".to_string()
    } else if fs.contains("lvm") {
        "lvm".to_string()
    } else if fs.contains("raid") {
        "raid".to_string()
    } else if fs.contains("zfs") {
        "zfs".to_string()
    } else {
        fs
    }
}

fn read_lvm_pv_map() -> HashMap<String, String> {
    let Some(output) = command_output(
        "pvs",
        &["--noheadings", "--separator", ";", "-o", "pv_name,vg_name"],
    ) else {
        return HashMap::new();
    };

    output
        .lines()
        .filter_map(|line| {
            let mut parts = line.split(';').map(str::trim);
            let pv = parts.next()?;
            let vg = parts.next()?;
            let dev = pv.strip_prefix("/dev/").unwrap_or(pv);
            if dev.is_empty() || vg.is_empty() {
                None
            } else {
                Some((dev.to_string(), vg.to_string()))
            }
        })
        .collect()
}

fn read_lvm_lv_map() -> HashMap<String, (String, String, String)> {
    let Some(output) = command_output(
        "lvs",
        &[
            "--noheadings",
            "--separator",
            ";",
            "-o",
            "lv_dm_path,vg_name,lv_name,lv_path",
        ],
    ) else {
        return HashMap::new();
    };

    let mut map = HashMap::new();
    for line in output.lines() {
        let mut parts = line.split(';').map(str::trim);
        let dm_path = parts.next().unwrap_or_default();
        let vg_name = parts.next().unwrap_or_default();
        let lv_name = parts.next().unwrap_or_default();
        let lv_path = parts.next().unwrap_or_default();
        let key = dm_path.strip_prefix("/dev/").unwrap_or(dm_path).to_string();
        if !key.is_empty() && !vg_name.is_empty() && !lv_name.is_empty() {
            map.insert(
                key,
                (
                    vg_name.to_string(),
                    lv_name.to_string(),
                    lv_path.to_string(),
                ),
            );
        }
    }
    map
}

type FindmntInfo = (Vec<String>, String, String);

fn read_findmnt_map() -> HashMap<String, FindmntInfo> {
    let Some(output) = command_output("findmnt", &["-J", "-r", "-o", "SOURCE,TARGET,FSTYPE"])
    else {
        return HashMap::new();
    };
    let Ok(value) = serde_json::from_str::<Value>(&output) else {
        return HashMap::new();
    };

    let mut map: HashMap<String, FindmntInfo> = HashMap::new();
    for item in value
        .get("filesystems")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let source = json_string(item, "source").unwrap_or_default();
        let target = json_string(item, "target").unwrap_or_default();
        let fstype = json_string(item, "fstype").unwrap_or_default();
        let Some(dev) = source.strip_prefix("/dev/") else {
            continue;
        };
        map.entry(dev.to_string())
            .and_modify(|entry| entry.0.push(target.clone()))
            .or_insert_with(|| (vec![target], fstype, source));
    }
    map
}

#[derive(Clone)]
struct DmsetupInfo {
    name: String,
    uuid: String,
}

fn read_dmsetup_map() -> HashMap<String, DmsetupInfo> {
    let Some(output) = command_output(
        "dmsetup",
        &["info", "-C", "--noheadings", "-o", "blkdevname,name,uuid"],
    ) else {
        return HashMap::new();
    };

    output
        .lines()
        .filter_map(|line| {
            let parts = line.split_whitespace().collect::<Vec<_>>();
            if parts.len() < 3 {
                return None;
            }
            Some((
                parts[0].to_string(),
                DmsetupInfo {
                    name: parts[1].to_string(),
                    uuid: parts[2].to_string(),
                },
            ))
        })
        .collect()
}

#[derive(Clone)]
struct MdadmInfo {
    level: String,
    name: String,
    uuid: String,
}

fn read_mdadm_map() -> HashMap<String, MdadmInfo> {
    let Some(output) = command_output("mdadm", &["--detail", "--scan"]) else {
        return HashMap::new();
    };

    output
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            if parts.next()? != "ARRAY" {
                return None;
            }
            let dev = parts
                .next()?
                .strip_prefix("/dev/")
                .unwrap_or_default()
                .to_string();
            if dev.is_empty() {
                return None;
            }

            let mut level = String::new();
            let mut name = String::new();
            let mut uuid = String::new();
            for token in parts {
                if let Some(value) = token.strip_prefix("level=") {
                    level = value.to_string();
                } else if let Some(value) = token.strip_prefix("name=") {
                    name = value.to_string();
                } else if let Some(value) = token.strip_prefix("UUID=") {
                    uuid = value.to_string();
                }
            }

            Some((dev, MdadmInfo { level, name, uuid }))
        })
        .collect()
}

#[derive(Clone)]
struct BtrfsInfo {
    label: String,
    uuid: String,
}

fn read_btrfs_map() -> HashMap<String, BtrfsInfo> {
    let Some(output) = command_output("btrfs", &["filesystem", "show"]) else {
        return HashMap::new();
    };

    let mut map = HashMap::new();
    let mut current = BtrfsInfo {
        label: String::new(),
        uuid: String::new(),
    };

    for line in output.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("Label:") {
            let label = rest
                .split("uuid:")
                .next()
                .map(str::trim)
                .unwrap_or_default()
                .trim_matches('\'')
                .to_string();
            let uuid = trimmed
                .split("uuid:")
                .nth(1)
                .map(str::trim)
                .unwrap_or_default()
                .to_string();
            current = BtrfsInfo { label, uuid };
        } else if let Some(path) = trimmed.split_whitespace().last() {
            if let Some(device) = path.strip_prefix("/dev/") {
                map.insert(device.to_string(), current.clone());
            }
        }
    }

    map
}

fn read_zfs_mount_map() -> Vec<(String, String)> {
    let Some(output) = command_output("zfs", &["list", "-H", "-o", "name,mountpoint"]) else {
        return Vec::new();
    };

    output
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let name = parts.next()?.to_string();
            let mount = parts.next()?.to_string();
            if matches!(mount.as_str(), "-" | "legacy" | "none") {
                None
            } else {
                Some((mount, name))
            }
        })
        .collect()
}

fn read_zpool_device_map() -> HashMap<String, String> {
    let Some(output) = command_output("zpool", &["status", "-P"]) else {
        return HashMap::new();
    };

    let mut current_pool = String::new();
    let mut map = HashMap::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if let Some(name) = trimmed.strip_prefix("pool:") {
            current_pool = name.trim().to_string();
            continue;
        }
        if current_pool.is_empty() {
            continue;
        }
        if let Some(dev) = trimmed.strip_prefix("/dev/") {
            let device = dev.split_whitespace().next().unwrap_or_default();
            if !device.is_empty() {
                map.insert(device.to_string(), current_pool.clone());
            }
        }
    }
    map
}

fn read_crypttab_map() -> HashMap<String, String> {
    let Ok(content) = fs::read_to_string("/etc/crypttab") else {
        return HashMap::new();
    };

    content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                return None;
            }
            let mut parts = trimmed.split_whitespace();
            let name = parts.next()?.to_string();
            let source = parts.next()?;
            let device = source.strip_prefix("/dev/")?.to_string();
            Some((device, name))
        })
        .collect()
}

fn is_remote_mount(source: &str, filesystem: &str) -> bool {
    matches!(
        filesystem.to_ascii_lowercase().as_str(),
        "nfs" | "nfs4" | "cifs" | "smb3" | "smbfs" | "sshfs" | "9p"
    ) || source.starts_with("//")
        || source.contains(':')
}

fn logical_stack_for(device: &str, parent_map: &HashMap<String, Option<String>>) -> Vec<String> {
    let mut stack = Vec::new();
    let mut current = Some(device.to_string());

    while let Some(name) = current {
        stack.push(name.clone());
        current = parent_map.get(&name).cloned().flatten();
    }

    stack.reverse();
    stack
}

fn read_string_file(path: &Path) -> Option<String> {
    let value = fs::read_to_string(path).ok()?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn read_bool_file(path: &Path) -> Option<bool> {
    match read_string_file(path)?.as_str() {
        "1" | "y" | "yes" | "true" => Some(true),
        "0" | "n" | "no" | "false" => Some(false),
        _ => None,
    }
}

fn read_dir_names(path: &Path) -> Vec<String> {
    let mut names = fs::read_dir(path)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect::<Vec<_>>();
    names.sort();
    names
}

fn json_string(node: &Value, field: &str) -> Option<String> {
    match node.get(field) {
        Some(Value::String(value)) if !value.is_empty() => Some(value.to_string()),
        Some(Value::Number(value)) => Some(value.to_string()),
        _ => None,
    }
}

fn json_string_list(node: &Value, field: &str) -> Vec<String> {
    node.get(field)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str())
                .filter(|value| !value.is_empty() && *value != "[SWAP]")
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

// ─── Memory ───────────────────────────────────────────────────────────────────

pub fn read_memory() -> Result<RawMemoryInfo> {
    let content = fs::read_to_string("/proc/meminfo")?;
    let mut memory = RawMemoryInfo::default();

    for line in content.lines() {
        let mut parts = line.split_whitespace();
        let Some(key) = parts.next() else {
            continue;
        };
        let Some(val) = parts.next().and_then(|v| v.parse::<u64>().ok()) else {
            continue;
        };

        match key {
            "MemTotal:" => memory.total_kb = val,
            "MemFree:" => memory.free_kb = val,
            "MemAvailable:" => memory.available_kb = val,
            "Buffers:" => memory.buffers_kb = val,
            "Cached:" => memory.cached_kb = val,
            "SwapTotal:" => memory.swap_total_kb = val,
            "SwapFree:" => memory.swap_used_kb = memory.swap_total_kb.saturating_sub(val),
            "Dirty:" => memory.dirty_kb = val,
            _ => {}
        }
    }

    memory.used_kb = memory.total_kb.saturating_sub(memory.available_kb);
    memory.usage_pct = if memory.total_kb > 0 {
        (memory.used_kb as f64 / memory.total_kb as f64 * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };

    if let Ok(content) = fs::read_to_string("/proc/vmstat") {
        memory.vm_pgfault = parse_vmstat_counter(&content, "pgfault");
        memory.vm_pgmajfault = parse_vmstat_counter(&content, "pgmajfault");
        memory.vm_pgpgin = parse_vmstat_counter(&content, "pgpgin");
        memory.vm_pgpgout = parse_vmstat_counter(&content, "pgpgout");
        memory.vm_pswpin = parse_vmstat_counter(&content, "pswpin");
        memory.vm_pswpout = parse_vmstat_counter(&content, "pswpout");
        memory.vm_pgscan = parse_vmstat_sum(&content, "pgscan_");
        memory.vm_pgsteal = parse_vmstat_sum(&content, "pgsteal_");
    }

    Ok(memory)
}

// ─── System ───────────────────────────────────────────────────────────────────

pub fn read_system() -> Result<RawSystemInfo> {
    Ok(RawSystemInfo {
        hostname: fs::read_to_string("/proc/sys/kernel/hostname")
            .unwrap_or_default()
            .trim()
            .to_string(),
        os_name: std::env::consts::OS.to_string(),
        os_version: fs::read_to_string("/etc/os-release")
            .unwrap_or_default()
            .lines()
            .find(|l| l.starts_with("PRETTY_NAME="))
            .map(|l| {
                l.trim_start_matches("PRETTY_NAME=")
                    .trim_matches('"')
                    .to_string()
            })
            .unwrap_or_else(|| "Linux".to_string()),
        kernel_version: fs::read_to_string("/proc/version")
            .unwrap_or_default()
            .split_whitespace()
            .nth(2)
            .unwrap_or("unknown")
            .to_string(),
        uptime_seconds: fs::read_to_string("/proc/uptime")
            .unwrap_or_default()
            .split_whitespace()
            .next()
            .and_then(|s| s.parse::<f64>().ok())
            .map(|f| f as u64)
            .unwrap_or(0),
        cpu_count: {
            let n = unsafe { libc::sysconf(libc::_SC_NPROCESSORS_ONLN) };
            if n > 0 {
                n as u32
            } else {
                1
            }
        },
        architecture: std::env::consts::ARCH.to_string(),
    })
}

// ─── Linux-specific metrics ──────────────────────────────────────────────────

pub fn read_linux_metrics() -> Result<RawLinuxMetrics> {
    Ok(RawLinuxMetrics {
        cgroup: read_cgroup_v2_metrics(),
        psi: read_psi_metrics(),
    })
}

fn read_cgroup_v2_metrics() -> Option<RawCgroupMetrics> {
    let cgroup_root = Path::new("/sys/fs/cgroup");
    if !cgroup_root.join("cgroup.controllers").exists() {
        return None;
    }

    let relative_path = parse_cgroup_v2_path(&fs::read_to_string("/proc/self/cgroup").ok()?)?;
    let full_path = join_cgroup_path(cgroup_root, &relative_path);

    let memory_current_bytes = read_u64_file(&full_path.join("memory.current")).unwrap_or(0);
    let memory_max_bytes = read_limit_u64_file(&full_path.join("memory.max"));
    let memory_swap_current_bytes =
        read_u64_file(&full_path.join("memory.swap.current")).unwrap_or(0);
    let memory_swap_max_bytes = read_limit_u64_file(&full_path.join("memory.swap.max"));
    let pids_current = read_u64_file(&full_path.join("pids.current")).unwrap_or(0);
    let pids_max = read_limit_u64_file(&full_path.join("pids.max"));

    let cpu_stat = parse_cpu_stat(&fs::read_to_string(full_path.join("cpu.stat")).ok()?);
    let (cpu_quota_usec, cpu_period_usec) =
        parse_cpu_max(&fs::read_to_string(full_path.join("cpu.max")).unwrap_or_default());

    let memory_usage_pct = memory_max_bytes
        .filter(|max| *max > 0)
        .map(|max| (memory_current_bytes as f64 / max as f64 * 100.0).clamp(0.0, 100.0))
        .unwrap_or(0.0);

    Some(RawCgroupMetrics {
        version: 2,
        path: relative_path,
        memory_current_bytes,
        memory_max_bytes,
        memory_swap_current_bytes,
        memory_swap_max_bytes,
        memory_usage_pct,
        pids_current,
        pids_max,
        cpu_usage_usec: cpu_stat.0,
        cpu_user_usec: cpu_stat.1,
        cpu_system_usec: cpu_stat.2,
        cpu_nr_periods: cpu_stat.3,
        cpu_nr_throttled: cpu_stat.4,
        cpu_throttled_usec: cpu_stat.5,
        cpu_quota_usec,
        cpu_period_usec,
    })
}

fn read_psi_metrics() -> Option<RawPsiMetrics> {
    let cpu = read_pressure_metric(Path::new("/proc/pressure/cpu"));
    let memory = read_pressure_metric(Path::new("/proc/pressure/memory"));
    let io = read_pressure_metric(Path::new("/proc/pressure/io"));

    if cpu.is_none() && memory.is_none() && io.is_none() {
        return None;
    }

    Some(RawPsiMetrics {
        cpu: cpu.unwrap_or_default(),
        memory: memory.unwrap_or_default(),
        io: io.unwrap_or_default(),
    })
}

fn read_pressure_metric(path: &Path) -> Option<RawPressureMetric> {
    parse_pressure_metric(&fs::read_to_string(path).ok()?)
}

fn parse_pressure_metric(content: &str) -> Option<RawPressureMetric> {
    let mut metric = RawPressureMetric::default();

    for line in content.lines() {
        let (kind, window) = parse_pressure_line(line)?;
        match kind {
            "some" => metric.some = Some(window),
            "full" => metric.full = Some(window),
            _ => {}
        }
    }

    if metric.some.is_none() && metric.full.is_none() {
        None
    } else {
        Some(metric)
    }
}

fn parse_pressure_line(line: &str) -> Option<(&str, RawPressureWindow)> {
    let mut parts = line.split_whitespace();
    let kind = parts.next()?;

    let mut window = RawPressureWindow::default();
    for part in parts {
        let (key, value) = part.split_once('=')?;
        match key {
            "avg10" => window.avg10 = value.parse().ok()?,
            "avg60" => window.avg60 = value.parse().ok()?,
            "avg300" => window.avg300 = value.parse().ok()?,
            "total" => window.total = value.parse().ok()?,
            _ => {}
        }
    }

    Some((kind, window))
}

fn parse_cgroup_v2_path(content: &str) -> Option<String> {
    content.lines().find_map(|line| {
        let mut parts = line.splitn(3, ':');
        let hierarchy = parts.next()?;
        let controllers = parts.next()?;
        let path = parts.next()?;
        if hierarchy == "0" && controllers.is_empty() {
            Some(path.to_string())
        } else {
            None
        }
    })
}

fn join_cgroup_path(root: &Path, relative_path: &str) -> PathBuf {
    if relative_path == "/" {
        root.to_path_buf()
    } else {
        root.join(relative_path.trim_start_matches('/'))
    }
}

fn read_u64_file(path: &Path) -> Option<u64> {
    fs::read_to_string(path).ok()?.trim().parse().ok()
}

fn read_limit_u64_file(path: &Path) -> Option<u64> {
    parse_limit_u64(&fs::read_to_string(path).ok()?)
}

fn parse_limit_u64(content: &str) -> Option<u64> {
    let trimmed = content.trim();
    if trimmed == "max" || trimmed.is_empty() {
        None
    } else {
        trimmed.parse().ok()
    }
}

fn parse_cpu_stat(content: &str) -> (u64, u64, u64, u64, u64, u64) {
    let mut usage_usec = 0;
    let mut user_usec = 0;
    let mut system_usec = 0;
    let mut nr_periods = 0;
    let mut nr_throttled = 0;
    let mut throttled_usec = 0;

    for line in content.lines() {
        let mut parts = line.split_whitespace();
        let Some(key) = parts.next() else {
            continue;
        };
        let Some(value) = parts.next().and_then(|value| value.parse::<u64>().ok()) else {
            continue;
        };

        match key {
            "usage_usec" => usage_usec = value,
            "user_usec" => user_usec = value,
            "system_usec" => system_usec = value,
            "nr_periods" => nr_periods = value,
            "nr_throttled" => nr_throttled = value,
            "throttled_usec" => throttled_usec = value,
            _ => {}
        }
    }

    (
        usage_usec,
        user_usec,
        system_usec,
        nr_periods,
        nr_throttled,
        throttled_usec,
    )
}

fn parse_cpu_max(content: &str) -> (Option<u64>, Option<u64>) {
    let mut parts = content.split_whitespace();
    let quota = parts.next().and_then(parse_cpu_max_part);
    let period = parts.next().and_then(parse_cpu_max_part);
    (quota, period)
}

fn parse_cpu_max_part(part: &str) -> Option<u64> {
    if part == "max" {
        None
    } else {
        part.parse().ok()
    }
}

fn parse_socket_table(content: &str, is_udp: bool, connections: &mut RawNetConnections) {
    for line in content.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }

        let state = parts[3];
        if is_udp {
            connections.udp_total += 1;
            match state {
                "01" => connections.udp_established += 1,
                "07" => connections.udp_close += 1,
                _ => connections.udp_other += 1,
            }
        } else {
            connections.total += 1;
            match state {
                "01" => connections.established += 1,
                "02" => connections.tcp_syn_sent += 1,
                "03" => connections.tcp_syn_recv += 1,
                "04" => connections.tcp_fin_wait1 += 1,
                "05" => connections.tcp_fin_wait2 += 1,
                "06" => connections.tcp_time_wait += 1,
                "07" => connections.tcp_close += 1,
                "08" => connections.tcp_close_wait += 1,
                "09" => connections.tcp_last_ack += 1,
                "0A" => connections.tcp_listen += 1,
                "0B" => connections.tcp_closing += 1,
                _ => connections.tcp_other += 1,
            }
        }
    }
}

fn read_snmp_counter(path: &str, section: &str, field: &str) -> Option<u64> {
    let content = fs::read_to_string(path).ok()?;
    let mut lines = content.lines();

    while let Some(header) = lines.next() {
        let values = lines.next()?;
        let Some(header_body) = header.strip_prefix(&format!("{section}:")) else {
            continue;
        };
        let Some(value_body) = values.strip_prefix(&format!("{section}:")) else {
            continue;
        };

        let headers: Vec<&str> = header_body.split_whitespace().collect();
        let values: Vec<&str> = value_body.split_whitespace().collect();
        let index = headers.iter().position(|name| *name == field)?;
        return values.get(index).and_then(|value| value.parse().ok());
    }

    None
}

fn parse_vmstat_counter(content: &str, key: &str) -> u64 {
    content
        .lines()
        .find_map(|line| {
            let mut parts = line.split_whitespace();
            let name = parts.next()?;
            let value = parts.next()?.parse::<u64>().ok()?;
            if name == key {
                Some(value)
            } else {
                None
            }
        })
        .unwrap_or(0)
}

fn parse_vmstat_sum(content: &str, prefix: &str) -> u64 {
    content
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let name = parts.next()?;
            let value = parts.next()?.parse::<u64>().ok()?;
            if name.starts_with(prefix) {
                Some(value)
            } else {
                None
            }
        })
        .sum()
}

// ─── Process ──────────────────────────────────────────────────────────────────

pub fn page_size() -> u64 {
    unsafe { libc::sysconf(libc::_SC_PAGESIZE) as u64 }
}

pub fn clock_ticks() -> f64 {
    unsafe { libc::sysconf(libc::_SC_CLK_TCK) as f64 }
}

pub fn num_cpus() -> f64 {
    let n = unsafe { libc::sysconf(libc::_SC_NPROCESSORS_ONLN) };
    if n > 0 {
        n as f64
    } else {
        1.0
    }
}

fn uid_to_user(pid: u32) -> String {
    use std::os::unix::fs::MetadataExt;
    let meta = fs::metadata(format!("/proc/{}", pid)).ok();
    if let Some(m) = meta {
        let uid = m.uid();
        if let Ok(passwd) = fs::read_to_string("/etc/passwd") {
            for line in passwd.lines() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 3 && parts[2].parse::<u32>().ok() == Some(uid) {
                    return parts[0].to_string();
                }
            }
        }
        return uid.to_string();
    }
    String::new()
}

pub fn read_processes() -> Result<Vec<RawProcReading>> {
    let proc_dir = fs::read_dir("/proc")?;
    let mut result = Vec::new();

    for entry in proc_dir.flatten() {
        let fname = entry.file_name();
        let pid_str = fname.to_string_lossy();
        let pid: u32 = match pid_str.parse() {
            Ok(p) => p,
            Err(_) => continue,
        };

        let stat = match fs::read_to_string(format!("/proc/{}/stat", pid)) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let start = match stat.find('(') {
            Some(p) => p,
            None => continue,
        };
        let end = match stat.rfind(')') {
            Some(p) => p,
            None => continue,
        };
        let name = stat[start + 1..end].to_string();
        let rest: Vec<&str> = stat[end + 2..].split_whitespace().collect();

        let state_char = rest.first().and_then(|s| s.chars().next()).unwrap_or('?');
        let utime: u64 = rest.get(11).and_then(|v| v.parse().ok()).unwrap_or(0);
        let stime: u64 = rest.get(12).and_then(|v| v.parse().ok()).unwrap_or(0);
        let vsize: u64 = rest.get(20).and_then(|v| v.parse().ok()).unwrap_or(0);
        let rss: u64 = rest.get(21).and_then(|v| v.parse().ok()).unwrap_or(0);
        let threads: u32 = rest.get(17).and_then(|v| v.parse().ok()).unwrap_or(0);

        let cmdline = fs::read_to_string(format!("/proc/{}/cmdline", pid))
            .unwrap_or_default()
            .replace('\0', " ")
            .trim()
            .to_string();

        let (io_read_bytes, io_write_bytes) = {
            let io = fs::read_to_string(format!("/proc/{}/io", pid)).unwrap_or_default();
            let mut rb = 0u64;
            let mut wb = 0u64;
            for line in io.lines() {
                if line.starts_with("read_bytes:") {
                    rb = line
                        .split_whitespace()
                        .nth(1)
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(0);
                } else if line.starts_with("write_bytes:") {
                    wb = line
                        .split_whitespace()
                        .nth(1)
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(0);
                }
            }
            (rb, wb)
        };

        result.push(RawProcReading {
            pid,
            name,
            cmdline,
            utime,
            stime,
            vsize,
            rss,
            threads,
            fd_count: fs::read_dir(format!("/proc/{}/fd", pid))
                .map(|d| d.count() as u32)
                .unwrap_or(0),
            state_char,
            user: uid_to_user(pid),
            io_read_bytes,
            io_write_bytes,
            cpu_pct_hint: None,
        });
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pressure_lines() {
        let (kind, window) =
            parse_pressure_line("some avg10=1.23 avg60=0.45 avg300=0.67 total=890").unwrap();

        assert_eq!(kind, "some");
        assert!((window.avg10 - 1.23).abs() < f64::EPSILON);
        assert!((window.avg60 - 0.45).abs() < f64::EPSILON);
        assert!((window.avg300 - 0.67).abs() < f64::EPSILON);
        assert_eq!(window.total, 890);
    }

    #[test]
    fn parses_pressure_metrics_with_some_and_full() {
        let metric = parse_pressure_metric(
            "some avg10=1.00 avg60=2.00 avg300=3.00 total=4\nfull avg10=5.00 avg60=6.00 avg300=7.00 total=8\n",
        )
        .unwrap();

        assert_eq!(metric.some.unwrap().total, 4);
        assert_eq!(metric.full.unwrap().total, 8);
    }

    #[test]
    fn parses_cgroup_v2_path() {
        let path = parse_cgroup_v2_path("0::/user.slice/app.slice/sysray.service\n").unwrap();
        assert_eq!(path, "/user.slice/app.slice/sysray.service");
    }

    #[test]
    fn parses_limit_values() {
        assert_eq!(parse_limit_u64("12345\n"), Some(12345));
        assert_eq!(parse_limit_u64("max\n"), None);
    }

    #[test]
    fn parses_cpu_max_values() {
        assert_eq!(parse_cpu_max("max 100000"), (None, Some(100000)));
        assert_eq!(parse_cpu_max("20000 100000"), (Some(20000), Some(100000)));
    }

    #[test]
    fn parses_vmstat_counters_and_prefix_sums() {
        let content = "pgfault 10\npgmajfault 2\npgscan_kswapd 5\npgscan_direct 7\n";
        assert_eq!(parse_vmstat_counter(content, "pgfault"), 10);
        assert_eq!(parse_vmstat_counter(content, "pgmajfault"), 2);
        assert_eq!(parse_vmstat_sum(content, "pgscan_"), 12);
    }

    #[test]
    fn parses_socket_tables_into_state_buckets() {
        let tcp = "  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode\n   0: 00000000:0016 00000000:0000 0A 00000000:00000000 00:00000000 00000000   100        0 1 1 0000000000000000 100 0 0 10 0\n   1: 0100007F:9C40 0100007F:1F90 01 00000000:00000000 00:00000000 00000000   100        0 1 1 0000000000000000 100 0 0 10 0\n";
        let udp = "  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode ref pointer drops\n   0: 00000000:0035 00000000:0000 07 00000000:00000000 00:00000000 00000000   0        0 1 2 0000000000000000 0\n";
        let mut connections = RawNetConnections::default();

        parse_socket_table(tcp, false, &mut connections);
        parse_socket_table(udp, true, &mut connections);

        assert_eq!(connections.total, 2);
        assert_eq!(connections.established, 1);
        assert_eq!(connections.tcp_listen, 1);
        assert_eq!(connections.udp_total, 1);
        assert_eq!(connections.udp_close, 1);
    }
}
