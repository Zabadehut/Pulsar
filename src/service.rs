use crate::cli::ServiceAction;
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const CONFIG_TEMPLATE: &str = include_str!("../config/pulsar.toml.example");

#[cfg(target_os = "linux")]
pub async fn run_service(action: ServiceAction) -> Result<()> {
    linux::run(action)
}

#[cfg(target_os = "macos")]
pub async fn run_service(action: ServiceAction) -> Result<()> {
    macos::run(action)
}

#[cfg(target_os = "windows")]
pub async fn run_service(action: ServiceAction) -> Result<()> {
    windows::run(action)
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub async fn run_service(_action: ServiceAction) -> Result<()> {
    bail!("Service management is not supported on this OS")
}

fn current_exe_string() -> Result<String> {
    Ok(std::env::current_exe()?.to_string_lossy().into_owned())
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

#[cfg(unix)]
fn write_runner_script(path: &Path, exe: &str, config: &Path, output: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let script = format!(
        "#!/usr/bin/env sh\nexec \"{}\" --config \"{}\" record --interval 5s --output \"{}\"\n",
        exe,
        config.display(),
        output.display()
    );
    fs::write(path, script)?;
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms)?;
    Ok(())
}

#[cfg(windows)]
fn write_runner_script(path: &Path, exe: &str, config: &Path, output: &Path) -> Result<()> {
    let script = format!(
        "@echo off\r\n\"{}\" --config \"{}\" record --interval 5s --output \"{}\"\r\n",
        exe,
        config.display(),
        output.display()
    );
    fs::write(path, script)?;
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

#[cfg(target_os = "linux")]
mod linux {
    use super::*;

    const TEMPLATE: &str = include_str!("../deploy/systemd/pulsar.service");

    pub fn run(action: ServiceAction) -> Result<()> {
        let config_dir = home_dir()?.join(".config/pulsar");
        let data_dir = home_dir()?.join(".local/share/pulsar");
        let config_path = config_dir.join("pulsar.toml");
        let runner_path = data_dir.join("pulsar-service.sh");
        let unit_path = home_dir()?.join(".config/systemd/user/pulsar.service");

        match action {
            ServiceAction::Install => {
                ensure_dir(&config_dir)?;
                ensure_dir(&data_dir)?;
                ensure_config_file(&config_path)?;
                let exe = current_exe_string()?;
                write_runner_script(&runner_path, &exe, &config_path, &data_dir)?;

                let content = TEMPLATE.replace("__PULSAR_RUNNER__", &runner_path.to_string_lossy());
                write_template(&unit_path, &content)?;
                run_command("systemctl", &["--user", "daemon-reload"])?;
                run_command(
                    "systemctl",
                    &["--user", "enable", "--now", "pulsar.service"],
                )?;
                println!(
                    "Installed user service at {} using config {} and output {}",
                    unit_path.display(),
                    config_path.display(),
                    data_dir.display()
                );
            }
            ServiceAction::Uninstall => {
                let _ = run_command(
                    "systemctl",
                    &["--user", "disable", "--now", "pulsar.service"],
                );
                if unit_path.exists() {
                    fs::remove_file(&unit_path)?;
                }
                if runner_path.exists() {
                    fs::remove_file(&runner_path)?;
                }
                let _ = run_command("systemctl", &["--user", "daemon-reload"]);
                println!("Removed user service from {}", unit_path.display());
            }
            ServiceAction::Status => {
                run_command("systemctl", &["--user", "status", "pulsar.service"])?;
            }
        }
        Ok(())
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;

    const TEMPLATE: &str = include_str!("../deploy/launchd/com.zabadehut.pulsar.plist");

    pub fn run(action: ServiceAction) -> Result<()> {
        let app_dir = home_dir()?.join("Library/Application Support/Pulsar");
        let config_path = app_dir.join("pulsar.toml");
        let output_dir = app_dir.join("data");
        let runner_path = app_dir.join("pulsar-service.sh");
        let plist_path = home_dir()?.join("Library/LaunchAgents/com.zabadehut.pulsar.plist");
        let label = "com.zabadehut.pulsar";
        match action {
            ServiceAction::Install => {
                ensure_dir(&app_dir)?;
                ensure_dir(&output_dir)?;
                ensure_config_file(&config_path)?;
                let exe = current_exe_string()?;
                write_runner_script(&runner_path, &exe, &config_path, &output_dir)?;

                let content = TEMPLATE.replace("__PULSAR_RUNNER__", &runner_path.to_string_lossy());
                write_template(&plist_path, &content)?;
                run_command(
                    "launchctl",
                    &["unload", plist_path.to_string_lossy().as_ref()],
                )
                .ok();
                run_command(
                    "launchctl",
                    &["load", plist_path.to_string_lossy().as_ref()],
                )?;
                println!(
                    "Installed launch agent at {} using config {} and output {}",
                    plist_path.display(),
                    config_path.display(),
                    output_dir.display()
                );
            }
            ServiceAction::Uninstall => {
                let _ = run_command(
                    "launchctl",
                    &["unload", plist_path.to_string_lossy().as_ref()],
                );
                if plist_path.exists() {
                    fs::remove_file(&plist_path)?;
                }
                if runner_path.exists() {
                    fs::remove_file(&runner_path)?;
                }
                println!("Removed launch agent {}", label);
            }
            ServiceAction::Status => {
                run_command("launchctl", &["list", label])?;
            }
        }
        Ok(())
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use super::*;

    const TEMPLATE: &str = include_str!("../deploy/windows/pulsar-task.xml");

    pub fn run(action: ServiceAction) -> Result<()> {
        let app_dir = std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or(std::env::temp_dir())
            .join("Pulsar");
        let config_path = app_dir.join("pulsar.toml");
        let output_dir = app_dir.join("data");
        let runner_path = app_dir.join("pulsar-service.cmd");
        let xml_path = app_dir.join("pulsar-task.xml");
        let task_name = "Pulsar";
        match action {
            ServiceAction::Install => {
                ensure_dir(&app_dir)?;
                ensure_dir(&output_dir)?;
                ensure_config_file(&config_path)?;
                let exe = current_exe_string()?;
                write_runner_script(&runner_path, &exe, &config_path, &output_dir)?;

                let content = TEMPLATE.replace("__PULSAR_RUNNER__", &runner_path.to_string_lossy());
                write_template(&xml_path, &content)?;
                run_command(
                    "schtasks",
                    &[
                        "/Create",
                        "/TN",
                        task_name,
                        "/XML",
                        xml_path.to_string_lossy().as_ref(),
                        "/F",
                    ],
                )?;
                println!(
                    "Installed scheduled task {} using config {} and output {}",
                    task_name,
                    config_path.display(),
                    output_dir.display()
                );
            }
            ServiceAction::Uninstall => {
                run_command("schtasks", &["/Delete", "/TN", task_name, "/F"])?;
                if runner_path.exists() {
                    fs::remove_file(&runner_path)?;
                }
                if xml_path.exists() {
                    fs::remove_file(&xml_path)?;
                }
                println!("Removed scheduled task {}", task_name);
            }
            ServiceAction::Status => {
                run_command(
                    "schtasks",
                    &["/Query", "/TN", task_name, "/V", "/FO", "LIST"],
                )?;
            }
        }
        Ok(())
    }
}
