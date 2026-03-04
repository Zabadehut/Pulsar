use crate::platform::api::{
    RawCpuReading, RawCpuStat, RawDiskInventory, RawDiskSpace, RawDiskStat, RawLinuxMetrics,
    RawMemoryInfo, RawNetConnections, RawNetStat, RawProcReading, RawSystemInfo,
};
use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::ffi::OsString;
use std::mem::size_of;
use std::process::Command;
use std::ptr;
use std::slice;
use std::sync::OnceLock;
use windows::Win32::Foundation::{
    CloseHandle, ERROR_INSUFFICIENT_BUFFER, FILETIME, HANDLE, NO_ERROR,
};
use windows::Win32::NetworkManagement::IpHelper::{
    FreeMibTable, GetIfTable2, GetTcp6Table2, GetTcpStatisticsEx2, GetTcpTable2,
    GetUdpStatisticsEx, MIB_IF_ROW2, MIB_IF_TABLE2, MIB_TCP6ROW2, MIB_TCP6TABLE2, MIB_TCPROW2,
    MIB_TCPSTATS2, MIB_TCPTABLE2, MIB_TCP_STATE_CLOSED, MIB_TCP_STATE_CLOSE_WAIT,
    MIB_TCP_STATE_CLOSING, MIB_TCP_STATE_ESTAB, MIB_TCP_STATE_FIN_WAIT1, MIB_TCP_STATE_FIN_WAIT2,
    MIB_TCP_STATE_LAST_ACK, MIB_TCP_STATE_LISTEN, MIB_TCP_STATE_SYN_RCVD, MIB_TCP_STATE_SYN_SENT,
    MIB_TCP_STATE_TIME_WAIT, MIB_UDPSTATS,
};
use windows::Win32::Networking::WinSock::{AF_INET, AF_INET6};
use windows::Win32::System::SystemInformation::{
    GetNativeSystemInfo, GetTickCount64, GlobalMemoryStatusEx, MEMORYSTATUSEX, SYSTEM_INFO,
};
use windows::Win32::System::Threading::{
    GetProcessHandleCount, GetProcessIoCounters, GetProcessTimes, GetSystemTimes, OpenProcess,
    IO_COUNTERS, PROCESS_QUERY_LIMITED_INFORMATION,
};

pub fn read_cpu() -> Result<RawCpuReading> {
    let global = read_native_cpu_stat().unwrap_or_default();
    let total_usage = powershell_json(
        "(Get-Counter '\\Processor(_Total)\\% Processor Time').CounterSamples | Select-Object CookedValue | ConvertTo-Json -Compress",
    )
    .as_ref()
    .and_then(|value| json_field_f64(value, "CookedValue"))
    .unwrap_or(0.0);

    let per_core = powershell_json(
        "(Get-Counter '\\Processor(*)\\% Processor Time').CounterSamples | Where-Object { $_.InstanceName -notin @('_Total','Idle') } | Sort-Object InstanceName | Select-Object InstanceName,CookedValue | ConvertTo-Json -Compress",
    )
    .map(json_values)
    .unwrap_or_default()
    .into_iter()
    .filter_map(|value| json_field_f64(&value, "CookedValue"))
    .collect();

    Ok(RawCpuReading {
        global,
        direct_global_usage_pct: Some(total_usage.clamp(0.0, 100.0)),
        direct_iowait_pct: Some(0.0),
        direct_steal_pct: Some(0.0),
        direct_per_core_usage_pct: per_core,
        ..RawCpuReading::default()
    })
}

pub fn read_mount_map() -> HashMap<String, String> {
    disk_entries()
        .into_iter()
        .map(|entry| (entry.device.clone(), entry.mount_point.clone()))
        .collect()
}

