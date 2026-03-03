use clap::{Parser, Subcommand};
use std::path::PathBuf;

const AFTER_HELP: &str = "\
Scheduling examples:

Linux cron tasks:
  */5 * * * * /home/<user>/dev/pulsar/scripts/pulsar-daily-snapshot.sh
  0 2 */15 * * /home/<user>/dev/pulsar/scripts/pulsar-prune-raw.sh
  30 2 * * * /home/<user>/dev/pulsar/scripts/pulsar-archive-raw.sh

Linux task intent:
  1) append snapshots into ~/.local/share/pulsar/daily/YYYY-MM-DD.jsonl
  2) rotate raw files by hour/day/size when needed
  3) keep raw retention bounded with --keep

macOS launchd:
  pulsar service install
  launchctl list | grep com.zabadehut.pulsar

Windows Task Scheduler:
  pulsar.exe service install
  schtasks /Query /TN Pulsar /V /FO LIST

Built-in service integration:
  Linux: systemd --user
  macOS: launchd LaunchAgent
  Windows: Task Scheduler

Reference and explain:
  pulsar explain latency
  pulsar explain swap --lang en --audience beginner
";

/// Pulsar — System Observability Engine
/// Your system. Always beating.
#[derive(Parser, Debug)]
#[command(
    name = "pulsar",
    version,
    author = "Kevin Vanden-Brande <zaba88@hotmail.fr>",
    about = "Local-first system observability engine for Linux, macOS, and Windows",
    after_help = AFTER_HELP,
    long_about = "Pulsar is a local-first system observability engine.\n\nAvailable today:\n- interactive TUI with operator presets and reference index\n- one-shot snapshots in json/csv/prometheus\n- local recording to .jsonl with built-in rotation and raw retention\n- top/watch process inspection\n- OS service scaffolding\n\nStill planned:\n- built-in archive zip compression\n\nSee docs/help.md for the operator cheat sheet and recording workflow."
)]
pub struct Cli {
    /// Path to the Pulsar configuration file
    #[arg(short, long, env = "PULSAR_CONFIG", value_name = "FILE")]
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

        /// Keep only the latest N raw .jsonl files in the output directory
        #[arg(long, value_name = "COUNT")]
        keep_files: Option<usize>,
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
