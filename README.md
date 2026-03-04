# Sysray

**Modern host diagnostics in one binary.**

Sysray is a system observability engine written in Rust.
It is being built as a modern replacement for legacy local monitoring tools such as NMON: single binary, low overhead, cross-platform by design, and extensible from day one.

Current workspace version: `0.5.4`

## Positioning

Sysray is not presented as a finished enterprise platform today.
What exists already is a serious technical foundation with working Linux collectors, a TUI, JSON/CSV/Prometheus exporters, a post-collection pipeline, and a cross-platform abstraction layer that is ready to receive real macOS and Windows implementations.

What Sysray aims to become:

- A fast local observability binary for Linux, macOS, and Windows
- A strong open core foundation for future Pro, Cloud, and enterprise offerings
- A modern Rust-native replacement for aging system tools

Product boundary:

- Sysray Core should include deep local diagnostics for advanced operators
- Sysray Enterprise should add governance, fleet-level control, and shared history
- Enterprise is not where local expert analysis gets hidden

## Current Status

Current stage: early V1 foundation.

What is working now:

- Linux collectors for CPU, memory, disk, network, process, and system metrics
- Linux-specific cgroup v2 and PSI metrics in snapshots/exporters when available
- Partial real macOS and Windows support for baseline host collection paths
- Native CI validation on Linux, macOS, and Windows runners
- Expanded Linux CI coverage across multiple Ubuntu runner versions plus a `musl` target check
- Interactive TUI mode
- One-shot snapshot export in JSON, CSV, and Prometheus text format
- Local recording to `.jsonl` with built-in rotation and raw retention controls
- Post-collection computed metrics pipeline
- Cross-platform platform layer with Linux implementation and macOS/Windows stubs
- Service management scaffolding for `systemd`, `launchd`, and Windows Task Scheduler
- Shared technical reference catalog exposed in TUI, API, and CLI explain mode
- TUI depth can continue to grow with expert local diagnostics without becoming an enterprise-only feature

What is not finished yet:

- Real macOS collectors
- Real Windows collectors
- Full public launch assets: CI, tests, docs, changelog, release automation
- Enterprise features such as RBAC, SSO, retention, multi-host orchestration, SLA-level hardening
- Distributed scalability architecture

## What We Can Honestly Claim Today

- Written in Rust
- Single binary application
- Apache 2.0 licensed
- Linux-first implementation with explicit cross-platform architecture
- Partial host-metric parity on macOS and Windows, with deeper collectors still pending
- Low-level system access on Linux with minimal abstraction overhead
- Forward-compatible metric schema via `serde(default)`

## What We Do Not Claim Yet

- Production-ready on all platforms
- Zero overhead
- Enterprise-ready
- Infinitely scalable
- Full macOS and Windows support

## Why Sysray

- Clean internal architecture: collectors, scheduler, exporters, pipeline, platform layer
- Rust performance and memory safety
- Backward-compatible snapshot schema
- Clear path from local observability tool to larger platform
- Honest product posture: strong foundation first, marketing claims second

## Commands

Use `sysray --help` for the live CLI and `sysray <command> --help` for per-command details.
Built-in recording rotation, retention, and closed-segment `zip` compression are now in the CLI. The standalone archive command is still planned and documented in [`docs/help.md`](docs/help.md).
The TUI now exposes a technical reference pane with `/` for search, `?` for the index, `l` for the live logs pane, `L` to add watched paths, `e` for an error-focused logs view, `1` to `6` for operator presets, `7` to `0` for expert local diagnostics, `g` for `inventory+`, `v` for detail density, and `i` to switch `fr`/`en`. The Linux panel toggle now lives on `k` so `l` stays dedicated to logs. The index follows the active language and specialist view context. In practice, `io` is the broad storage preset, `disk+` is the contention/performance drill-down, and `inventory+` is the local topology / stack / refs view. The logs pane now follows active files incrementally with offset tracking and rotation/truncation detection.

## Screens

Overview TUI:

![Sysray TUI overview](docs/screenshots/tui-overview-real.svg)

Expert network view:

![Sysray expert network diagnostics](docs/screenshots/tui-expert-network-real.svg)

Expert pressure view:

![Sysray expert pressure diagnostics](docs/screenshots/tui-expert-pressure-real.svg)

Expert JVM view:

![Sysray expert JVM diagnostics](docs/screenshots/tui-expert-jvm-real.svg)

Expert disk view:

![Sysray expert disk diagnostics](docs/screenshots/tui-expert-disk-real.svg)

Disk inventory view:

![Sysray expert disk inventory](docs/screenshots/tui-expert-inventory-real.svg)

These captures are generated from real terminal frames with:

```bash
bash scripts/capture-tui-screens.sh docs/screenshots
```

