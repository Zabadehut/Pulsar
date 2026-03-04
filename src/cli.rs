use clap::{Parser, Subcommand};
use std::path::PathBuf;

const AFTER_HELP: &str = "\
Native automation examples:

  sysray install
  sysray schedule install
  sysray maintenance daily-snapshot
  sysray maintenance prune --retention-days 15
  sysray maintenance archive --min-age-days 15 --max-age-days 60

macOS launchd:
  sysray install
  sysray schedule install
  sysray service install
  launchctl list | grep com.zabadehut.sysray

Windows Task Scheduler:
  sysray.exe install
  sysray.exe schedule install
  sysray.exe service install
  schtasks /Query /TN Sysray /V /FO LIST

Built-in service integration:
  Linux: systemd --user
  macOS: launchd LaunchAgent
  Windows: Task Scheduler

Reference and explain:
  sysray explain latency
  sysray explain swap --lang en --audience beginner
";

/// Sysray — System Observability Engine
/// Your system. Always beating.
#[derive(Parser, Debug)]
#[command(
    name = "sysray",
    version,
    author = "Kevin Vanden-Brande <zaba88@hotmail.fr>",
    about = "Local-first system observability engine for Linux, macOS, and Windows",
    after_help = AFTER_HELP,
    long_about = "Sysray is a local-first system observability engine.\n\nAvailable today:\n- interactive TUI with operator presets, detailed views, and a localized reference index\n- one-shot snapshots in json/csv/prometheus\n- local recording to .jsonl with built-in rotation, retention, and optional zip compression for closed segments\n- top/watch process inspection\n- OS service scaffolding\n\nStill planned:\n- a standalone archive command surface\n\nSee docs/help.md for the operator cheat sheet and recording workflow."
)]
pub struct Cli {
    /// Path to the Sysray configuration file
    #[arg(short, long, env = "SYSRAY_CONFIG", value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Override log level: error, warn, info, debug, or trace
    #[arg(short, long, value_name = "LEVEL")]
    pub log_level: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Launch the interactive dashboard (default command)
    Tui,

    /// Record snapshots continuously to newline-delimited JSON files
    Record {
        /// Collection interval, for example 5s or 10s
        #[arg(short, long)]
        interval: Option<String>,

        /// Output directory for generated .jsonl files
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Rotate raw files: never, hourly, or daily
        #[arg(long)]
        rotate: Option<String>,

        /// Rotate after this many megabytes
        #[arg(long, value_name = "MB")]
        max_file_size_mb: Option<u64>,

        /// Keep only the latest N local recording segments in the output directory
        #[arg(long, value_name = "COUNT")]
        keep_files: Option<usize>,

        /// Compress closed segments: none or zip
        #[arg(long)]
        compress: Option<String>,
    },

    /// Print one snapshot to stdout
    Snapshot {
        /// Output format: json, csv, or prometheus
        #[arg(short, long, default_value = "json")]
        format: String,
    },

    /// Start the local HTTP server exposing Prometheus metrics
    Server {
        /// TCP port to listen on
        #[arg(short, long, default_value_t = 9090)]
        port: u16,
    },

    /// Print the current host inventory with disk groups and network interfaces
    Inventory {
        /// Output format: table or json
        #[arg(short, long, default_value = "table")]
        format: String,
    },

    /// Show top processes in a one-shot view
    Top {
        /// Sort by cpu, mem, pid, or name
        #[arg(short, long, default_value = "cpu")]
        sort: String,

        /// Maximum number of processes to print
        #[arg(short, long, default_value_t = 20)]
        limit: usize,
    },

    /// Watch one process by PID
    Watch {
        /// Process ID to watch
        #[arg(short, long)]
        pid: u32,
    },

    /// Replay a recorded session from a file
    Replay {
        /// Path to a previously recorded session file
        file: PathBuf,
    },

    /// Explain a technical term using the shared reference catalog
    Explain {
        /// Search term, metric, or concept to explain
        term: String,

        /// Output language: fr or en
        #[arg(long, default_value = "fr")]
        lang: String,

        /// Audience level filter: beginner or expert
        #[arg(long)]
        audience: Option<String>,
    },

    /// Install the current executable to a stable user path
    Install {
        /// Copy the binary only and skip service installation
        #[arg(long)]
        no_service: bool,
    },

    /// Run built-in maintenance tasks without external shell scripts
    Maintenance {
        #[command(subcommand)]
        action: MaintenanceAction,
    },

    /// Install, remove, or inspect native recurring schedules
    Schedule {
        #[command(subcommand)]
        action: ScheduleAction,
    },

    /// Install, remove, or inspect OS service integration
    Service {
        #[command(subcommand)]
        action: ServiceAction,
    },
}

#[derive(Subcommand, Debug, Clone, Copy)]
pub enum ServiceAction {
    /// Install the current binary as a user-level service or scheduled task
    Install,
    /// Remove the installed service or scheduled task
    Uninstall,
    /// Query the current service or scheduled-task status
    Status,
}

#[derive(Subcommand, Debug, Clone)]
pub enum MaintenanceAction {
    /// Append one compact JSON snapshot to the daily jsonl file
    DailySnapshot {
        /// Output directory for YYYY-MM-DD.jsonl files
        #[arg(long)]
        output_dir: Option<PathBuf>,
    },
    /// Delete daily jsonl files older than N days
    Prune {
        /// Directory containing daily jsonl files
        #[arg(long)]
        directory: Option<PathBuf>,

        /// Delete files older than this many days
        #[arg(long, default_value_t = 15)]
        retention_days: u64,
    },
    /// Zip aged daily jsonl files and delete old archives
    Archive {
        /// Directory containing daily jsonl files
        #[arg(long)]
        source_dir: Option<PathBuf>,

        /// Output directory for zip archives
        #[arg(long)]
        archive_dir: Option<PathBuf>,

        /// Minimum file age in days before archiving
        #[arg(long, default_value_t = 15)]
        min_age_days: u64,

        /// Maximum age window in days kept in archive form
        #[arg(long, default_value_t = 60)]
        max_age_days: u64,
    },
}

#[derive(Subcommand, Debug, Clone, Copy)]
pub enum ScheduleAction {
    /// Install native recurring tasks for snapshot, prune, and archive
    Install,
    /// Remove native recurring tasks
    Uninstall,
    /// Query recurring-task status
    Status,
}