pub fn read_disk_space(mount: &str) -> RawDiskSpace {
    disk_entries()
        .into_iter()
        .find(|entry| entry.mount_point.eq_ignore_ascii_case(mount))
        .map(|entry| RawDiskSpace {
            total_gb: entry.total_bytes as f64 / 1_000_000_000.0,
            used_gb: entry.used_bytes as f64 / 1_000_000_000.0,
            free_gb: entry.free_bytes as f64 / 1_000_000_000.0,
            usage_pct: if entry.total_bytes > 0 {
                (entry.used_bytes as f64 / entry.total_bytes as f64 * 100.0).clamp(0.0, 100.0)
            } else {
                0.0
            },
        })
        .unwrap_or_default()
}

pub fn read_disks() -> Result<Vec<RawDiskStat>> {
    Ok(disk_entries()
        .into_iter()
        .map(|entry| RawDiskStat {
            device: entry.device,
            ..RawDiskStat::default()
        })
        .collect())
}

pub fn read_disk_inventory() -> Result<Vec<RawDiskInventory>> {
    let values = powershell_json(
        "$volByLetter = @{}; \
         Get-Volume | ForEach-Object { if ($_.DriveLetter) { $volByLetter[[string]$_.DriveLetter] = $_ } }; \
         $parts = Get-Partition | Sort-Object DiskNumber, PartitionNumber; \
         $out = foreach ($part in $parts) { \
           $letter = if ($part.DriveLetter) { [string]$part.DriveLetter } else { $null }; \
           $vol = if ($letter -and $volByLetter.ContainsKey($letter)) { $volByLetter[$letter] } else { $null }; \
           $disk = Get-Disk -Number $part.DiskNumber -ErrorAction SilentlyContinue; \
           [pscustomobject]@{ \
             Device = if ($letter) { '{0}:' -f $letter } else { 'disk{0}-part{1}' -f $part.DiskNumber, $part.PartitionNumber }; \
             Parent = if ($disk) { 'disk{0}' -f $disk.Number } else { $null }; \
             Structure = if ($part.Type) { [string]$part.Type } else { 'partition' }; \
             FileSystem = if ($vol) { [string]$vol.FileSystem } else { $null }; \
             Label = if ($vol) { [string]$vol.FileSystemLabel } else { $null }; \
             Uuid = if ($vol) { [string]$vol.UniqueId } else { $null }; \
             PartUuid = if ($part.Guid) { [string]$part.Guid } else { $null }; \
             Model = if ($disk) { [string]$disk.FriendlyName } else { $null }; \
             Serial = if ($disk) { [string]$disk.SerialNumber } else { $null }; \
             Transport = if ($disk) { [string]$disk.BusType } else { $null }; \
             Reference = if ($disk -and $disk.UniqueId) { [string]$disk.UniqueId } elseif ($vol -and $vol.Path) { [string]$vol.Path } else { $null }; \
             MountPoints = @($part.AccessPaths); \
           } \
         }; \
         $out | ConvertTo-Json -Compress",
    )
    .map(json_values)
    .unwrap_or_default();

    let mut inventory = Vec::new();
    for value in values {
        let mount_points = value
            .get("MountPoints")
            .map(|item| match item {
                Value::Array(items) => items
                    .iter()
                    .filter_map(Value::as_str)
                    .filter(|entry| !entry.is_empty())
                    .map(ToString::to_string)
                    .collect::<Vec<_>>(),
                Value::String(single) if !single.is_empty() => vec![single.to_string()],
                _ => Vec::new(),
            })
            .unwrap_or_default();

        inventory.push(RawDiskInventory {
            device: json_field_string(&value, "Device").unwrap_or_default(),
            parent: json_field_string(&value, "Parent"),
            structure: json_field_string(&value, "Structure").unwrap_or_default(),
            volume_kind: "windows-volume".to_string(),
            filesystem: json_field_string(&value, "FileSystem").unwrap_or_default(),
            filesystem_family: json_field_string(&value, "FileSystem")
                .map(|fs| match fs.to_ascii_lowercase().as_str() {
                    "ntfs" | "refs" => "windows".to_string(),
                    "fat" | "fat32" | "exfat" => "fat".to_string(),
                    other => other.to_string(),
                })
                .unwrap_or_default(),
            label: json_field_string(&value, "Label").unwrap_or_default(),
            uuid: json_field_string(&value, "Uuid").unwrap_or_default(),
            part_uuid: json_field_string(&value, "PartUuid").unwrap_or_default(),
            model: json_field_string(&value, "Model").unwrap_or_default(),
            serial: json_field_string(&value, "Serial").unwrap_or_default(),
            transport: json_field_string(&value, "Transport").unwrap_or_default(),
            reference: json_field_string(&value, "Reference").unwrap_or_default(),
            scheduler: String::new(),
            rotational: None,
            removable: None,
            read_only: None,
            mount_points,
            logical_stack: Vec::new(),
            slaves: Vec::new(),
            holders: Vec::new(),
            children: Vec::new(),
        });
    }

    inventory.extend(read_remote_mount_inventory());

    let mut children_map: HashMap<String, Vec<String>> = HashMap::new();
    for item in &inventory {
        if let Some(parent) = &item.parent {
            children_map
                .entry(parent.clone())
                .or_default()
                .push(item.device.clone());
        }
    }
    let parent_map: HashMap<String, Option<String>> = inventory
        .iter()
        .map(|item| (item.device.clone(), item.parent.clone()))
        .collect();
    for item in &mut inventory {
        item.children = children_map.get(&item.device).cloned().unwrap_or_default();
        item.logical_stack = logical_stack_from_map(&item.device, &parent_map);
    }

    Ok(inventory)
}

