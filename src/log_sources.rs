use crate::collectors::{AlertLevel, LogEntry};
use chrono::Utc;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime};

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

pub fn read_tailed_paths(
    patterns: &[String],
    recent_secs: u64,
    max_files: usize,
    max_lines_per_file: usize,
) -> Vec<LogEntry> {
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
            Some((modified, path))
        })
        .collect::<Vec<_>>();

    recent_files.sort_by(|a, b| b.0.cmp(&a.0));

    let mut entries = Vec::new();
    for (_, path) in recent_files.into_iter().take(max_files) {
        let lines = tail_lines(&path, max_lines_per_file);
        let timestamp = file_timestamp(&path);
        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            entries.push(LogEntry {
                timestamp,
                level: infer_level(trimmed),
                source: "file".to_string(),
                origin: path.display().to_string(),
                message: trimmed.to_string(),
            });
        }
    }

    entries.sort_by(|a, b| {
        b.timestamp
            .cmp(&a.timestamp)
            .then_with(|| a.origin.cmp(&b.origin))
    });
    entries
}

fn file_timestamp(path: &Path) -> i64 {
    fs::metadata(path)
        .ok()
        .and_then(|metadata| metadata.modified().ok())
        .and_then(|modified| modified.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or_else(|| Utc::now().timestamp())
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

    #[test]
    fn wildcard_match_supports_star_and_question_mark() {
        assert!(wildcard_match(b"/var/log/*.log", b"/var/log/sys.log"));
        assert!(wildcard_match(b"/tmp/app-?.txt", b"/tmp/app-1.txt"));
        assert!(!wildcard_match(b"/tmp/app-?.txt", b"/tmp/app-10.txt"));
    }
}
