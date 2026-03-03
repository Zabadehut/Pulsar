# Pulsar

**Your system. Always beating.**

Pulsar is a system observability engine written in Rust.
It is being built as a modern replacement for legacy local monitoring tools such as NMON: single binary, low overhead, cross-platform by design, and extensible from day one.

## Positioning

Pulsar is not presented as a finished enterprise platform today.
What exists already is a serious technical foundation with working Linux collectors, a TUI, JSON/CSV/Prometheus exporters, a post-collection pipeline, and a cross-platform abstraction layer that is ready to receive real macOS and Windows implementations.

What Pulsar aims to become:

- A fast local observability binary for Linux, macOS, and Windows
- A strong open core foundation for future Pro, Cloud, and enterprise offerings
- A modern Rust-native replacement for aging system tools

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

## Why Pulsar

- Clean internal architecture: collectors, scheduler, exporters, pipeline, platform layer
- Rust performance and memory safety
- Backward-compatible snapshot schema
- Clear path from local observability tool to larger platform
- Honest product posture: strong foundation first, marketing claims second

## Commands

Use `pulsar --help` for the live CLI and `pulsar <command> --help` for per-command details.
Built-in raw recording rotation and retention are now in the CLI. Zip archive compression is still planned and documented in [`docs/help.md`](docs/help.md).
The TUI now exposes a technical reference pane with `/` for search, `?` for the index, and `1` to `6` for operator presets.

```bash
# Interactive TUI
pulsar

# One-shot snapshot
pulsar snapshot --format json

# Continuous recording
pulsar record --interval 5s --output ./captures --rotate hourly --keep-files 48

# HTTP server
pulsar server --port 9090

# Top processes
pulsar top --sort cpu --limit 20

# Watch one process
pulsar watch --pid 1234

# Replay a recorded session
pulsar replay ./captures/pulsar_20260303_130000.jsonl

# Explain a technical term
pulsar explain latency
pulsar explain swap --lang en --audience beginner

# Service integration
pulsar service install
pulsar service status
pulsar service uninstall
```

## Linux Install And Update

On Linux, install the release binary to a stable path instead of running from `target/debug/` or `target/release/`.

Current Linux packaging assumptions:

- build and runtime target the generic Linux kernel surface, not a named distribution
- service installation is currently `systemd`-oriented
- non-`systemd` distributions can still run the binary, but the bundled service installer is not a universal Linux service manager

Recommended user-level install:

```bash
./scripts/install-linux-user.sh
```

This script:

- builds the release bundle if `dist/` is missing
- installs the release binary to `~/.local/bin/pulsar`
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
install -m 755 dist/pulsar-<version>-<target>/standalone/pulsar ~/.local/bin/pulsar
~/.local/bin/pulsar service uninstall
~/.local/bin/pulsar service install
systemctl --user status pulsar.service
```

## Configuration

Example configuration is available in [`config/pulsar.toml.example`](config/pulsar.toml.example).

Pulsar uses:

- Config file: `pulsar.toml`
- Env var: `PULSAR_CONFIG`
- Binary name: `pulsar`

Recording defaults can now also be centralized in the config file:

- `record.interval_secs`
- `record.output`
- `record.rotate`
- `record.max_file_size_mb`
- `record.keep_files`

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

Pulsar is cross-platform by architecture today, not yet by implementation completeness.

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

Pulsar Core is intended to stay community-accessible and open source.
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

On the March 3, 2026 Rocky Linux baseline run (`30s`, `5s`, `25` snapshots), `pulsar record` measured about `0.35%` average CPU and `~13 MB` RSS, while repeated one-shot JSON snapshots measured about `0.83%` average CPU with a similar peak RSS. See [`docs/benchmarking.md`](docs/benchmarking.md).

`cargo build` only rebuilds the workspace binary in `target/debug/`.
It does not update the user service binary installed in `~/.local/bin/pulsar`.

## Dev Workstation Update Flow

For this local Linux developer setup, the user service runs the installed binary path, not the build output from `target/debug/`.

Current service runner:

- `~/.config/systemd/user/pulsar.service` -> `~/.local/share/pulsar/pulsar-service.sh`
- `~/.local/share/pulsar/pulsar-service.sh` -> `~/.local/bin/pulsar`

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
- `systemctl --user restart pulsar.service`
- `systemctl --user status pulsar.service --no-pager`
- `journalctl --user -u pulsar.service -n 50 --no-pager`

If you only run `cargo build`, you have rebuilt the dev binary in `target/debug/`, but the service still uses the installed binary in `~/.local/bin/pulsar` until you reinstall it.

If you want the manual flow instead of the helper script:

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
cargo build
./scripts/install-linux-user.sh --force-build
systemctl --user restart pulsar.service
systemctl --user status pulsar.service --no-pager
journalctl --user -u pulsar.service -n 50 --no-pager
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

- `dist/pulsar-<version>-<target>/standalone/`
- `dist/pulsar-<version>-<target>/install-prereqs/`
- `dist/pulsar-<version>-<target>.tar.gz`
- `dist/pulsar-<version>-<target>.zip` on Windows targets
- `dist/pulsar-<version>-<target>.SHA256SUMS`
- `dist/pulsar-<version>-<target>.SHA256SUMS.asc` when `gpg` is available and `PULSAR_GPG_KEY_ID` is set

CI runs this same script on Linux, macOS, and Windows and uploads the generated `dist/` artifacts automatically.

Release publication on GitHub is triggered by pushing a `v*` tag and requires the signing secrets documented in `docs/release-process.md`.

## License

Apache License 2.0.

Pulsar Core is intended to remain open source.

## Author

Kevin Vanden-Brande