fn read_remote_mount_inventory() -> Vec<RawDiskInventory> {
    let values = powershell_json(
        "$maps = @(); \
         if (Get-Command Get-SmbMapping -ErrorAction SilentlyContinue) { \
           $maps += Get-SmbMapping | Select-Object LocalPath,RemotePath,Status; \
         }; \
         $maps | ConvertTo-Json -Compress",
    )
    .map(json_values)
    .unwrap_or_default();

    values
        .into_iter()
        .filter_map(|value| {
            let device = json_field_string(&value, "LocalPath")
                .or_else(|| json_field_string(&value, "RemotePath"))?;
            let remote = json_field_string(&value, "RemotePath").unwrap_or_default();
            let label = if remote.is_empty() {
                device.clone()
            } else {
                remote.clone()
            };
            let logical_stack_entry = if remote.is_empty() {
                device.clone()
            } else {
                remote.clone()
            };
            Some(RawDiskInventory {
                device: device.clone(),
                parent: None,
                structure: "remote-mount".to_string(),
                volume_kind: "smb-share".to_string(),
                filesystem: "smb".to_string(),
                filesystem_family: "remote".to_string(),
                label,
                uuid: String::new(),
                part_uuid: String::new(),
                model: String::new(),
                serial: String::new(),
                transport: "smb".to_string(),
                reference: remote,
                scheduler: String::new(),
                rotational: None,
                removable: None,
                read_only: None,
                mount_points: vec![device.clone()],
                logical_stack: vec![logical_stack_entry],
                slaves: Vec::new(),
                holders: Vec::new(),
                children: Vec::new(),
            })
        })
        .collect()
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

pub fn read_net_connections() -> RawNetConnections {
    read_native_net_connections().unwrap_or_else(|| {
        let total =
            powershell_number_u32("(Get-NetTCPConnection | Measure-Object).Count").unwrap_or(0);
        let established = powershell_number_u32(
            "(Get-NetTCPConnection -State Established | Measure-Object).Count",
        )
        .unwrap_or(0);
        RawNetConnections {
            total,
            established,
            ..RawNetConnections::default()
        }
    })
}

pub fn read_network() -> Result<Vec<RawNetStat>> {
    if let Some(stats) = read_native_network_stats() {
        return Ok(stats);
    }

    let values = powershell_json(
        "Get-NetAdapterStatistics | Select-Object Name,ReceivedBytes,SentBytes,ReceivedUnicastPackets,SentUnicastPackets,ReceivedPacketErrors,OutboundPacketErrors,ReceivedDiscardedPackets,OutboundDiscardedPackets | ConvertTo-Json -Compress",
    )
    .map(json_values)
    .unwrap_or_default();

    let mut stats = Vec::new();
    for value in values {
        let interface = json_field_string(&value, "Name").unwrap_or_default();
        if interface.is_empty() {
            continue;
        }

        stats.push(RawNetStat {
            interface,
            rx_bytes: json_field_u64(&value, "ReceivedBytes").unwrap_or(0),
            rx_packets: json_field_u64(&value, "ReceivedUnicastPackets").unwrap_or(0),
            rx_errors: json_field_u64(&value, "ReceivedPacketErrors").unwrap_or(0),
            rx_dropped: json_field_u64(&value, "ReceivedDiscardedPackets").unwrap_or(0),
            tx_bytes: json_field_u64(&value, "SentBytes").unwrap_or(0),
            tx_packets: json_field_u64(&value, "SentUnicastPackets").unwrap_or(0),
            tx_errors: json_field_u64(&value, "OutboundPacketErrors").unwrap_or(0),
            tx_dropped: json_field_u64(&value, "OutboundDiscardedPackets").unwrap_or(0),
        });
    }

    Ok(stats)
}

pub fn read_memory() -> Result<RawMemoryInfo> {
    let mut status = MEMORYSTATUSEX {
        dwLength: size_of::<MEMORYSTATUSEX>() as u32,
        ..Default::default()
    };

    unsafe {
        let _ = GlobalMemoryStatusEx(&mut status);
    }

    let total_kb = status.ullTotalPhys / 1024;
    let available_kb = status.ullAvailPhys / 1024;
    let used_kb = total_kb.saturating_sub(available_kb);
    let free_kb = available_kb;
    let swap_total_kb = status.ullTotalPageFile.saturating_sub(status.ullTotalPhys) / 1024;
    let swap_used_kb = status
        .ullTotalPageFile
        .saturating_sub(status.ullAvailPageFile)
        .saturating_sub(status.ullTotalPhys.saturating_sub(status.ullAvailPhys))
        / 1024;
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
        cached_kb: 0,
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
    })
}

