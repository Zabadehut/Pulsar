# Sysray Cross-OS Cheatsheet

This sheet is the quick operational reference for Linux, macOS, and Windows.

## Current OS Posture

### Linux

- deepest implementation today
- validated locally and in CI
- bundled service path targets `systemd`
- Linux-specific cgroup v2 and PSI depth available when the host exposes it

### macOS

- baseline CPU, memory, disk, network, process, and system collection present
- process CPU, FD count, and process IO now have real collection paths
- validated in native CI
- parity is still not complete relative to Linux depth

### Windows

- baseline CPU, memory, disk, network, process, and system collection present
- process CPU hints, handle-count-as-FD-equivalent, process IO, and richer system metadata now exist
- validated in native CI
- parity is still not complete relative to Linux depth

## Safe Product Language

Use:

- cross-platform by architecture and baseline runtime
- Linux-first with native macOS and Windows baseline support
- parity still pending outside Linux

Avoid:

- full macOS support
- full Windows support
- identical metrics semantics across all OSes

## Core Commands By OS

These commands reflect the current CLI. Raw file rotation, retention, and closed-segment zip compression are implemented. A standalone archive command remains planned.

### Linux

```bash
sysray
sysray install
sysray schedule install
sysray snapshot --format json
sysray record --interval 5s --output ./captures --rotate hourly --keep-files 24 --compress zip
sysray service install
```

### macOS

```bash
sysray
sysray install
sysray schedule install
sysray snapshot --format json
sysray record --interval 5s --output ./captures --rotate daily --keep-files 14 --compress zip
sysray service install
launchctl list com.zabadehut.sysray
```

### Windows

```powershell
sysray.exe
sysray.exe install
sysray.exe schedule install
sysray.exe snapshot --format json
sysray.exe record --interval 5s --output .\captures --rotate daily --keep-files 14 --compress zip
sysray.exe service install
schtasks /Query /TN Sysray /V /FO LIST
```

## Service Model By OS

| OS | Service mechanism | Current status |
|---|---|---|
| Linux | `systemd --user` | usable when `systemd` user bus is available |
| macOS | `launchd` user agent | usable, validated in native CI |
| Windows | Task Scheduler | usable, validated in native CI |

## Native Schedule Model By OS

| OS | Recurring schedule mechanism | Current status |
|---|---|---|
| Linux | `systemd --user` timers | usable when `systemd` user bus is available |
| macOS | `launchd` LaunchAgents | usable |
| Windows | Task Scheduler recurring tasks | usable |

## Recording Layout Recommendation

Use one directory per host:

```text
captures/
  sysray_active.jsonl
  sysray_20260303_130000.jsonl.zip
  sysray_20260303_140000.jsonl.zip
```

## Raw Rotation Matrix

This matrix matches the current recording CLI.

| Usage pattern | Interval | Rotation | Max file size | Compression |
|---|---|---|---|---|
| Short incident capture | `1s` to `5s` | hourly | `256MB` | zip |
| Standard baseline | `5s` to `15s` | daily | `512MB` | zip |
| Long-running low-noise host | `30s` to `60s` | daily | `1GB` | zip |

## Planned Portable Commands

Only the standalone archive command is still a roadmap target. Recording rotation and closed-segment compression are already in the current CLI.

### Cross-OS recording

```bash
sysray record \
  --interval 5s \
  --output ./captures \
  --rotate daily \
  --max-file-size-mb 512 \
  --keep-files 14 \
  --compress zip
```

### Cross-OS archive compression

```bash
sysray archive zip \
  --input ./captures/sysray_20260303_140000.jsonl \
  --output ./captures/sysray_20260303_140000.jsonl.zip
```

## Compression Requirements

The archive path should be:

- implemented in Rust
- independent from OS shell tools
- valid on Linux, macOS, and Windows
- safe to invoke from CI and service contexts
- explicit about input, output, overwrite, and validation

## Validation Reference

Use native CI as the source of truth for cross-OS status.

Related docs:

- `docs/cross-platform-validation.md`
- `docs/metrics-matrix.md`
- `docs/metrics-checklist.md`
