mod api;
mod cli;
mod collectors;
mod config;
mod engine;
mod exporters;
mod pipeline;
mod platform;
mod recording;
mod reference;
mod service;
mod tui;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};
use collectors::{
    CpuCollector, DiskCollector, LinuxCollector, MemoryCollector, NetworkCollector,
    ProcessCollector, SystemCollector,
};
use config::Config;
use engine::{Registry, Scheduler};
use exporters::{csv::CsvExporter, json::JsonExporter, prometheus::PrometheusExporter, Exporter};
use pipeline::{AlertStage, CpuTrendStage, MemoryPressureStage, PipelineRunner, PipelineStage};
use recording::RecordRuntimeOptions;
use reference::{Audience, Locale};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let config_path = cli
        .config
        .clone()
        .unwrap_or_else(|| PathBuf::from("pulsar.toml"));
    let config = Config::load_or_default(&config_path);

    let log_level = cli
        .log_level
        .as_deref()
        .unwrap_or(&config.general.log_level)
        .to_string();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                log_level
                    .parse()
                    .unwrap_or_else(|_| "info".parse().unwrap())
            }),
        )
        .with_target(false)
        .with_writer(std::io::stderr) // logs → stderr, stdout réservé aux données
        .compact()
        .init();

    match cli.command.unwrap_or(Commands::Tui) {
        Commands::Tui => run_tui(config).await,
        Commands::Snapshot { format } => run_snapshot(&config, &format).await,
        Commands::Server { port } => run_server(config, port).await,
        Commands::Top { sort, limit } => run_top(config, &sort, limit).await,
        Commands::Record {
            interval,
            output,
            rotate,
            max_file_size_mb,
            keep_files,
            compress,
        } => {
            let options = RecordRuntimeOptions::from_sources(
                &config.record,
                interval.as_deref(),
                output,
                rotate.as_deref(),
                max_file_size_mb,
                keep_files,
                compress.as_deref(),
            );
            run_record(config, options).await
        }
        Commands::Watch { pid } => run_watch(pid).await,
        Commands::Replay { file } => run_replay(file).await,
        Commands::Explain {
            term,
            lang,
            audience,
        } => run_explain(&term, &lang, audience.as_deref()),
        Commands::Service { action } => service::run_service(action).await,
    }
}

// ─── Registry builder ────────────────────────────────────────────────────────

fn build_registry(config: &Config) -> Registry {
    let mut r = Registry::new();
    let col = &config.collectors;
    let secs = |n: u64| Duration::from_secs(n);

    if col.cpu {
        r.register(CpuCollector::new(), secs(col.cpu_interval_secs));
    }
    if col.memory {
        r.register(MemoryCollector::new(), secs(col.memory_interval_secs));
    }
    if col.disk {
        r.register(DiskCollector::new(), secs(col.disk_interval_secs));
    }
    if col.network {
        r.register(NetworkCollector::new(), secs(col.network_interval_secs));
    }
    if col.process {
        r.register(
            ProcessCollector::new(col.process_top_n, col.jvm_detection),
            secs(col.process_interval_secs),
        );
    }
    r.register(SystemCollector::new(), secs(col.system_interval_secs));
    #[cfg(target_os = "linux")]
    r.register(LinuxCollector::new(), secs(col.system_interval_secs));
    r
}

fn build_pipeline(config: &Config) -> PipelineRunner {
    let mut stages: Vec<Box<dyn PipelineStage>> = Vec::new();
    let pipeline = &config.pipeline;

    if pipeline.cpu_trend {
        stages.push(Box::new(CpuTrendStage::new(60)));
    }
    if pipeline.mem_pressure {
        stages.push(Box::new(MemoryPressureStage::new()));
    }
    if pipeline.alerts {
        stages.push(Box::new(AlertStage::new(pipeline.thresholds.clone())));
    }

    PipelineRunner::new(stages)
}

// ─── Mode TUI ────────────────────────────────────────────────────────────────

async fn run_tui(config: Config) -> Result<()> {
    info!("Starting Pulsar TUI");
    let (scheduler, rx) = Scheduler::new(build_registry(&config), build_pipeline(&config));
    let token = CancellationToken::new();

    let token_clone = token.clone();
    tokio::spawn(async move {
        scheduler.run(token_clone).await;
    });

    tui::run_tui(&config.tui, rx).await?;
    token.cancel();
    Ok(())
}

// ─── Mode Snapshot ───────────────────────────────────────────────────────────

