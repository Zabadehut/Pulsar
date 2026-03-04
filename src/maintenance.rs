use crate::cli::MaintenanceAction;
use crate::collectors::Snapshot;
use crate::config::Config;
use crate::engine::Scheduler;
use crate::{build_pipeline, build_registry};
use anyhow::{bail, Context, Result};
use chrono::Local;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use tokio_util::sync::CancellationToken;
use zip::write::SimpleFileOptions;
use zip::CompressionMethod;
use zip::ZipWriter;

pub async fn run(action: MaintenanceAction, config: &Config) -> Result<()> {
    match action {
        MaintenanceAction::DailySnapshot { output_dir } => {
            let output_dir = output_dir.unwrap_or_else(default_daily_dir);
            let snapshot = collect_single_snapshot(config).await?;
            append_daily_snapshot(&output_dir, &snapshot)?;
        }
        MaintenanceAction::Prune {
            directory,
            retention_days,
        } => {
            let directory = directory.unwrap_or_else(default_daily_dir);
            prune_daily_files(&directory, retention_days)?;
        }
        MaintenanceAction::Archive {
            source_dir,
            archive_dir,
            min_age_days,
            max_age_days,
        } => {
            if min_age_days >= max_age_days {
                bail!("--min-age-days must be lower than --max-age-days");
            }
            let source_dir = source_dir.unwrap_or_else(default_daily_dir);
            let archive_dir = archive_dir.unwrap_or_else(default_archive_dir);
            archive_daily_files(&source_dir, &archive_dir, min_age_days, max_age_days)?;
        }
    }

    Ok(())
}

async fn collect_single_snapshot(config: &Config) -> Result<Snapshot> {
    let (scheduler, mut rx) = Scheduler::new(build_registry(config), build_pipeline(config));
    let token = CancellationToken::new();

    let token_clone = token.clone();
    tokio::spawn(async move {
        scheduler.run(token_clone).await;
    });

    let tick = rx
        .recv()
        .await
        .context("Failed to receive a snapshot from scheduler")?;
    token.cancel();
    Ok(tick.snapshot)
}

fn append_daily_snapshot(output_dir: &Path, snapshot: &Snapshot) -> Result<()> {
    fs::create_dir_all(output_dir)?;
    let file_name = format!("{}.jsonl", Local::now().format("%F"));
    let output_path = output_dir.join(file_name);
    let line = serde_json::to_string(snapshot)?;

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&output_path)
        .with_context(|| format!("Failed to open {}", output_path.display()))?;
    writeln!(file, "{line}")?;
    println!("Appended snapshot to {}", output_path.display());
    Ok(())
}

fn prune_daily_files(directory: &Path, retention_days: u64) -> Result<()> {
    fs::create_dir_all(directory)?;
    let cutoff = SystemTime::now()
        .checked_sub(Duration::from_secs(retention_days.saturating_mul(86_400)))
        .context("Failed to compute retention cutoff")?;

    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if !is_jsonl_file(&path) {
            continue;
        }

        let modified = entry.metadata()?.modified()?;
        if modified < cutoff {
            fs::remove_file(&path)?;
            println!("Removed {}", path.display());
        }
    }

    Ok(())
}

fn archive_daily_files(
    source_dir: &Path,
    archive_dir: &Path,
    min_age_days: u64,
    max_age_days: u64,
) -> Result<()> {
    fs::create_dir_all(source_dir)?;
    fs::create_dir_all(archive_dir)?;

    let now = SystemTime::now();
    let min_age_cutoff = now
        .checked_sub(Duration::from_secs(min_age_days.saturating_mul(86_400)))
        .context("Failed to compute minimum archive cutoff")?;
    let max_age_cutoff = now
        .checked_sub(Duration::from_secs(max_age_days.saturating_mul(86_400)))
        .context("Failed to compute maximum archive cutoff")?;

    let mut files = Vec::new();
    for entry in fs::read_dir(source_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !is_jsonl_file(&path) {
            continue;
        }

        let modified = entry.metadata()?.modified()?;
        if modified <= min_age_cutoff && modified >= max_age_cutoff {
            files.push(path);
        }
    }

    if !files.is_empty() {
        let archive_path =
            archive_dir.join(format!("sysray-archive-{}.zip", Local::now().format("%F")));
        write_archive(&archive_path, &files)?;
        for path in &files {
            fs::remove_file(path)?;
        }
        println!("Created archive {}", archive_path.display());
    }

    let archive_cutoff = SystemTime::now()
        .checked_sub(Duration::from_secs(max_age_days.saturating_mul(86_400)))
        .context("Failed to compute archive retention cutoff")?;
    for entry in fs::read_dir(archive_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("zip") {
            continue;
        }

        if entry.metadata()?.modified()? < archive_cutoff {
            fs::remove_file(&path)?;
            println!("Removed {}", path.display());
        }
    }

    Ok(())
}

fn write_archive(target: &Path, files: &[PathBuf]) -> Result<()> {
    let file = File::create(target)?;
    let mut writer = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    for path in files {
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .context("Archive input is missing a valid file name")?;
        writer.start_file(name, options)?;
        let content = fs::read(path)?;
        writer.write_all(&content)?;
    }

    writer.finish()?;
    Ok(())
}

fn is_jsonl_file(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()) == Some("jsonl")
}

fn default_daily_dir() -> PathBuf {
    home_dir()
        .unwrap_or_else(|_| std::env::temp_dir())
        .join(".local")
        .join("share")
        .join("sysray")
        .join("daily")
}

fn default_archive_dir() -> PathBuf {
    home_dir()
        .unwrap_or_else(|_| std::env::temp_dir())
        .join(".local")
        .join("share")
        .join("sysray")
        .join("archives")
}

fn home_dir() -> Result<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .context("HOME is not set")
}