pub fn read_system() -> Result<RawSystemInfo> {
    let mut info = SYSTEM_INFO::default();
    unsafe {
        GetNativeSystemInfo(&mut info);
    }
    let details = read_system_details();
    let os_name = details
        .as_ref()
        .and_then(|value| json_field_string(value, "Caption"))
        .unwrap_or_else(|| "Windows".to_string());
    let version = details
        .as_ref()
        .and_then(|value| json_field_string(value, "Version"))
        .unwrap_or_default();
    let build = details
        .as_ref()
        .and_then(|value| json_field_string(value, "BuildNumber"))
        .unwrap_or_default();

    Ok(RawSystemInfo {
        hostname: details
            .as_ref()
            .and_then(|value| json_field_string(value, "CSName"))
            .or_else(|| std::env::var("COMPUTERNAME").ok())
            .unwrap_or_default(),
        os_name,
        os_version: format_version_and_build(&version, &build),
        uptime_seconds: unsafe { GetTickCount64() / 1000 },
        architecture: std::env::consts::ARCH.to_string(),
        cpu_count: info.dwNumberOfProcessors,
        kernel_version: version,
    })
}

pub fn page_size() -> u64 {
    4096
}

pub fn clock_ticks() -> f64 {
    10_000_000.0
}

pub fn num_cpus() -> f64 {
    std::thread::available_parallelism()
        .map(|n| n.get() as f64)
        .unwrap_or(1.0)
}

