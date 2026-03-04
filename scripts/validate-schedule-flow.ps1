param(
    [string]$Binary = "target/debug/sysray.exe"
)

$ErrorActionPreference = "Stop"
$Binary = (Resolve-Path $Binary).Path

$root = Join-Path $env:RUNNER_TEMP ("sysray-schedule-validation-" + [guid]::NewGuid().ToString())
$env:APPDATA = Join-Path $root "AppData\Roaming"
New-Item -ItemType Directory -Force -Path $env:APPDATA | Out-Null

$appDir = Join-Path $env:APPDATA "Sysray"
$scheduleDir = Join-Path $appDir "schedule"
$configPath = Join-Path $appDir "sysray.toml"
$runnerPaths = @(
    (Join-Path $scheduleDir "snapshot.cmd"),
    (Join-Path $scheduleDir "prune.cmd"),
    (Join-Path $scheduleDir "archive.cmd")
)
$taskNames = @("Sysray Snapshot", "Sysray Prune", "Sysray Archive")

try {
    & $Binary schedule install
    $installExitCode = $LASTEXITCODE
    if ($installExitCode -ne 0 -and -not $env:GITHUB_ACTIONS) {
        throw "schedule install failed with exit code $LASTEXITCODE"
    }

    if (-not (Test-Path $configPath)) {
        throw "missing expected schedule config: $configPath"
    }

    foreach ($path in $runnerPaths) {
        if (-not (Test-Path $path)) {
            throw "missing expected schedule artifact: $path"
        }

        if (-not (Select-String -Path $path -Pattern ([regex]::Escape($Binary)) -Quiet)) {
            throw "runner script does not reference the built binary: $path"
        }
    }

    if ($installExitCode -eq 0) {
        & $Binary schedule status
        if ($LASTEXITCODE -ne 0) {
            throw "schedule status failed with exit code $LASTEXITCODE"
        }
    }

    foreach ($taskName in $taskNames) {
        schtasks /Query /TN $taskName /V /FO LIST | Out-Null
        if ($LASTEXITCODE -ne 0) {
            throw "scheduled task query failed for $taskName"
        }
    }

    & $Binary schedule uninstall
    if ($LASTEXITCODE -ne 0 -and -not $env:GITHUB_ACTIONS) {
        throw "schedule uninstall failed with exit code $LASTEXITCODE"
    }

    foreach ($path in $runnerPaths) {
        if ((Test-Path $path) -and (-not $env:GITHUB_ACTIONS)) {
            throw "schedule artifact should have been removed: $path"
        }
    }
}
finally {
    try {
        & $Binary schedule uninstall *> $null
    }
    catch {
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
