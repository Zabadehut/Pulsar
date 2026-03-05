use crate::cli::ScheduleAction;
use crate::install;
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const CONFIG_TEMPLATE: &str = include_str!("../config/sysray.toml.example");

#[cfg(target_os = "linux")]
pub async fn run_schedule(action: ScheduleAction) -> Result<()> {
    linux::run(action)
}

#[cfg(target_os = "macos")]
pub async fn run_schedule(action: ScheduleAction) -> Result<()> {
    macos::run(action)
}

#[cfg(target_os = "windows")]
pub async fn run_schedule(action: ScheduleAction) -> Result<()> {
    windows::run(action)
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub async fn run_schedule(_action: ScheduleAction) -> Result<()> {
    bail!("Recurring schedule management is not supported on this OS")
}

fn home_dir() -> Result<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .context("HOME is not set")
}

fn write_template(target: &Path, template: &str) -> Result<()> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(target, template)?;
    Ok(())
}

fn ensure_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path)?;
    Ok(())
}

fn ensure_config_file(path: &Path) -> Result<()> {
    if !path.exists() {
        write_template(path, CONFIG_TEMPLATE)?;
    }
    Ok(())
}

fn run_command(program: &str, args: &[&str]) -> Result<()> {
    let status = Command::new(program)
        .args(args)
        .status()
        .with_context(|| format!("Failed to start {}", program))?;
    if !status.success() {
        bail!("{} {:?} failed with status {}", program, args, status);
    }
    Ok(())
}

fn preferred_executable_path() -> Result<PathBuf> {
    let install_path = install::install_path()?;
    if install_path.exists() {
        Ok(install_path)
    } else {
        std::env::current_exe().context("Failed to resolve current executable path")
    }
}

#[cfg(unix)]
fn write_runner_script(
    path: &Path,
    exe: &Path,
    config: &Path,
    maintenance_args: &[&str],
) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut script = format!(
        "#!/usr/bin/env sh\nexec \"{}\" --config \"{}\" maintenance",
        exe.display(),
        config.display()
    );
    for arg in maintenance_args {
        script.push(' ');
        script.push('"');
        script.push_str(arg);
        script.push('"');
    }
    script.push('\n');

    fs::write(path, script)?;
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms)?;
    Ok(())
}

#[cfg(windows)]
fn write_runner_script(
    path: &Path,
    exe: &Path,
    config: &Path,
    maintenance_args: &[&str],
) -> Result<()> {
    let mut script = format!(
        "@echo off\r\n\"{}\" --config \"{}\" maintenance",
        exe.display(),
        config.display()
    );
    for arg in maintenance_args {
        script.push(' ');
        script.push('"');
        script.push_str(arg);
        script.push('"');
    }
    script.push_str("\r\n");

    fs::write(path, script)?;
    Ok(())
}

#[cfg(target_os = "linux")]
mod linux {
    use super::*;