pub fn read_processes() -> Result<Vec<RawProcReading>> {
    let values = powershell_json(
        "$perfByPid = @{}; \
         Get-CimInstance Win32_PerfFormattedData_PerfProc_Process | \
         Where-Object { $_.IDProcess -gt 0 -and $_.Name -notin @('_Total', 'Idle') } | \
         ForEach-Object { $perfByPid[[string]$_.IDProcess] = [double]$_.PercentProcessorTime }; \
         Get-CimInstance Win32_Process | ForEach-Object { \
           $owner = Invoke-CimMethod -InputObject $_ -MethodName GetOwner -ErrorAction SilentlyContinue; \
           [pscustomobject]@{ \
             ProcessId = $_.ProcessId; \
             Name = $_.Name; \
             CommandLine = $_.CommandLine; \
             WorkingSetSize = $_.WorkingSetSize; \
             VirtualSize = $_.VirtualSize; \
             ThreadCount = $_.ThreadCount; \
             UserModeTime = $_.UserModeTime; \
             KernelModeTime = $_.KernelModeTime; \
             User = if ($owner -and $owner.ReturnValue -eq 0 -and $owner.User) { if ($owner.Domain) { '{0}\\{1}' -f $owner.Domain, $owner.User } else { $owner.User } } else { $null }; \
             CpuPct = if ($perfByPid.ContainsKey([string]$_.ProcessId)) { $perfByPid[[string]$_.ProcessId] } else { $null } \
           } \
         } | ConvertTo-Json -Compress",
    )
    .map(json_values)
    .unwrap_or_default();

    let mut processes = Vec::new();
    for value in values {
        let pid = match json_field_u64(&value, "ProcessId") {
            Some(pid) if pid > 0 => pid as u32,
            _ => continue,
        };

        let working_set = json_field_u64(&value, "WorkingSetSize").unwrap_or(0);
        let native = read_process_native_metrics(pid);
        processes.push(RawProcReading {
            pid,
            name: json_field_string(&value, "Name").unwrap_or_default(),
            cmdline: json_field_string(&value, "CommandLine").unwrap_or_default(),
            utime: native
                .as_ref()
                .map(|metrics| metrics.utime)
                .unwrap_or_else(|| json_field_u64(&value, "UserModeTime").unwrap_or(0)),
            stime: native
                .as_ref()
                .map(|metrics| metrics.stime)
                .unwrap_or_else(|| json_field_u64(&value, "KernelModeTime").unwrap_or(0)),
            vsize: json_field_u64(&value, "VirtualSize").unwrap_or(0),
            rss: working_set / page_size(),
            threads: json_field_u64(&value, "ThreadCount").unwrap_or(0) as u32,
            fd_count: native.as_ref().map(|metrics| metrics.fd_count).unwrap_or(0),
            state_char: '?',
            user: json_field_string(&value, "User").unwrap_or_default(),
            io_read_bytes: native
                .as_ref()
                .map(|metrics| metrics.io_read_bytes)
                .unwrap_or(0),
            io_write_bytes: native
                .as_ref()
                .map(|metrics| metrics.io_write_bytes)
                .unwrap_or(0),
            cpu_pct_hint: json_field_f64(&value, "CpuPct"),
        });
    }

    Ok(processes)
}

pub fn read_linux_metrics() -> Result<RawLinuxMetrics> {
    Ok(RawLinuxMetrics::default())
}

#[derive(Debug, Clone)]
struct DiskEntry {
    device: String,
    mount_point: String,
    total_bytes: u64,
    free_bytes: u64,
    used_bytes: u64,
}

#[derive(Debug, Clone, Default)]
struct ProcessNativeMetrics {
    utime: u64,
    stime: u64,
    fd_count: u32,
    io_read_bytes: u64,
    io_write_bytes: u64,
}