async fn run_snapshot(config: &Config, format: &str) -> Result<()> {
    let (scheduler, mut rx) = Scheduler::new(build_registry(config), build_pipeline(config));
    let token = CancellationToken::new();

    let token_clone = token.clone();
    tokio::spawn(async move {
        scheduler.run(token_clone).await;
    });

    if let Ok(tick) = rx.recv().await {
        let exporter: &dyn Exporter = match format {
            "csv" => &CsvExporter,
            "prometheus" => &PrometheusExporter,
            _ => &JsonExporter,
        };
        info!(exporter = exporter.name(), format, "Snapshot exported");
        println!("{}", exporter.export(&tick.snapshot)?);
    }

    token.cancel();
    Ok(())
}

// ─── Mode Server ─────────────────────────────────────────────────────────────

async fn run_server(config: Config, port: u16) -> Result<()> {
    info!("Starting Pulsar server on :{}", port);
    let (scheduler, _rx) = Scheduler::new(build_registry(&config), build_pipeline(&config));
    let latest = scheduler.latest();
    let health = scheduler.health();
    let token = CancellationToken::new();

    let token_clone = token.clone();
    tokio::spawn(async move {
        scheduler.run(token_clone).await;
    });

    api::run_server(&config.api.bind, port, latest, health).await?;
    Ok(())
}

// ─── Mode Top ────────────────────────────────────────────────────────────────

async fn run_top(config: Config, sort: &str, limit: usize) -> Result<()> {
    let mut r = Registry::new();
    let interval = Duration::from_secs(config.collectors.process_interval_secs);
    r.register(
        ProcessCollector::new(limit, config.collectors.jvm_detection),
        interval,
    );
    let (scheduler, mut rx) = Scheduler::new(r, build_pipeline(&config));
    let token = CancellationToken::new();

    let token_clone = token.clone();
    tokio::spawn(async move {
        scheduler.run(token_clone).await;
    });

    if let Ok(tick) = rx.recv().await {
        let mut procs = tick.snapshot.processes;
        match sort {
            "mem" => procs.sort_by(|a, b| b.mem_rss_kb.cmp(&a.mem_rss_kb)),
            "pid" => procs.sort_by_key(|p| p.pid),
            "name" => procs.sort_by(|a, b| a.name.cmp(&b.name)),
            _ => procs.sort_by(|a, b| {
                b.cpu_pct
                    .partial_cmp(&a.cpu_pct)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }),
        }
        println!(
            "{:<7} {:<18} {:>7} {:>9} {:>8} USER",
            "PID", "NAME", "CPU%", "MEM MB", "THREADS"
        );
        println!("{}", "─".repeat(62));
        for p in procs.iter().take(limit) {
            println!(
                "{:<7} {:<18} {:>6.1}% {:>8.0}M {:>8} {}",
                p.pid,
                &p.name[..p.name.len().min(17)],
                p.cpu_pct,
                p.mem_rss_kb as f64 / 1024.0,
                p.threads,
                p.user,
            );
        }
    }

    token.cancel();
    Ok(())
}

// ─── Mode Record ─────────────────────────────────────────────────────────────

async fn run_record(config: Config, options: RecordRuntimeOptions) -> Result<()> {
    info!(
        "Recording every {}s to {:?} — Ctrl+C to stop",
        options.interval_secs, options.output
    );
    let (scheduler, mut rx) = Scheduler::new(build_registry(&config), build_pipeline(&config));
    let token = CancellationToken::new();

    let token_clone = token.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        token_clone.cancel();
    });

    let token_clone2 = token.clone();
    tokio::spawn(async move {
        scheduler.run(token_clone2).await;
    });

    let mut recorder = recording::Recorder::new(options.clone())?;

    let min_interval = std::time::Duration::from_secs(options.interval_secs);
    let mut last_write = std::time::Instant::now() - min_interval;

    while let Ok(tick) = rx.recv().await {
        if last_write.elapsed() >= min_interval {
            recorder.write_snapshot(&tick.snapshot)?;
            last_write = std::time::Instant::now();
        }
    }
    Ok(())
}