```bash
# Interactive TUI
sysray

# Install current binary to a stable user path
sysray install

# Install native recurring tasks
sysray schedule install

# One-shot snapshot
sysray snapshot --format json

# Continuous recording
sysray record --interval 5s --output ./captures --rotate hourly --keep-files 48 --compress zip

# HTTP server
sysray server --port 9090

# Top processes
sysray top --sort cpu --limit 20

# Watch one process
sysray watch --pid 1234

# Replay a recorded session
sysray replay ./captures/sysray_20260303_130000.jsonl

# Explain a technical term
sysray explain latency
sysray explain swap --lang en --audience beginner

# Built-in maintenance tasks
sysray maintenance daily-snapshot
sysray maintenance prune --retention-days 15
sysray maintenance archive --min-age-days 15 --max-age-days 60

# Service integration
sysray service install
sysray service status
sysray service uninstall
```

## Linux Install And Update

On Linux, install the binary to a stable path instead of running from `target/debug/` or `target/release/`.

Current Linux packaging assumptions:

- build and runtime target the generic Linux kernel surface, not a named distribution
- service installation is currently `systemd`-oriented
- non-`systemd` distributions can still run the binary, but the bundled service installer is not a universal Linux service manager

Recommended user-level install from an existing binary:

```bash
sysray install
```

This command:

- installs the current executable to `~/.local/bin/sysray`
- reinstalls the user service so it points to that stable binary path
- warns when `~/.local/bin` is not in `PATH`

For native recurring automation without `crontab` scripts:

```bash
sysray schedule install
```

This installs OS-native recurring jobs for:

- snapshot append every 5 minutes
- prune once per day at `02:00`
- archive once per day at `02:30`

Workspace helper for Linux developers:

```bash
./scripts/install-linux-user.sh
```

This script:

- builds the release bundle if `dist/` is missing
- installs the release binary to `~/.local/bin/sysray`
- reinstalls the user service so it points to that stable binary path

For a fresh rebuild from the current workspace before install:

```bash
./scripts/install-linux-user.sh --force-build
```

For a binary-only update without touching the service:

```bash
./scripts/install-linux-user.sh --no-service
```

Manual update flow:

```bash
./scripts/build-complete.sh
install -m 755 dist/sysray-<version>-<target>/standalone/sysray ~/.local/bin/sysray
~/.local/bin/sysray service uninstall
~/.local/bin/sysray service install
systemctl --user status sysray.service
```

Important for a blank machine:

- `cargo install sysray` is a developer bootstrap path, not the recommended end-user install path
- it requires a Rust toolchain and usually leaves the binary under `~/.cargo/bin`
- after a release-binary install, prefer `sysray install` plus `sysray service install`
- on Linux, package formats such as `.rpm` or a standalone release tarball are a better first-run story than requiring Rust
- real `systemd --user` runtime validation should be done on a dedicated Linux environment with a proper user session, not faked inside a generic hosted CI runner

## Configuration

Example configuration is available in [`config/sysray.toml.example`](config/sysray.toml.example).

Sysray uses:

- Config file: `sysray.toml`
- Env var: `SYSRAY_CONFIG`
- Binary name: `sysray`

Recording defaults can now also be centralized in the config file:

- `record.interval_secs`
- `record.output`
- `record.rotate`
- `record.max_file_size_mb`
- `record.keep_files`
- `record.compress`

TUI defaults can also be centralized:

- `tui.theme`
- `tui.locale`

## Architecture

```text
src/
├── main.rs
├── cli.rs
├── config.rs
├── service.rs
├── engine/
├── collectors/
├── exporters/
├── pipeline/
├── platform/
├── tui/
└── api/
```

Key design points:

- `collectors/`: metric gathering and snapshot population
- `platform/`: OS-specific boundary
- `pipeline/`: derived metrics and alerts
- `engine/`: scheduling and runtime orchestration
- `exporters/`: output formats
- `service.rs`: OS service integration

Detailed planning documents:

- [`docs/help.md`](docs/help.md)
- [`docs/benchmarking.md`](docs/benchmarking.md)
- [`docs/reference-architecture.md`](docs/reference-architecture.md)
- [`docs/product-scope.md`](docs/product-scope.md)
- [`docs/metrics-matrix.md`](docs/metrics-matrix.md)
- [`docs/metrics-checklist.md`](docs/metrics-checklist.md)
- [`docs/community-cheatsheet.md`](docs/community-cheatsheet.md)
- [`docs/enterprise-cheatsheet.md`](docs/enterprise-cheatsheet.md)
- [`docs/cross-os-cheatsheet.md`](docs/cross-os-cheatsheet.md)
- [`docs/backlog.md`](docs/backlog.md)
- [`docs/execution-roadmap.md`](docs/execution-roadmap.md)
- [`docs/cross-platform-validation.md`](docs/cross-platform-validation.md)

## Cross-Platform Strategy

Sysray is cross-platform by architecture today, not yet by implementation completeness.

That distinction matters:

- Linux: real collector implementation
- macOS: baseline CPU, memory, disk, network, process, and system collection paths implemented, broader parity pending
- Windows: baseline CPU, memory, disk, network, process, and system collection paths implemented, broader parity pending

Validation is tracked separately from implementation:

- Linux: validated locally and in CI
- macOS: baseline implementation present and validated in native CI, with broader parity still pending
- Windows: baseline implementation present and validated in native CI, with broader parity still pending

Linux support should also be read carefully:

- the code targets Linux generally, not an explicit matrix of every distribution
- collector compatibility primarily depends on standard kernel interfaces such as `/proc`
- service management compatibility depends on the init system, and only `systemd` is bundled today

The goal is that adding or improving an OS implementation happens primarily inside `src/platform/`, not by scattering conditional compilation across collectors.

Validation policy is documented in [`docs/cross-platform-validation.md`](docs/cross-platform-validation.md).

## Public Roadmap

### V1

- Stabilize Linux collectors
- Add tests and CI
- Finish public repository assets
- Improve TUI polish and exporter coverage
- Validate service installation flows

### V2

- Real macOS support
- Real Windows support
- JVM and container depth
- Replay and alerting improvements

### V3

- Multi-host architecture
- Web and API ecosystem
- Advanced enterprise features

For a stricter breakdown of what belongs to V1, V2, V3, and enterprise scope, see [`docs/product-scope.md`](docs/product-scope.md).

## Community And Enterprise

Sysray Core is intended to stay community-accessible and open source.
That community/core layer should keep getting the main local observability primitives: collectors, TUI, exporters, replay, Linux depth, and cross-platform baseline support.

The enterprise track is a separate concern:

- governance and access control
- supportability and release discipline
- compatibility guarantees
- hardened deployment and auditability

Enterprise should add operational guarantees around the core, not replace the community roadmap or hide core host observability behind a paywall.

## Build

```bash
cargo build
```

## Benchmarking

Use the Linux benchmark harness to measure local overhead before making performance claims:

```bash
./scripts/benchmark-overhead-linux.sh --duration 30 --interval 5 --snapshot-count 25
```

Results are written under `.benchmarks/<UTC_RUN_ID>/` with both `summary.md` and `summary.csv`.

On the March 3, 2026 Rocky Linux baseline run (`30s`, `5s`, `25` snapshots), `sysray record` measured about `0.35%` average CPU and `~13 MB` RSS, while repeated one-shot JSON snapshots measured about `0.83%` average CPU with a similar peak RSS. See [`docs/benchmarking.md`](docs/benchmarking.md).

`cargo build` only rebuilds the workspace binary in `target/debug/`.
It does not update the user service binary installed in `~/.local/bin/sysray`.

## Dev Workstation Update Flow

For this local Linux developer setup, the user service runs the installed binary path, not the build output from `target/debug/`.

Current service runner:

- `~/.config/systemd/user/sysray.service` -> `~/.local/share/sysray/sysray-service.sh`
- `~/.local/share/sysray/sysray-service.sh` -> `~/.local/bin/sysray`

Recommended flow after code changes:

```bash
./scripts/redeploy-dev-user-service.sh
```

This script runs:

- `cargo fmt --all`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- `cargo build`
- `./scripts/install-linux-user.sh --force-build`
- `systemctl --user restart sysray.service`
- `systemctl --user status sysray.service --no-pager`
- `journalctl --user -u sysray.service -n 50 --no-pager`

If you only run `cargo build`, you have rebuilt the dev binary in `target/debug/`, but the service still uses the installed binary in `~/.local/bin/sysray` until you reinstall it.

If you want the manual flow instead of the helper script:

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
cargo build
./scripts/install-linux-user.sh --force-build
systemctl --user restart sysray.service
systemctl --user status sysray.service --no-pager
journalctl --user -u sysray.service -n 50 --no-pager
```

## Complete Build

Run the full local release flow in one command:

```bash
./scripts/build-complete.sh
```

This command runs:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- `cargo build --release`

It then creates:

- `dist/sysray-<version>-<target>/standalone/`
- `dist/sysray-<version>-<target>/install-prereqs/`
- `dist/sysray-<version>-<target>.tar.gz`
- `dist/sysray-<version>-<target>.rpm` on Linux targets when `rpmbuild` is available
- `dist/sysray-<version>-<target>.zip` on Windows targets
- `dist/sysray-<version>-<target>.exe` on Windows targets
- `dist/sysray-<version>-<target>.SHA256SUMS`
- `dist/sysray-<version>-<target>.SHA256SUMS.asc` when `gpg` is available and `SYSRAY_GPG_KEY_ID` is set

CI runs this same script on Linux, macOS, and Windows and uploads the generated `dist/` artifacts automatically.

Release publication on GitHub is triggered by pushing a `v*` tag and requires the signing secrets documented in `docs/release-process.md`.

## License

Apache License 2.0.

Sysray Core is intended to remain open source.

## Author

Kevin Vanden-Brande
