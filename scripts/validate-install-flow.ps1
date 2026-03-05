param(
    [string]$Binary = "target/debug/sysray.exe"
)

$ErrorActionPreference = "Stop"
$Binary = (Resolve-Path $Binary).Path

$root = Join-Path $env:RUNNER_TEMP ("sysray-install-validation-" + [guid]::NewGuid().ToString())
$env:LOCALAPPDATA = Join-Path $root "AppData\Local"
$env:APPDATA = Join-Path $root "AppData\Roaming"
New-Item -ItemType Directory -Force -Path $env:LOCALAPPDATA | Out-Null
New-Item -ItemType Directory -Force -Path $env:APPDATA | Out-Null

$installPath = Join-Path $env:LOCALAPPDATA "Programs\Sysray\sysray.exe"
$appDir = Join-Path $env:APPDATA "Sysray"
$configPath = Join-Path $appDir "sysray.toml"
$serviceRunnerPath = Join-Path $appDir "sysray-service.cmd"
$serviceXmlPath = Join-Path $appDir "sysray-task.xml"
$scheduleDir = Join-Path $appDir "schedule"
$scheduleRunnerPaths = @(
    (Join-Path $scheduleDir "snapshot.cmd"),
    (Join-Path $scheduleDir "prune.cmd"),
    (Join-Path $scheduleDir "archive.cmd")
)
$taskNames = @("Sysray", "Sysray Snapshot", "Sysray Prune", "Sysray Archive")

try {
    & $Binary install --no-path
    $installExitCode = $LASTEXITCODE
    if ($installExitCode -ne 0 -and -not $env:GITHUB_ACTIONS) {
        throw "install failed with exit code $LASTEXITCODE"
    }

    foreach ($path in @($installPath, $configPath, $serviceRunnerPath, $serviceXmlPath) + $scheduleRunnerPaths) {
        if (-not (Test-Path $path)) {
            throw "missing expected install artifact: $path"
        }
    }

    if (-not (Select-String -Path $serviceRunnerPath -Pattern ([regex]::Escape($installPath)) -Quiet)) {
        throw "service runner script does not reference the installed binary"
    }

    if (-not (Select-String -Path $serviceXmlPath -Pattern ([regex]::Escape($serviceRunnerPath)) -Quiet)) {
        throw "service task XML does not reference the generated runner script"
    }

    foreach ($path in $scheduleRunnerPaths) {
        if (-not (Select-String -Path $path -Pattern ([regex]::Escape($installPath)) -Quiet)) {
            throw "schedule runner script does not reference the installed binary: $path"
        }
    }

    foreach ($taskName in $taskNames) {
        schtasks /Query /TN $taskName /V /FO LIST | Out-Null
        if ($LASTEXITCODE -ne 0) {
            throw "scheduled task query failed for $taskName"
        }
    }

    if ($installExitCode -eq 0) {
        & $installPath service status
        if ($LASTEXITCODE -ne 0) {
            throw "service status failed with exit code $LASTEXITCODE"
        }

        & $installPath schedule status
        if ($LASTEXITCODE -ne 0) {
            throw "schedule status failed with exit code $LASTEXITCODE"
        }
    }

    & $installPath uninstall --keep-path --purge-data
    if ($LASTEXITCODE -ne 0 -and -not $env:GITHUB_ACTIONS) {
        throw "uninstall failed with exit code $LASTEXITCODE"
    }

    Start-Sleep -Seconds 2

    foreach ($path in @($installPath, $serviceRunnerPath, $serviceXmlPath, $configPath) + $scheduleRunnerPaths) {
        if ((Test-Path $path) -and (-not $env:GITHUB_ACTIONS)) {
            throw "install artifact should have been removed: $path"
        }
    }
}
finally {
    Get-Process -Name "sysray" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

    if (Test-Path $installPath) {
        try {
            & $installPath schedule uninstall *> $null
        }
        catch {
        }

        try {
            & $installPath service uninstall *> $null
        }
        catch {
        }
    } else {
        try {
            & $Binary schedule uninstall *> $null
        }
        catch {
        }

        try {
            & $Binary service uninstall *> $null
        }
        catch {
        }
    }

    foreach ($taskName in $taskNames) {
        try {
            schtasks /Delete /TN $taskName /F *> $null
        }
        catch {
        }
    }

    $global:LASTEXITCODE = 0
    Remove-Item -Recurse -Force $root -ErrorAction SilentlyContinue
}
