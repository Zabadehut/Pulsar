use crate::collectors::{AlertLevel, LogEntry};
use chrono::Utc;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, Default)]
pub struct FileTailState {
    pub offset: u64,
    pub file_size: u64,
    pub modified_ts: i64,
}

#[derive(Debug, Clone, Default)]
pub struct TailRefresh {
    pub entries: Vec<LogEntry>,
    pub active_files: Vec<String>,
    pub rotated_files: usize,
}

pub fn read_system_events(window_secs: u64, max_entries: usize) -> Vec<LogEntry> {
    #[cfg(target_os = "linux")]
    {
        return read_linux_system_events(window_secs, max_entries);
    }
    #[cfg(target_os = "macos")]
    {
        return read_macos_system_events(window_secs, max_entries);
    }
    #[cfg(target_os = "windows")]
    {
        return read_windows_system_events(window_secs, max_entries);
    }
    #[allow(unreachable_code)]
    Vec::new()
}

pub fn refresh_tailed_paths(
    patterns: &[String],
    states: &mut HashMap<String, FileTailState>,
    recent_secs: u64,
    max_files: usize,
    max_lines_per_file: usize,
) -> TailRefresh {
    let now = SystemTime::now();
    let recent_threshold = now
        .checked_sub(Duration::from_secs(recent_secs))
        .unwrap_or(SystemTime::UNIX_EPOCH);
    let mut files = Vec::new();

    for pattern in patterns {
        files.extend(expand_pattern(pattern));
    }

    files.sort();
    files.dedup();

    let mut recent_files = files
        .into_iter()
        .filter_map(|path| {
            let metadata = fs::metadata(&path).ok()?;
            let modified = metadata.modified().ok()?;
            if modified < recent_threshold || !metadata.is_file() {
                return None;
            }
            let file_size = metadata.len();
            let modified_ts = system_time_to_timestamp(modified);
            let previous = states.get(path.to_string_lossy().as_ref());
            let active = previous
                .map(|state| state.file_size != file_size || state.modified_ts != modified_ts)
                .unwrap_or(true);
            Some((active, modified, file_size, modified_ts, path))
        })
        .collect::<Vec<_>>();

    recent_files.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.cmp(&a.1)));

    let mut refresh = TailRefresh::default();
    let selected = recent_files.into_iter().take(max_files).collect::<Vec<_>>();
    let selected_keys = selected
        .iter()
        .map(|(_, _, _, _, path)| path.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    states.retain(|path, _| selected_keys.iter().any(|selected| selected == path));

    for (active, _, file_size, modified_ts, path) in selected {
        let path_key = path.to_string_lossy().to_string();
        let previous = states.get(&path_key).cloned().unwrap_or_default();
        let (lines, state, rotated) =
            read_incremental_lines(&path, &previous, max_lines_per_file, file_size, modified_ts);

        if active {
            refresh.active_files.push(path_key.clone());
        }
        if rotated {
            refresh.rotated_files += 1;
        }
        states.insert(path_key.clone(), state);

        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            refresh
                .entries
                .push(parse_file_log_line(trimmed, &path_key, modified_ts));
        }
    }

    refresh.entries.sort_by(|a, b| {
        severity_rank(&b.level)
            .cmp(&severity_rank(&a.level))
            .then_with(|| b.timestamp.cmp(&a.timestamp))
            .then_with(|| a.origin.cmp(&b.origin))
    });
    refresh
}

fn system_time_to_timestamp(value: SystemTime) -> i64 {
    value
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or_else(|_| Utc::now().timestamp())
}

fn tail_lines(path: &Path, max_lines: usize) -> Vec<String> {
    let Ok(content) = fs::read_to_string(path) else {
        return Vec::new();
    };

    let mut lines = content.lines().map(str::to_string).collect::<Vec<_>>();
    if lines.len() > max_lines {
        lines = lines.split_off(lines.len() - max_lines);
    }
    lines
}