fn disk_entries() -> Vec<DiskEntry> {
    let values = powershell_json(
        "Get-CimInstance Win32_LogicalDisk -Filter \"DriveType=3\" | Select-Object DeviceID,Size,FreeSpace | ConvertTo-Json -Compress",
    )
    .map(json_values)
    .unwrap_or_default();

    values
        .into_iter()
        .filter_map(|value| {
            let device = json_field_string(&value, "DeviceID")?;
            let total_bytes = json_field_u64(&value, "Size").unwrap_or(0);
            let free_bytes = json_field_u64(&value, "FreeSpace").unwrap_or(0);
            Some(DiskEntry {
                device: device.clone(),
                mount_point: device,
                total_bytes,
                free_bytes,
                used_bytes: total_bytes.saturating_sub(free_bytes),
            })
        })
        .collect()
}

fn powershell_json(script: &str) -> Option<Value> {
    let output = powershell(script)?;
    serde_json::from_str(&output).ok()
}

fn powershell_number_u32(script: &str) -> Option<u32> {
    powershell(script)?.trim().parse().ok()
}

fn read_system_details() -> Option<Value> {
    powershell_json(
        "Get-CimInstance Win32_OperatingSystem | \
         Select-Object Caption,Version,BuildNumber,CSName | \
         ConvertTo-Json -Compress",
    )
}

fn powershell(script: &str) -> Option<String> {
    let command = powershell_command()?;
    let output = Command::new(command)
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .output()
        .ok()?;
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

fn powershell_command() -> Option<&'static OsString> {
    static COMMAND: OnceLock<Option<OsString>> = OnceLock::new();
    COMMAND
        .get_or_init(|| {
            for candidate in ["powershell.exe", "powershell", "pwsh.exe", "pwsh"] {
                let status = Command::new(candidate)
                    .args([
                        "-NoLogo",
                        "-NoProfile",
                        "-NonInteractive",
                        "-Command",
                        "$PSVersionTable.PSVersion.Major",
                    ])
                    .status();
                if matches!(status, Ok(result) if result.success()) {
                    return Some(OsString::from(candidate));
                }
            }
            None
        })
        .as_ref()
}

fn read_native_cpu_stat() -> Option<RawCpuStat> {
    let mut idle = FILETIME::default();
    let mut kernel = FILETIME::default();
    let mut user = FILETIME::default();
    unsafe {
        GetSystemTimes(Some(&mut idle), Some(&mut kernel), Some(&mut user)).ok()?;
    }

    let idle_ticks = filetime_to_u64(idle);
    let kernel_ticks = filetime_to_u64(kernel);
    let user_ticks = filetime_to_u64(user);

    Some(RawCpuStat {
        user: user_ticks,
        system: kernel_ticks.saturating_sub(idle_ticks),
        idle: idle_ticks,
        ..RawCpuStat::default()
    })
}

fn read_process_native_metrics(pid: u32) -> Option<ProcessNativeMetrics> {
    let handle = ProcessHandle::open(pid)?;

    let mut creation = FILETIME::default();
    let mut exit = FILETIME::default();
    let mut kernel = FILETIME::default();
    let mut user = FILETIME::default();
    unsafe {
        GetProcessTimes(handle.0, &mut creation, &mut exit, &mut kernel, &mut user).ok()?;
    }

    let mut io = IO_COUNTERS::default();
    unsafe {
        GetProcessIoCounters(handle.0, &mut io).ok()?;
    }

    let mut handle_count = 0u32;
    unsafe {
        GetProcessHandleCount(handle.0, &mut handle_count).ok()?;
    }

    Some(ProcessNativeMetrics {
        utime: filetime_to_u64(user),
        stime: filetime_to_u64(kernel),
        fd_count: handle_count,
        io_read_bytes: io.ReadTransferCount,
        io_write_bytes: io.WriteTransferCount,
    })
}