async fn run_watch(pid: u32) -> Result<()> {
    let config = Config::default();
    let mut registry = Registry::new();
    registry.register(
        ProcessCollector::new_watching(pid, config.collectors.jvm_detection),
        Duration::from_secs(config.collectors.process_interval_secs),
    );
    let (scheduler, mut rx) = Scheduler::new(registry, build_pipeline(&config));
    let token = CancellationToken::new();

    let token_clone = token.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        token_clone.cancel();
    });

    let token_clone = token.clone();
    tokio::spawn(async move {
        scheduler.run(token_clone).await;
    });

    let mut stdout = std::io::stdout();
    while let Ok(tick) = rx.recv().await {
        print!("\x1B[2J\x1B[H");
        if let Some(process) = tick.snapshot.processes.into_iter().find(|p| p.pid == pid) {
            println!("Watching PID {} ({})", process.pid, process.name);
            println!(
                "User: {}  State: {:?}  JVM: {}",
                process.user, process.state, process.is_jvm
            );
            println!(
                "CPU: {:.1}%  RSS: {:.1} MB  VSZ: {:.1} MB",
                process.cpu_pct,
                process.mem_rss_kb as f64 / 1024.0,
                process.mem_vsz_kb as f64 / 1024.0
            );
            println!(
                "Threads: {}  FDs: {}  Read: {} B  Write: {} B",
                process.threads, process.fd_count, process.io_read_bytes, process.io_write_bytes
            );
            println!("Cmdline: {}", process.cmdline);
        } else {
            println!("PID {} not found in current snapshot.", pid);
        }
        println!();
        println!("Press Ctrl+C to stop.");
        stdout.flush()?;
    }

    Ok(())
}

async fn run_replay(file: PathBuf) -> Result<()> {
    let reader = BufReader::new(File::open(&file)?);
    let mut previous_ts: Option<i64> = None;

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let snapshot: collectors::Snapshot = serde_json::from_str(&line)?;
        if let Some(prev) = previous_ts {
            let delay_secs = (snapshot.timestamp - prev).max(0) as u64;
            if delay_secs > 0 {
                tokio::time::sleep(Duration::from_secs(delay_secs.min(2))).await;
            }
        }

        print!("\x1B[2J\x1B[H");
        println!("Replay {}", file.display());
        println!("Timestamp: {}", snapshot.timestamp);

        if let Some(system) = snapshot.system.as_ref() {
            println!(
                "Host: {}  OS: {} {}  Uptime: {}s",
                system.hostname, system.os_name, system.os_version, system.uptime_seconds
            );
        }
        if let Some(cpu) = snapshot.cpu.as_ref() {
            println!(
                "CPU: {:.1}%  load {:.2}/{:.2}/{:.2}  iowait {:.1}%  steal {:.1}%",
                cpu.global_usage_pct,
                cpu.load_avg_1,
                cpu.load_avg_5,
                cpu.load_avg_15,
                cpu.iowait_pct,
                cpu.steal_pct
            );
        }
        if let Some(memory) = snapshot.memory.as_ref() {
            println!(
                "Memory: {:.1}%  used {:.1}/{:.1} GB  pressure {:.0}%",
                memory.usage_pct,
                memory.used_kb as f64 / 1_048_576.0,
                memory.total_kb as f64 / 1_048_576.0,
                snapshot.computed.memory_pressure * 100.0
            );
        }
        println!(
            "Disks: {}  Networks: {}  Processes: {}",
            snapshot.disks.len(),
            snapshot.networks.len(),
            snapshot.processes.len()
        );
        if !snapshot.computed.alerts.is_empty() {
            println!("Alerts:");
            for alert in &snapshot.computed.alerts {
                println!("- {:?}: {}", alert.level, alert.message);
            }
        }

        previous_ts = Some(snapshot.timestamp);
    }

    Ok(())
}

fn run_explain(term: &str, lang: &str, audience: Option<&str>) -> Result<()> {
    let locale = Locale::parse(lang);
    let audience = audience.and_then(parse_audience);
    let hits = reference::search(term, locale);

    let filtered: Vec<_> = hits
        .into_iter()
        .filter(|hit| audience.is_none_or(|value| hit.entry.audience == value))
        .collect();

    if filtered.is_empty() {
        println!("No reference entry found for '{term}'.");
        return Ok(());
    }

    for (index, hit) in filtered.iter().enumerate() {
        if index > 0 {
            println!();
        }

        println!(
            "{} [{}] ({:?})",
            hit.entry.title, hit.entry.panel, hit.entry.audience
        );
        println!("id: {}", hit.entry.id);
        println!("category: {}", hit.entry.category);
        println!("status: {:?}", hit.entry.status);
        println!("ui_visibility: {:?}", hit.entry.ui_visibility);
        println!("summary: {}", hit.entry.summary);
        println!("beginner: {}", hit.entry.beginner);
        println!("expert: {}", hit.entry.expert);
        println!("aliases: {}", hit.entry.aliases.join(", "));
        println!("tags: {}", hit.entry.tags.join(", "));
    }

    Ok(())
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn parse_audience(value: &str) -> Option<Audience> {
    match value.to_ascii_lowercase().as_str() {
        "beginner" | "debutant" => Some(Audience::Beginner),
        "expert" => Some(Audience::Expert),
        _ => None,
    }
}