fn read_incremental_lines(
    path: &Path,
    previous: &FileTailState,
    max_lines: usize,
    file_size: u64,
    modified_ts: i64,
) -> (Vec<String>, FileTailState, bool) {
    if previous.offset == 0 || previous.file_size == 0 {
        return (
            tail_lines(path, max_lines),
            FileTailState {
                offset: file_size,
                file_size,
                modified_ts,
            },
            false,
        );
    }

    if file_size < previous.offset {
        return (
            tail_lines(path, max_lines),
            FileTailState {
                offset: file_size,
                file_size,
                modified_ts,
            },
            true,
        );
    }

    if file_size == previous.offset && previous.modified_ts == modified_ts {
        return (
            Vec::new(),
            FileTailState {
                offset: file_size,
                file_size,
                modified_ts,
            },
            false,
        );
    }

    let Ok(file) = File::open(path) else {
        return (
            Vec::new(),
            FileTailState {
                offset: previous.offset,
                file_size,
                modified_ts,
            },
            false,
        );
    };
    let mut reader = BufReader::new(file);
    if reader.seek(SeekFrom::Start(previous.offset)).is_err() {
        return (
            tail_lines(path, max_lines),
            FileTailState {
                offset: file_size,
                file_size,
                modified_ts,
            },
            true,
        );
    }

    let mut lines = Vec::new();
    let mut buffer = String::new();
    loop {
        buffer.clear();
        let Ok(bytes_read) = reader.read_line(&mut buffer) else {
            break;
        };
        if bytes_read == 0 {
            break;
        }
        lines.push(buffer.trim_end_matches(['\n', '\r']).to_string());
    }
    if lines.len() > max_lines {
        lines = lines.split_off(lines.len() - max_lines);
    }

    (
        lines,
        FileTailState {
            offset: file_size,
            file_size,
            modified_ts,
        },
        false,
    )
}

fn expand_pattern(pattern: &str) -> Vec<PathBuf> {
    let path = Path::new(pattern);
    if !pattern.contains('*') && !pattern.contains('?') {
        return expand_plain_path(path);
    }

    let root = wildcard_root(pattern);
    let mut candidates = Vec::new();
    walk_paths(&root, &mut candidates);
    candidates
        .into_iter()
        .filter(|candidate| wildcard_match_path(pattern, candidate))
        .collect()
}

fn expand_plain_path(path: &Path) -> Vec<PathBuf> {
    if path.is_file() {
        return vec![path.to_path_buf()];
    }
    if path.is_dir() {
        let mut entries = Vec::new();
        walk_paths(path, &mut entries);
        return entries;
    }
    Vec::new()
}

fn wildcard_root(pattern: &str) -> PathBuf {
    let wildcard_index = pattern
        .find(|ch| ['*', '?'].contains(&ch))
        .unwrap_or(pattern.len());
    let prefix = &pattern[..wildcard_index];
    let path = Path::new(prefix);
    path.ancestors()
        .find(|ancestor| ancestor.exists())
        .unwrap_or_else(|| Path::new("/"))
        .to_path_buf()
}

fn walk_paths(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(read_dir) = fs::read_dir(root) else {
        return;
    };

    for entry in read_dir.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_paths(&path, out);
        } else {
            out.push(path);
        }
    }
}

fn wildcard_match_path(pattern: &str, path: &Path) -> bool {
    wildcard_match(pattern.as_bytes(), path.to_string_lossy().as_bytes())
}

fn wildcard_match(pattern: &[u8], text: &[u8]) -> bool {
    let mut p = 0usize;
    let mut t = 0usize;
    let mut star = None;
    let mut match_index = 0usize;

    while t < text.len() {
        if p < pattern.len() && (pattern[p] == text[t] || pattern[p] == b'?') {
            p += 1;
            t += 1;
        } else if p < pattern.len() && pattern[p] == b'*' {
            star = Some(p);
            p += 1;
            match_index = t;
        } else if let Some(star_index) = star {
            p = star_index + 1;
            match_index += 1;
            t = match_index;
        } else {
            return false;
        }
    }

    while p < pattern.len() && pattern[p] == b'*' {
        p += 1;
    }

    p == pattern.len()
}

fn infer_level(message: &str) -> AlertLevel {
    let lowered = message.to_ascii_lowercase();
    if lowered.contains("fatal")
        || lowered.contains("panic")
        || lowered.contains("error")
        || lowered.contains("failed")
        || lowered.contains("critical")
    {
        AlertLevel::Critical
    } else if lowered.contains("warn") || lowered.contains("timeout") || lowered.contains("drop") {
        AlertLevel::Warning
    } else {
        AlertLevel::Info
    }
}