fn read_native_network_stats() -> Option<Vec<RawNetStat>> {
    let mut table = ptr::null_mut::<MIB_IF_TABLE2>();
    let status = unsafe { GetIfTable2(&mut table) };
    if status != NO_ERROR || table.is_null() {
        return None;
    }

    let stats = unsafe {
        let entries = slice::from_raw_parts((*table).Table.as_ptr(), (*table).NumEntries as usize);
        entries
            .iter()
            .map(|row| RawNetStat {
                interface: interface_name(row),
                rx_bytes: row.InOctets,
                rx_packets: row.InUcastPkts.saturating_add(row.InNUcastPkts),
                rx_errors: row.InErrors,
                rx_dropped: row.InDiscards,
                tx_bytes: row.OutOctets,
                tx_packets: row.OutUcastPkts.saturating_add(row.OutNUcastPkts),
                tx_errors: row.OutErrors,
                tx_dropped: row.OutDiscards,
            })
            .filter(|row| !row.interface.is_empty())
            .collect::<Vec<_>>()
    };

    unsafe {
        FreeMibTable(table.cast());
    }

    if stats.is_empty() {
        None
    } else {
        Some(stats)
    }
}

fn read_native_net_connections() -> Option<RawNetConnections> {
    let tcp_v4 = read_tcp_rows_v4()?;
    let tcp_v6 = read_tcp_rows_v6().unwrap_or_default();

    let mut snapshot = RawNetConnections::default();
    for state in tcp_v4.iter().copied() {
        apply_tcp_state(&mut snapshot, state);
    }
    for state in tcp_v6.iter().copied() {
        apply_tcp_state(&mut snapshot, state);
    }
    snapshot.total = tcp_v4.len().saturating_add(tcp_v6.len()) as u32;

    let udp_v4 = read_udp_datagrams(AF_INET.0 as u32).unwrap_or(0);
    let udp_v6 = read_udp_datagrams(AF_INET6.0 as u32).unwrap_or(0);
    snapshot.udp_total = udp_v4.saturating_add(udp_v6);

    let retrans_v4 = read_tcp_retransmissions(AF_INET.0 as u32).unwrap_or(0);
    let retrans_v6 = read_tcp_retransmissions(AF_INET6.0 as u32).unwrap_or(0);
    snapshot.retrans_segs = retrans_v4.saturating_add(retrans_v6);

    Some(snapshot)
}

fn read_tcp_rows_v4() -> Option<Vec<u32>> {
    let mut size = 0u32;
    let status = unsafe { GetTcpTable2(None, &mut size, true) };
    if status != ERROR_INSUFFICIENT_BUFFER.0 || size == 0 {
        return None;
    }

    let mut buffer = vec![0u8; size as usize];
    let table = buffer.as_mut_ptr().cast::<MIB_TCPTABLE2>();
    let status = unsafe { GetTcpTable2(Some(table), &mut size, true) };
    if status != NO_ERROR.0 {
        return None;
    }

    Some(unsafe {
        slice::from_raw_parts((*table).table.as_ptr(), (*table).dwNumEntries as usize)
            .iter()
            .map(|row: &MIB_TCPROW2| row.dwState)
            .collect()
    })
}

fn read_tcp_rows_v6() -> Option<Vec<u32>> {
    let mut size = 0u32;
    let status = unsafe { GetTcp6Table2(ptr::null_mut(), &mut size, true) };
    if status != ERROR_INSUFFICIENT_BUFFER.0 || size == 0 {
        return None;
    }

    let mut buffer = vec![0u8; size as usize];
    let table = buffer.as_mut_ptr().cast::<MIB_TCP6TABLE2>();
    let status = unsafe { GetTcp6Table2(table, &mut size, true) };
    if status != NO_ERROR.0 {
        return None;
    }

    Some(unsafe {
        slice::from_raw_parts((*table).table.as_ptr(), (*table).dwNumEntries as usize)
            .iter()
            .map(|row: &MIB_TCP6ROW2| row.State)
            .collect()
    })
}

fn read_udp_datagrams(family: u32) -> Option<u32> {
    let mut stats = MIB_UDPSTATS::default();
    let status = unsafe { GetUdpStatisticsEx(&mut stats, family) };
    if status == NO_ERROR.0 {
        Some(stats.dwInDatagrams.saturating_add(stats.dwOutDatagrams))
    } else {
        None
    }
}

