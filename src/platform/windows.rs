use crate::platform::api::{
    RawCpuReading, RawDiskInventory, RawDiskSpace, RawDiskStat, RawLinuxMetrics, RawMemoryInfo,
    RawNetConnections, RawNetStat, RawProcReading, RawSystemInfo,
};
use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::mem::size_of;
use std::process::Command;
use windows::Win32::Foundation::{CloseHandle, FILETIME, HANDLE};
use windows::Win32::System::SystemInformation::{
    GetNativeSystemInfo, GetTickCount64, GlobalMemoryStatusEx, MEMORYSTATUSEX, SYSTEM_INFO,
};
use windows::Win32::System::Threading::{
    GetProcessHandleCount, GetProcessIoCounters, GetProcessTimes, OpenProcess, IO_COUNTERS,
    PROCESS_QUERY_LIMITED_INFORMATION,
};

pub fn read_cpu() -> Result<RawCpuReading> {
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
            filesystem: json_field_string(&value, "FileSystem").unwrap_or_default(),
            label: json_field_string(&value, "Label").unwrap_or_default(),
            uuid: json_field_string(&value, "Uuid").unwrap_or_default(),
            part_uuid: json_field_string(&value, "PartUuid").unwrap_or_default(),
            model: json_field_string(&value, "Model").unwrap_or_default(),
            serial: json_field_string(&value, "Serial").unwrap_or_default(),
            transport: json_field_string(&value, "Transport").unwrap_or_default(),
            reference: json_field_string(&value, "Reference").unwrap_or_default(),
            mount_points,
            children: Vec::new(),
        });
    }

    let mut children_map: HashMap<String, Vec<String>> = HashMap::new();
    for item in &inventory {
        if let Some(parent) = &item.parent {
            children_map
                .entry(parent.clone())
                .or_default()
                .push(item.device.clone());
        }
    }
    for item in &mut inventory {
        item.children = children_map.get(&item.device).cloned().unwrap_or_default();
    }

    Ok(inventory)
}

pub fn read_net_connections() -> RawNetConnections {
    let total = powershell_number_u32("(Get-NetTCPConnection | Measure-Object).Count").unwrap_or(0);
    let established =
        powershell_number_u32("(Get-NetTCPConnection -State Established | Measure-Object).Count")
            .unwrap_or(0);
    RawNetConnections {
        total,
        established,
        ..RawNetConnections::default()
    }
}

pub fn read_network() -> Result<Vec<RawNetStat>> {
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
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", script])
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
