# Pulsar Community Cheatsheet

This sheet is the fast reference for Pulsar Core / Community usage.

## Positioning

- open source local observability engine
- Linux-first depth, with baseline macOS and Windows support
- single binary workflow
- local-first operation: TUI, snapshots, exporters, recording

## What Community Includes

- CPU, memory, disk, network, process, and system metrics
- TUI
- JSON, CSV, and Prometheus text export
- local recording to `.jsonl`
- shared pipeline for derived metrics and threshold alerts
- Linux-specific cgroup v2 and PSI depth when available
- service install scaffolding for `systemd`, `launchd`, and Task Scheduler

## What Community Does Not Claim Yet

- parity-complete macOS support
- parity-complete Windows support
- multi-host orchestration
- enterprise governance controls
- retention platform
- SLA-backed operational guarantees

## Daily Commands

These commands exist in the current CLI. Use `pulsar --help` for the live command list.

```bash
# TUI
pulsar

# one-shot snapshot
pulsar snapshot --format json
pulsar snapshot --format prometheus

# continuous local recording
pulsar record --interval 5s --output ./captures --rotate hourly --keep-files 24

# top processes
pulsar top --sort cpu --limit 20

# service integration
pulsar service install
pulsar service status
pulsar service uninstall
```

## Recommended Community Workflows

### Local investigation

```bash
pulsar snapshot --format json
pulsar top --sort cpu --limit 20
```

### Lightweight capture session

```bash
mkdir -p ./captures
pulsar record --interval 5s --output ./captures --rotate hourly --keep-files 24
```

### Linux user service

```bash
./scripts/install-linux-user.sh
systemctl --user status pulsar.service
```

## Community Packaging Rules

- keep the binary self-contained
- keep the default workflow local and operator-friendly
- do not move core host observability behind enterprise gating
- describe macOS and Windows as baseline coverage until parity is real

## Recording Rotation

Raw rotation and raw retention are implemented in the current CLI. Only archive compression is still planned.

### Recommended policy

- rotate hourly for dense troubleshooting captures
- rotate daily for long-running baseline captures
- force rotation when a file exceeds a max size such as `256MB`, `512MB`, or `1GB`
- keep only a bounded number of raw files locally
- keep compression as a later layer so the write path stays cheap

### Proposed CLI shape

```bash
pulsar record \
  --interval 5s \
  --output ./captures \
  --rotate hourly \
  --max-file-size-mb 512 \
  --keep-files 168
```

### Current semantics

- `--rotate never|hourly|daily`
- `--max-file-size-mb <MB>`
- `--keep-files <count>` keeps the latest raw `.jsonl` segments

## Planned Rust-Only Zip Command

This is the portable command shape to aim for if compression is added without shelling out to OS tools. It is not part of the current CLI help.

```bash
pulsar archive zip \
  --input ./captures/pulsar_20260303_140000.jsonl \
  --output ./captures/pulsar_20260303_140000.jsonl.zip
```

Design constraints:

- implemented in Rust only
- no dependency on `zip`, `tar`, `powershell Compress-Archive`, or platform-specific binaries
- deterministic output layout
- safe for Linux, macOS, and Windows CI

## Naming Convention Suggestion

Use predictable file names:

```text
pulsar_active.jsonl
pulsar_YYYYMMDD_HH0000.jsonl
pulsar_YYYYMMDD_HH0000.jsonl.zip
```

## Best-Fit Audience

- developers
- SREs needing a local host tool
- operators wanting a modern replacement for older single-host monitors
