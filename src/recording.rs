use crate::collectors::Snapshot;
use crate::config::RecordConfig;
use anyhow::Result;
use chrono::{DateTime, Datelike, Timelike, Utc};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotationPolicy {
    Never,
    Hourly,
    Daily,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionMode {
    None,
    Zip,
}

impl CompressionMode {
    pub fn parse(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "zip" => Self::Zip,
            _ => Self::None,
        }
    }
}

impl RotationPolicy {
    pub fn parse(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "hour" | "hourly" => Self::Hourly,
            "day" | "daily" => Self::Daily,
            _ => Self::Never,
        }
    }

    fn bucket_key(self, now: DateTime<Utc>) -> Option<String> {
        match self {
            Self::Never => None,
            Self::Hourly => Some(format!(
                "{:04}{:02}{:02}{:02}",
                now.year(),
                now.month(),
                now.day(),
                now.hour()
            )),
            Self::Daily => Some(format!(
                "{:04}{:02}{:02}",
                now.year(),
                now.month(),
                now.day()
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RecordRuntimeOptions {
    pub interval_secs: u64,
    pub output: PathBuf,
    pub rotate: RotationPolicy,
    pub max_file_size_bytes: Option<u64>,
    pub keep_files: Option<usize>,
    pub compress: CompressionMode,
}

impl RecordRuntimeOptions {
    pub fn from_sources(
        defaults: &RecordConfig,
        interval: Option<&str>,
        output: Option<PathBuf>,
        rotate: Option<&str>,
        max_file_size_mb: Option<u64>,
        keep_files: Option<usize>,
        compress: Option<&str>,
    ) -> Self {
        Self {
            interval_secs: parse_interval(interval).unwrap_or(defaults.interval_secs),
            output: output.unwrap_or_else(|| PathBuf::from(&defaults.output)),
            rotate: rotate
                .map(RotationPolicy::parse)
                .unwrap_or_else(|| RotationPolicy::parse(&defaults.rotate)),
            max_file_size_bytes: max_file_size_mb
                .or(defaults.max_file_size_mb)
                .filter(|value| *value > 0)
                .map(|value| value * 1024 * 1024),
            keep_files: keep_files
                .or(defaults.keep_files)
                .filter(|value| *value > 0),
            compress: compress
                .map(CompressionMode::parse)
                .unwrap_or_else(|| CompressionMode::parse(&defaults.compress)),
        }
    }
}

#[derive(Debug)]
struct ActiveFile {
    path: PathBuf,
    file: File,
    bucket_key: Option<String>,
    bytes_written: u64,
}

pub struct Recorder {
    output: PathBuf,
    rotate: RotationPolicy,
    max_file_size_bytes: Option<u64>,
    keep_files: Option<usize>,
    compress: CompressionMode,
    active: Option<ActiveFile>,
}

impl Recorder {
    pub fn new(options: RecordRuntimeOptions) -> Result<Self> {
        fs::create_dir_all(&options.output)?;
        Ok(Self {
            output: options.output,
            rotate: options.rotate,
            max_file_size_bytes: options.max_file_size_bytes,
            keep_files: options.keep_files,
            compress: options.compress,
            active: None,
        })
    }

    pub fn write_snapshot(&mut self, snapshot: &Snapshot) -> Result<PathBuf> {
        let now = Utc::now();
        self.ensure_active(now)?;
        let active = self.active.as_mut().expect("active file should exist");
        let line = serde_json::to_string(snapshot)?;
        writeln!(active.file, "{line}")?;
        active.bytes_written += line.len() as u64 + 1;
        Ok(active.path.clone())
    }

    fn ensure_active(&mut self, now: DateTime<Utc>) -> Result<()> {
        if self
            .active
            .as_ref()
            .is_none_or(|active| self.should_rotate(active, now))
        {
            self.rotate_file(now)?;
        }
        Ok(())
    }

    fn should_rotate(&self, active: &ActiveFile, now: DateTime<Utc>) -> bool {
        if self.rotate.bucket_key(now) != active.bucket_key {
            return true;
        }

        self.max_file_size_bytes
            .is_some_and(|limit| active.bytes_written >= limit)
    }

    fn rotate_file(&mut self, now: DateTime<Utc>) -> Result<()> {
        let previous = self.active.take();
        let path = self
            .output
            .join(format!("pulsar_{}.jsonl", now.format("%Y%m%d_%H%M%S_%3f")));
        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        self.active = Some(ActiveFile {
            path: path.clone(),
            file,
            bucket_key: self.rotate.bucket_key(now),
            bytes_written: 0,
        });
        if let Some(previous) = previous {
            self.close_segment(previous.path);
        }
        self.prune_old_files()?;
        tracing::info!("Writing to {:?}", path);
        Ok(())
    }

    fn close_segment(&self, path: PathBuf) {
        if self.compress == CompressionMode::Zip {
            if let Err(error) = compress_segment_to_zip(&path) {
                tracing::warn!("Failed to compress {:?}: {}", path, error);
            }
        }
    }

    fn prune_old_files(&self) -> Result<()> {
        let Some(keep_files) = self.keep_files else {
            return Ok(());
        };

        let active_path = self.active.as_ref().map(|active| active.path.clone());
        let mut segments = list_segments(&self.output)?;
        segments.sort();

        let active_count = usize::from(active_path.is_some());
        segments.retain(|path| active_path.as_ref() != Some(path));

        let removable = segments
            .len()
            .saturating_sub(keep_files.saturating_sub(active_count));
        for candidate in segments.into_iter().take(removable) {
            fs::remove_file(candidate)?;
        }

        Ok(())
    }
}

fn list_segments(output: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(output)? {
        let entry = entry?;
        let path = entry.path();
        if path
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|name| {
                name.starts_with("pulsar_")
                    && (name.ends_with(".jsonl") || name.ends_with(".jsonl.zip"))
            })
        {
            files.push(path);
        }
    }
    Ok(files)
}

fn compress_segment_to_zip(path: &Path) -> Result<PathBuf> {
    let zip_path = PathBuf::from(format!("{}.zip", path.display()));
    let mut source = File::open(path)?;
    let target = File::create(&zip_path)?;
    let mut writer = ZipWriter::new(target);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("segment.jsonl");
    writer.start_file(name, options)?;
    let mut buffer = Vec::new();
    source.read_to_end(&mut buffer)?;
    writer.write_all(&buffer)?;
    writer.finish()?;
    fs::remove_file(path)?;
    Ok(zip_path)
}

fn parse_interval(value: Option<&str>) -> Option<u64> {
    let value = value?;
    if let Some(n) = value.strip_suffix('s') {
        n.parse().ok()
    } else if let Some(n) = value.strip_suffix('m') {
        n.parse::<u64>().ok().map(|seconds| seconds * 60)
    } else {
        value.parse().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn rotation_policy_parses_expected_values() {
        assert_eq!(RotationPolicy::parse("never"), RotationPolicy::Never);
        assert_eq!(RotationPolicy::parse("hourly"), RotationPolicy::Hourly);
        assert_eq!(RotationPolicy::parse("daily"), RotationPolicy::Daily);
    }

    #[test]
    fn runtime_options_merge_cli_and_config_values() {
        let defaults = RecordConfig {
            interval_secs: 5,
            output: "./captures".to_string(),
            rotate: "daily".to_string(),
            max_file_size_mb: Some(128),
            keep_files: Some(7),
            compress: "zip".to_string(),
        };

        let options = RecordRuntimeOptions::from_sources(
            &defaults,
            Some("10s"),
            None,
            Some("hourly"),
            None,
            Some(3),
            None,
        );

        assert_eq!(options.interval_secs, 10);
        assert_eq!(options.output, PathBuf::from("./captures"));
        assert_eq!(options.rotate, RotationPolicy::Hourly);
        assert_eq!(options.max_file_size_bytes, Some(128 * 1024 * 1024));
        assert_eq!(options.keep_files, Some(3));
        assert_eq!(options.compress, CompressionMode::Zip);
    }

    #[test]
    fn recorder_prunes_old_raw_segments() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let output = std::env::temp_dir().join(format!("sysray-recorder-test-{suffix}"));
        fs::create_dir_all(&output).unwrap();

        let options = RecordRuntimeOptions {
            interval_secs: 5,
            output: output.clone(),
            rotate: RotationPolicy::Never,
            max_file_size_bytes: None,
            keep_files: Some(2),
            compress: CompressionMode::None,
        };

        let mut recorder = Recorder::new(options).unwrap();
        for index in 0..3 {
            let path = output.join(format!("pulsar_20260303_12000{index}_000.jsonl"));
            fs::write(path, "{}\n").unwrap();
        }
        recorder.rotate_file(Utc::now()).unwrap();

        let files = list_segments(&output).unwrap();
        assert!(files.len() <= 2);

        fs::remove_dir_all(output).unwrap();
    }

    #[test]
    fn compression_replaces_raw_segment_with_zip() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let output = std::env::temp_dir().join(format!("sysray-recorder-zip-test-{suffix}"));
        fs::create_dir_all(&output).unwrap();

        let raw = output.join("pulsar_20260303_120000_000.jsonl");
        fs::write(&raw, "{\"ok\":true}\n").unwrap();

        let zip_path = compress_segment_to_zip(&raw).unwrap();
        assert!(!raw.exists());
        assert!(zip_path.exists());

        fs::remove_dir_all(output).unwrap();
    }
}
