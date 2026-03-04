use crate::cli::ServiceAction;
use crate::service;
use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub async fn install_current_executable(install_service: bool) -> Result<()> {
    let source = env::current_exe().context("Failed to resolve current executable path")?;
    let destination = install_path()?;

    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }

    if source != destination {
        fs::copy(&source, &destination).with_context(|| {
            format!(
                "Failed to copy executable from {} to {}",
                source.display(),
                destination.display()
            )
        })?;
        ensure_executable(&destination)?;
    }

    println!("Installed binary: {}", destination.display());

    if install_service {
        service::run_service_with_exe(ServiceAction::Install, Some(destination.as_path())).await?;
    }

    if !path_contains(destination.parent().unwrap_or(Path::new(""))) {
        println!(
            "PATH does not currently include {}",
            destination.parent().unwrap_or(Path::new("")).display()
        );
    }

    Ok(())
}

pub fn install_path() -> Result<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let local_app_data = env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .context("LOCALAPPDATA is not set")?;
        Ok(local_app_data
            .join("Programs")
            .join("Sysray")
            .join("sysray.exe"))
    }

    #[cfg(not(target_os = "windows"))]
    {
        let home = env::var_os("HOME")
            .map(PathBuf::from)
            .context("HOME is not set")?;
        Ok(home.join(".local").join("bin").join("sysray"))
    }
}

fn path_contains(dir: &Path) -> bool {
    env::var_os("PATH")
        .map(|value| env::split_paths(&value).any(|entry| entry == dir))
        .unwrap_or(false)
}

fn ensure_executable(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms)?;
    }

    #[cfg(not(unix))]
    {
        let _ = path;
    }

    Ok(())
}