fn parse_file_log_line(message: &str, path: &str, timestamp: i64) -> LogEntry {
    if let Some(entry) = parse_json_log_line(message, path, timestamp) {
        return entry;
    }
    if let Some(entry) = parse_nginx_log_line(message, path, timestamp) {
        return entry;
    }

    LogEntry {
        timestamp,
        level: infer_level(message),
        source: infer_source(path, message),
        origin: path.to_string(),
        message: message.to_string(),
    }
}

fn parse_json_log_line(message: &str, path: &str, timestamp: i64) -> Option<LogEntry> {
    let value = serde_json::from_str::<Value>(message).ok()?;
    let text = json_string(&value, "message")
        .or_else(|| json_string(&value, "msg"))
        .or_else(|| json_string(&value, "event"))
        .or_else(|| json_string(&value, "log"))?;
    let level = json_string(&value, "level")
        .or_else(|| json_string(&value, "severity"))
        .or_else(|| json_string(&value, "lvl"))
        .unwrap_or_default();
    let source = json_string(&value, "logger")
        .or_else(|| json_string(&value, "component"))
        .or_else(|| json_string(&value, "service"))
        .or_else(|| json_string(&value, "target"))
        .unwrap_or_else(|| infer_source(path, &text));

    Some(LogEntry {
        timestamp,
        level: infer_level(&level),
        source,
        origin: path.to_string(),
        message: text,
    })
}

fn parse_nginx_log_line(message: &str, path: &str, timestamp: i64) -> Option<LogEntry> {
    if !(message.contains("nginx") || path.contains("nginx") || message.contains('[')) {
        return None;
    }
    let level = if message.contains("[error]") || message.contains("[crit]") {
        AlertLevel::Critical
    } else if message.contains("[warn]") {
        AlertLevel::Warning
    } else {
        infer_level(message)
    };

    Some(LogEntry {
        timestamp,
        level,
        source: "nginx".to_string(),
        origin: path.to_string(),
        message: message.to_string(),
    })
}

fn infer_source(path: &str, message: &str) -> String {
    let lowered_path = path.to_ascii_lowercase();
    let lowered_message = message.to_ascii_lowercase();
    if lowered_path.contains("nginx") || lowered_message.contains("nginx") {
        "nginx".to_string()
    } else if lowered_path.contains("jvm")
        || lowered_path.contains("java")
        || lowered_message.contains("exception")
        || lowered_message.contains("stacktrace")
    {
        "jvm".to_string()
    } else if lowered_path.contains("syslog") {
        "syslog".to_string()
    } else {
        Path::new(path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("file")
            .to_string()
    }
}

fn severity_rank(level: &AlertLevel) -> usize {
    match level {
        AlertLevel::Critical => 3,
        AlertLevel::Warning => 2,
        AlertLevel::Info => 1,
    }
}

fn command_output(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(target_os = "linux")]
fn read_linux_system_events(window_secs: u64, max_entries: usize) -> Vec<LogEntry> {
    let since = format!("-{} seconds", window_secs);
    let Some(output) = command_output(
        "journalctl",
        &[
            "--since",
            &since,
            "-p",
            "info..emerg",
            "-o",
            "json",
            "--no-pager",
            "-n",
            &max_entries.to_string(),
        ],
    ) else {
        return Vec::new();
    };

    output
        .lines()
        .filter_map(|line| {
            let value: Value = serde_json::from_str(line).ok()?;
            let message = json_string(&value, "MESSAGE")?;
            Some(LogEntry {
                timestamp: json_string(&value, "__REALTIME_TIMESTAMP")
                    .and_then(|micros| micros.parse::<i64>().ok())
                    .map(|micros| micros / 1_000_000)
                    .unwrap_or_else(|| Utc::now().timestamp()),
                level: priority_to_level(
                    json_string(&value, "PRIORITY")
                        .and_then(|priority| priority.parse::<u8>().ok()),
                ),
                source: linux_source(&value),
                origin: linux_origin(&value),
                message,
            })
        })
        .collect()
}

#[cfg(target_os = "macos")]
fn read_macos_system_events(window_secs: u64, max_entries: usize) -> Vec<LogEntry> {
    if let Some(events) = read_macos_json_events(window_secs, max_entries) {
        return events;
    }

    let hours = ((window_secs as f64) / 3600.0).max(1.0);
    let last = format!("{hours:.1}h");
    let Some(output) = command_output(
        "log",
        &["show", "--last", &last, "--style", "compact", "--info"],
    ) else {
        return Vec::new();
    };

    output
        .lines()
        .rev()
        .take(max_entries)
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return None;
            }
            let source = trimmed
                .split_whitespace()
                .nth(4)
                .unwrap_or("log")
                .to_string();
            Some(LogEntry {
                timestamp: Utc::now().timestamp(),
                level: infer_level(trimmed),
                source,
                origin: "log show compact".to_string(),
                message: trimmed.to_string(),
            })
        })
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

