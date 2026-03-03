# Pulsar Cross-OS Cheatsheet

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

These commands reflect the current CLI. Planned rotation/archive commands are documented later and are not exposed by `pulsar --help` yet.

### Linux

```bash
pulsar
pulsar snapshot --format json
pulsar record --interval 5s --output ./captures
pulsar service install
```

### macOS

```bash
pulsar
pulsar snapshot --format json
pulsar record --interval 5s --output ./captures
pulsar service install
launchctl list com.zabadehut.pulsar
```

### Windows

```powershell
pulsar.exe
pulsar.exe snapshot --format json
pulsar.exe record --interval 5s --output .\captures
pulsar.exe service install
schtasks /Query /TN Pulsar /V /FO LIST
```

## Service Model By OS

| OS | Service mechanism | Current status |
|---|---|---|
| Linux | `systemd --user` | usable when `systemd` user bus is available |
| macOS | `launchd` user agent | usable, validated in native CI |
| Windows | Task Scheduler | usable, validated in native CI |

## Recording Layout Recommendation

Use one directory per host:

```text
captures/
  pulsar_active.jsonl
  pulsar_20260303_130000.jsonl.zip
  pulsar_20260303_140000.jsonl.zip
```

## Planned Rotation Matrix

This is a proposed design, not implemented CLI today.

| Usage pattern | Interval | Rotation | Max file size | Compression |
|---|---|---|---|---|
| Short incident capture | `1s` to `5s` | hourly | `256MB` | zip |
| Standard baseline | `5s` to `15s` | daily | `512MB` | zip |
| Long-running low-noise host | `30s` to `60s` | daily | `1GB` | zip |

## Planned Portable Commands

These command shapes are roadmap targets only. They are not in the current CLI help.

### Cross-OS recording

```bash
pulsar record \
  --interval 5s \
  --output ./captures \
  --rotate daily \
  --max-file-size 512MB \
  --compress zip
```

### Cross-OS archive compression

```bash
pulsar archive zip \
  --input ./captures/pulsar_20260303_140000.jsonl \
  --output ./captures/pulsar_20260303_140000.jsonl.zip
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
