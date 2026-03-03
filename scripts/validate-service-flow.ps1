param(
    [string]$Binary = "target/debug/sysray.exe"
)

$ErrorActionPreference = "Stop"
$Binary = (Resolve-Path $Binary).Path

$root = Join-Path $env:RUNNER_TEMP ("sysray-service-validation-" + [guid]::NewGuid().ToString())
$env:APPDATA = Join-Path $root "AppData\Roaming"
New-Item -ItemType Directory -Force -Path $env:APPDATA | Out-Null

$appDir = Join-Path $env:APPDATA "Sysray"
$runnerPath = Join-Path $appDir "sysray-service.cmd"
$xmlPath = Join-Path $appDir "sysray-task.xml"
$configPath = Join-Path $appDir "sysray.toml"

try {
    & $Binary service install
    $installExitCode = $LASTEXITCODE
    if ($installExitCode -ne 0 -and -not $env:GITHUB_ACTIONS) {
        throw "service install failed with exit code $LASTEXITCODE"
    }

    foreach ($path in @($runnerPath, $xmlPath, $configPath)) {
        if (-not (Test-Path $path)) {
            throw "missing expected service artifact: $path"
        }
    }

    if (-not $env:GITHUB_ACTIONS -and -not (Select-String -Path $runnerPath -Pattern ([regex]::Escape($Binary)) -Quiet)) {
        throw "runner script does not reference the built binary"
    }

    if (-not $env:GITHUB_ACTIONS -and -not (Select-String -Path $xmlPath -Pattern ([regex]::Escape($runnerPath)) -Quiet)) {
        throw "task XML does not reference the generated runner script"
    }

    if ($installExitCode -eq 0) {
        & $Binary service status
        if ($LASTEXITCODE -ne 0) {
            throw "service status failed with exit code $LASTEXITCODE"
        }
    }

    & $Binary service uninstall
    if ($LASTEXITCODE -ne 0 -and -not $env:GITHUB_ACTIONS) {
        throw "service uninstall failed with exit code $LASTEXITCODE"
    }

    foreach ($path in @($runnerPath, $xmlPath)) {
        if ((Test-Path $path) -and (-not $env:GITHUB_ACTIONS)) {
            throw "service artifact should have been removed: $path"
        }
    }
}
finally {
    & $Binary service uninstall *> $null
    Remove-Item -Recurse -Force $root -ErrorAction SilentlyContinue
}
