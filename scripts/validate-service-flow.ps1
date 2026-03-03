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
    if ($LASTEXITCODE -ne 0) {
        throw "service install failed with exit code $LASTEXITCODE"
    }

    foreach ($path in @($runnerPath, $xmlPath, $configPath)) {
        if (-not (Test-Path $path)) {
            throw "missing expected service artifact: $path"
        }
    }

    if (-not (Select-String -Path $runnerPath -Pattern ([regex]::Escape($Binary)) -Quiet)) {
        throw "runner script does not reference the built binary"
    }

    if (-not (Select-String -Path $xmlPath -Pattern ([regex]::Escape($runnerPath)) -Quiet)) {
        throw "task XML does not reference the generated runner script"
    }

    & $Binary service status
    if ($LASTEXITCODE -ne 0) {
        throw "service status failed with exit code $LASTEXITCODE"
    }

    & $Binary service uninstall
    if ($LASTEXITCODE -ne 0) {
        throw "service uninstall failed with exit code $LASTEXITCODE"
    }

    foreach ($path in @($runnerPath, $xmlPath)) {
        if (Test-Path $path) {
            throw "service artifact should have been removed: $path"
        }
    }
}
finally {
    & $Binary service uninstall *> $null
    Remove-Item -Recurse -Force $root -ErrorAction SilentlyContinue
}