    struct ScheduleUnit {
        service_name: &'static str,
        timer_name: &'static str,
        runner_name: &'static str,
        maintenance_args: &'static [&'static str],
        on_calendar: &'static str,
    }

    const UNITS: &[ScheduleUnit] = &[
        ScheduleUnit {
            service_name: "sysray-snapshot.service",
            timer_name: "sysray-snapshot.timer",
            runner_name: "snapshot.sh",
            maintenance_args: &["daily-snapshot"],
            on_calendar: "*:0/5",
        },
        ScheduleUnit {
            service_name: "sysray-prune.service",
            timer_name: "sysray-prune.timer",
            runner_name: "prune.sh",
            maintenance_args: &["prune", "--retention-days", "15"],
            on_calendar: "*-*-* 02:00:00",
        },
        ScheduleUnit {
            service_name: "sysray-archive.service",
            timer_name: "sysray-archive.timer",
            runner_name: "archive.sh",
            maintenance_args: &["archive", "--min-age-days", "15", "--max-age-days", "60"],
            on_calendar: "*-*-* 02:30:00",
        },
    ];

    pub fn run(action: ScheduleAction) -> Result<()> {
        let config_dir = home_dir()?.join(".config/sysray");
        let data_dir = home_dir()?.join(".local/share/sysray");
        let runner_dir = data_dir.join("schedule");
        let unit_dir = home_dir()?.join(".config/systemd/user");
        let config_path = config_dir.join("sysray.toml");
        let exe = preferred_executable_path()?;

        match action {
            ScheduleAction::Install => {
                ensure_dir(&config_dir)?;
                ensure_dir(&runner_dir)?;
                ensure_dir(&unit_dir)?;
                ensure_config_file(&config_path)?;

                for unit in UNITS {
                    let runner_path = runner_dir.join(unit.runner_name);
                    let service_path = unit_dir.join(unit.service_name);
                    let timer_path = unit_dir.join(unit.timer_name);
                    write_runner_script(&runner_path, &exe, &config_path, unit.maintenance_args)?;
                    write_template(
                        &service_path,
                        &format!(
                            "[Unit]\nDescription=Sysray scheduled task {}\n\n[Service]\nType=oneshot\nExecStart={}\n",
                            unit.service_name,
                            runner_path.display()
                        ),
                    )?;
                    write_template(
                        &timer_path,
                        &format!(
                            "[Unit]\nDescription=Sysray timer {}\n\n[Timer]\nOnCalendar={}\nPersistent=true\nUnit={}\n\n[Install]\nWantedBy=timers.target\n",
                            unit.timer_name,
                            unit.on_calendar,
                            unit.service_name
                        ),
                    )?;
                }

                run_command("systemctl", &["--user", "daemon-reload"])?;
                for unit in UNITS {
                    run_command("systemctl", &["--user", "enable", "--now", unit.timer_name])?;
                }
                println!(
                    "Installed native recurring schedules under {}",
                    unit_dir.display()
                );
            }
            ScheduleAction::Uninstall => {
                for unit in UNITS {
                    let _ = run_command(
                        "systemctl",
                        &["--user", "disable", "--now", unit.timer_name],
                    );
                    let service_path = unit_dir.join(unit.service_name);
                    let timer_path = unit_dir.join(unit.timer_name);
                    let runner_path = runner_dir.join(unit.runner_name);
                    if service_path.exists() {
                        fs::remove_file(service_path)?;
                    }
                    if timer_path.exists() {
                        fs::remove_file(timer_path)?;
                    }
                    if runner_path.exists() {
                        fs::remove_file(runner_path)?;
                    }
                }
                let _ = run_command("systemctl", &["--user", "daemon-reload"]);
                println!(
                    "Removed native recurring schedules from {}",
                    unit_dir.display()
                );
            }
            ScheduleAction::Status => {
                for unit in UNITS {
                    run_command("systemctl", &["--user", "status", unit.timer_name])?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;

    struct ScheduleAgent {
        label: &'static str,
        file_name: &'static str,
        runner_name: &'static str,
        maintenance_args: &'static [&'static str],
        trigger: Trigger,
    }

    enum Trigger {
        Interval(u32),
        Calendar { hour: u8, minute: u8 },
    }

    const AGENTS: &[ScheduleAgent] = &[
        ScheduleAgent {
            label: "com.zabadehut.sysray.snapshot",
            file_name: "com.zabadehut.sysray.snapshot.plist",
            runner_name: "snapshot.sh",
            maintenance_args: &["daily-snapshot"],
            trigger: Trigger::Interval(300),
        },
        ScheduleAgent {
            label: "com.zabadehut.sysray.prune",
            file_name: "com.zabadehut.sysray.prune.plist",
            runner_name: "prune.sh",
            maintenance_args: &["prune", "--retention-days", "15"],
            trigger: Trigger::Calendar { hour: 2, minute: 0 },
        },
        ScheduleAgent {
            label: "com.zabadehut.sysray.archive",
            file_name: "com.zabadehut.sysray.archive.plist",
            runner_name: "archive.sh",
            maintenance_args: &["archive", "--min-age-days", "15", "--max-age-days", "60"],
            trigger: Trigger::Calendar {
                hour: 2,
                minute: 30,
            },
        },
    ];

    pub fn run(action: ScheduleAction) -> Result<()> {
        let app_dir = home_dir()?.join("Library/Application Support/Sysray");
        let runner_dir = app_dir.join("schedule");
        let config_path = app_dir.join("sysray.toml");
        let agents_dir = home_dir()?.join("Library/LaunchAgents");
        let exe = preferred_executable_path()?;

        match action {
            ScheduleAction::Install => {
                ensure_dir(&app_dir)?;
                ensure_dir(&runner_dir)?;
                ensure_dir(&agents_dir)?;
                ensure_config_file(&config_path)?;

                for agent in AGENTS {
                    let runner_path = runner_dir.join(agent.runner_name);
                    let plist_path = agents_dir.join(agent.file_name);
                    write_runner_script(&runner_path, &exe, &config_path, agent.maintenance_args)?;
                    write_template(&plist_path, &plist_for_agent(agent, &runner_path))?;
                    let _ = run_command(
                        "launchctl",
                        &["unload", plist_path.to_string_lossy().as_ref()],
                    );
                    run_command(
                        "launchctl",
                        &["load", plist_path.to_string_lossy().as_ref()],
                    )?;
                }
                println!(
                    "Installed native recurring schedules under {}",
                    agents_dir.display()
                );
            }
            ScheduleAction::Uninstall => {
                for agent in AGENTS {
                    let plist_path = agents_dir.join(agent.file_name);
                    let runner_path = runner_dir.join(agent.runner_name);
                    let _ = run_command(
                        "launchctl",
                        &["unload", plist_path.to_string_lossy().as_ref()],
                    );
                    if plist_path.exists() {
                        fs::remove_file(plist_path)?;
                    }
                    if runner_path.exists() {
                        fs::remove_file(runner_path)?;
                    }
                }
                println!(
                    "Removed native recurring schedules from {}",
                    agents_dir.display()
                );
            }
            ScheduleAction::Status => {
                for agent in AGENTS {
                    run_command("launchctl", &["list", agent.label])?;
                }
            }
        }

        Ok(())
    }

    fn plist_for_agent(agent: &ScheduleAgent, runner_path: &Path) -> String {
        let trigger = match agent.trigger {
            Trigger::Interval(seconds) => {
                format!("  <key>StartInterval</key>\n  <integer>{seconds}</integer>\n")
            }
            Trigger::Calendar { hour, minute } => format!(
                "  <key>StartCalendarInterval</key>\n  <dict>\n    <key>Hour</key>\n    <integer>{hour}</integer>\n    <key>Minute</key>\n    <integer>{minute}</integer>\n  </dict>\n"
            ),
        };

        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n<plist version=\"1.0\">\n<dict>\n  <key>Label</key>\n  <string>{}</string>\n  <key>ProgramArguments</key>\n  <array>\n    <string>{}</string>\n  </array>\n{}  <key>RunAtLoad</key>\n  <false/>\n</dict>\n</plist>\n",
            agent.label,
            runner_path.display(),
            trigger
        )
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use super::*;

    struct ScheduledTask {
        name: &'static str,
        runner_name: &'static str,
        maintenance_args: &'static [&'static str],
        schedule_args: &'static [&'static str],
    }

    const TASKS: &[ScheduledTask] = &[
        ScheduledTask {
            name: "Sysray Snapshot",
            runner_name: "snapshot.cmd",
            maintenance_args: &["daily-snapshot"],
            schedule_args: &["/SC", "MINUTE", "/MO", "5"],
        },
        ScheduledTask {
            name: "Sysray Prune",
            runner_name: "prune.cmd",
            maintenance_args: &["prune", "--retention-days", "15"],
            schedule_args: &["/SC", "DAILY", "/ST", "02:00"],
        },
        ScheduledTask {
            name: "Sysray Archive",
            runner_name: "archive.cmd",
            maintenance_args: &["archive", "--min-age-days", "15", "--max-age-days", "60"],
            schedule_args: &["/SC", "DAILY", "/ST", "02:30"],
        },
    ];

    pub fn run(action: ScheduleAction) -> Result<()> {
        let app_dir = std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or(std::env::temp_dir())
            .join("Sysray");
        let runner_dir = app_dir.join("schedule");
        let config_path = app_dir.join("sysray.toml");
        let exe = preferred_executable_path()?;

        match action {
            ScheduleAction::Install => {
                ensure_dir(&app_dir)?;
                ensure_dir(&runner_dir)?;
                ensure_config_file(&config_path)?;

                for task in TASKS {
                    let runner_path = runner_dir.join(task.runner_name);
                    write_runner_script(&runner_path, &exe, &config_path, task.maintenance_args)?;

                    let task_command = format!("cmd.exe /c \"\\\"{}\\\"\"", runner_path.display());
                    let mut args = vec!["/Create", "/TN", task.name];
                    args.extend_from_slice(task.schedule_args);
                    args.extend_from_slice(&["/TR", &task_command, "/F"]);
                    run_command("schtasks", &args)?;
                }
                println!(
                    "Installed native recurring schedules under {}",
                    runner_dir.display()
                );
            }
            ScheduleAction::Uninstall => {
                for task in TASKS {
                    let _ = run_command("schtasks", &["/Delete", "/TN", task.name, "/F"]);
                    let runner_path = runner_dir.join(task.runner_name);
                    if runner_path.exists() {
                        fs::remove_file(runner_path)?;
                    }
                }
                println!(
                    "Removed native recurring schedules from {}",
                    runner_dir.display()
                );
            }
            ScheduleAction::Status => {
                for task in TASKS {
                    run_command(
                        "schtasks",
                        &["/Query", "/TN", task.name, "/V", "/FO", "LIST"],
                    )?;
                }
            }
        }
        Ok(())
    }
}