#[cfg(target_os = "windows")]
fn read_windows_system_events(window_secs: u64, max_entries: usize) -> Vec<LogEntry> {
    let hours = ((window_secs as f64) / 3600.0).max(1.0);
    let command = format!(
        "Get-WinEvent -FilterHashtable @{{LogName='Application','System'; StartTime=(Get-Date).AddHours(-{hours})}} -MaxEvents {max_entries} | Select-Object TimeCreated,LevelDisplayName,ProviderName,LogName,Id,MachineName,Message | ConvertTo-Json -Compress"
    );
    let Some(output) = command_output("powershell", &["-NoProfile", "-Command", &command]) else {
        return Vec::new();
    };

    let Ok(value) = serde_json::from_str::<Value>(&output) else {
        return Vec::new();
    };
    let items = match value {
        Value::Array(items) => items,
        other => vec![other],
    };

    items
        .into_iter()
        .filter_map(|item| {
            let message = json_string(&item, "Message")?;
            let log_name = json_string(&item, "LogName").unwrap_or_else(|| "eventlog".into());
            let provider = json_string(&item, "ProviderName").unwrap_or_else(|| "eventlog".into());
            let event_id = json_string(&item, "Id").unwrap_or_default();
            Some(LogEntry {
                timestamp: windows_timestamp(
                    json_string(&item, "TimeCreated")
                        .unwrap_or_default()
                        .as_str(),
                ),
                level: match json_string(&item, "LevelDisplayName")
                    .unwrap_or_default()
                    .to_ascii_lowercase()
                    .as_str()
                {
                    "error" | "critical" => AlertLevel::Critical,
                    "warning" => AlertLevel::Warning,
                    _ => AlertLevel::Info,
                },
                source: provider,
                origin: if event_id.is_empty() {
                    format!("Get-WinEvent/{log_name}")
                } else {
                    format!("Get-WinEvent/{log_name}/event-{event_id}")
                },
                message,
            })
        })
        .collect()
}

#[cfg(target_os = "macos")]
fn read_macos_json_events(window_secs: u64, max_entries: usize) -> Option<Vec<LogEntry>> {
    let hours = ((window_secs as f64) / 3600.0).max(1.0);
    let last = format!("{hours:.1}h");
    let output = command_output(
        "log",
        &["show", "--last", &last, "--style", "json", "--info"],
    )?;
    let value = serde_json::from_str::<Value>(&output).ok()?;
    let items = match value {
        Value::Array(items) => items,
        other => vec![other],
    };

    Some(
        items
            .into_iter()
            .rev()
            .take(max_entries)
            .filter_map(|item| {
                let message =
                    json_string(&item, "eventMessage").or_else(|| json_string(&item, "message"))?;
                let subsystem = json_string(&item, "subsystem").unwrap_or_default();
                let category = json_string(&item, "category").unwrap_or_default();
                let process = json_string(&item, "processImagePath")
                    .or_else(|| json_string(&item, "process"))
                    .unwrap_or_else(|| "log".to_string());
                Some(LogEntry {
                    timestamp: macos_timestamp(
                        json_string(&item, "timestamp").unwrap_or_default().as_str(),
                    ),
                    level: macos_level(
                        json_string(&item, "messageType")
                            .unwrap_or_default()
                            .as_str(),
                    ),
                    source: process,
                    origin: if subsystem.is_empty() && category.is_empty() {
                        "log show json".to_string()
                    } else if category.is_empty() {
                        format!("log show json/{subsystem}")
                    } else {
                        format!("log show json/{subsystem}/{category}")
                    },
                    message,
                })
            })
            .collect(),
    )
}