fn read_tcp_retransmissions(family: u32) -> Option<u64> {
    let mut stats = MIB_TCPSTATS2::default();
    let status = unsafe { GetTcpStatisticsEx2(&mut stats, family) };
    if status == NO_ERROR.0 {
        Some(stats.dwRetransSegs as u64)
    } else {
        None
    }
}

fn apply_tcp_state(snapshot: &mut RawNetConnections, state: u32) {
    match state as i32 {
        value if value == MIB_TCP_STATE_ESTAB.0 => snapshot.established += 1,
        value if value == MIB_TCP_STATE_SYN_SENT.0 => snapshot.tcp_syn_sent += 1,
        value if value == MIB_TCP_STATE_SYN_RCVD.0 => snapshot.tcp_syn_recv += 1,
        value if value == MIB_TCP_STATE_FIN_WAIT1.0 => snapshot.tcp_fin_wait1 += 1,
        value if value == MIB_TCP_STATE_FIN_WAIT2.0 => snapshot.tcp_fin_wait2 += 1,
        value if value == MIB_TCP_STATE_TIME_WAIT.0 => snapshot.tcp_time_wait += 1,
        value if value == MIB_TCP_STATE_CLOSED.0 => snapshot.tcp_close += 1,
        value if value == MIB_TCP_STATE_CLOSE_WAIT.0 => snapshot.tcp_close_wait += 1,
        value if value == MIB_TCP_STATE_LAST_ACK.0 => snapshot.tcp_last_ack += 1,
        value if value == MIB_TCP_STATE_LISTEN.0 => snapshot.tcp_listen += 1,
        value if value == MIB_TCP_STATE_CLOSING.0 => snapshot.tcp_closing += 1,
        _ => snapshot.tcp_other += 1,
    }
}

fn interface_name(row: &MIB_IF_ROW2) -> String {
    let alias = utf16z_to_string(&row.Alias);
    if !alias.is_empty() {
        return alias;
    }

    let description = utf16z_to_string(&row.Description);
    if !description.is_empty() {
        return description;
    }

    format!("if{}", row.InterfaceIndex)
}

fn utf16z_to_string(buffer: &[u16]) -> String {
    let len = buffer
        .iter()
        .position(|&value| value == 0)
        .unwrap_or(buffer.len());
    String::from_utf16_lossy(&buffer[..len]).trim().to_string()
}

fn filetime_to_u64(value: FILETIME) -> u64 {
    ((value.dwHighDateTime as u64) << 32) | value.dwLowDateTime as u64
}

fn format_version_and_build(version: &str, build: &str) -> String {
    match (version.trim(), build.trim()) {
        ("", "") => String::new(),
        (version, "") => version.to_string(),
        ("", build) => build.to_string(),
        (version, build) => format!("{} (build {})", version, build),
    }
}

struct ProcessHandle(HANDLE);

impl ProcessHandle {
    fn open(pid: u32) -> Option<Self> {
        unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok() }
            .map(ProcessHandle)
    }
}

impl Drop for ProcessHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

fn json_values(value: Value) -> Vec<Value> {
    match value {
        Value::Array(values) => values,
        Value::Null => Vec::new(),
        other => vec![other],
    }
}

fn json_field_u64(value: &Value, field: &str) -> Option<u64> {
    match value.get(field)? {
        Value::Number(number) => number.as_u64(),
        Value::String(text) => text.parse().ok(),
        _ => None,
    }
}

fn json_field_f64(value: &Value, field: &str) -> Option<f64> {
    match value.get(field)? {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.parse().ok(),
        _ => None,
    }
}

fn json_field_string(value: &Value, field: &str) -> Option<String> {
    match value.get(field)? {
        Value::String(text) => Some(text.clone()),
        Value::Null => None,
        other => Some(other.to_string().trim_matches('"').to_string()),
    }
}