fn priority_to_level(priority: Option<u8>) -> AlertLevel {
    match priority.unwrap_or(6) {
        0..=3 => AlertLevel::Critical,
        4 => AlertLevel::Warning,
        _ => AlertLevel::Info,
    }
}

#[cfg(target_os = "linux")]
fn linux_source(value: &Value) -> String {
    json_string(value, "SYSLOG_IDENTIFIER")
        .or_else(|| json_string(value, "_COMM"))
        .or_else(|| json_string(value, "_SYSTEMD_UNIT"))
        .unwrap_or_else(|| "journal".to_string())
}

#[cfg(target_os = "linux")]
fn linux_origin(value: &Value) -> String {
    if let Some(unit) = json_string(value, "_SYSTEMD_UNIT") {
        return format!("journalctl/{unit}");
    }
    if let Some(transport) = json_string(value, "_TRANSPORT") {
        return format!("journalctl/{transport}");
    }
    "journalctl".to_string()
}

#[cfg(target_os = "macos")]
fn macos_level(value: &str) -> AlertLevel {
    match value.to_ascii_lowercase().as_str() {
        "error" | "fault" => AlertLevel::Critical,
        "default" | "info" => AlertLevel::Info,
        _ => infer_level(value),
    }
}

#[cfg(target_os = "macos")]
fn macos_timestamp(value: &str) -> i64 {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|timestamp| timestamp.timestamp())
        .unwrap_or_else(|_| Utc::now().timestamp())
}

#[cfg(target_os = "windows")]
fn windows_timestamp(value: &str) -> i64 {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|timestamp| timestamp.timestamp())
        .unwrap_or_else(|_| Utc::now().timestamp())
}

fn json_string(value: &Value, key: &str) -> Option<String> {
    match value.get(key) {
        Some(Value::String(value)) => Some(value.clone()),
        Some(Value::Number(value)) => Some(value.to_string()),
        Some(Value::Array(items)) => items.first().and_then(Value::as_str).map(str::to_string),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn wildcard_match_supports_star_and_question_mark() {
        assert!(wildcard_match(b"/var/log/*.log", b"/var/log/sys.log"));
        assert!(wildcard_match(b"/tmp/app-?.txt", b"/tmp/app-1.txt"));
        assert!(!wildcard_match(b"/tmp/app-?.txt", b"/tmp/app-10.txt"));
    }

    #[test]
    fn refresh_tailed_paths_reads_only_new_lines() {
        let dir = std::env::temp_dir().join(format!("pulsar-log-test-{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        let file = dir.join("app.log");
        fs::write(&file, "first\n").unwrap();

        let mut states = HashMap::new();
        let first = refresh_tailed_paths(
            &[file.to_string_lossy().to_string()],
            &mut states,
            3600,
            4,
            20,
        );
        assert_eq!(first.entries.len(), 1);

        let mut handle = File::options().append(true).open(&file).unwrap();
        writeln!(handle, "second").unwrap();
        writeln!(handle, "third").unwrap();

        let second = refresh_tailed_paths(
            &[file.to_string_lossy().to_string()],
            &mut states,
            3600,
            4,
            20,
        );
        let messages = second
            .entries
            .iter()
            .map(|entry| entry.message.as_str())
            .collect::<Vec<_>>();
        assert!(messages.contains(&"second"));
        assert!(messages.contains(&"third"));
        assert!(!messages.contains(&"first"));
    }

    #[test]
    fn refresh_tailed_paths_detects_truncation() {
        let dir = std::env::temp_dir().join(format!("pulsar-log-trunc-{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        let file = dir.join("rotate.log");
        fs::write(&file, "before\n").unwrap();

        let mut states = HashMap::new();
        let _ = refresh_tailed_paths(
            &[file.to_string_lossy().to_string()],
            &mut states,
            3600,
            4,
            20,
        );

        fs::write(&file, "after\n").unwrap();
        let refresh = refresh_tailed_paths(
            &[file.to_string_lossy().to_string()],
            &mut states,
            3600,
            4,
            20,
        );
        assert_eq!(refresh.rotated_files, 1);
        assert!(refresh.entries.iter().any(|entry| entry.message == "after"));
    }
}
