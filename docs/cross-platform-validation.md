# Cross-Platform Validation

This document defines how Sysray should be validated across Linux, macOS, and Windows.

## Current Reality

Sysray can be validated locally on Linux from this repository today.

macOS and Windows validation should be considered incomplete unless one of these is true:

- the build is executed on a real `macos` or `windows` runner
- the required Rust targets and toolchains are installed locally and actually work for cross-builds

In practice, the most reliable validation path is GitHub Actions with native runners.

This repository now includes native CI coverage on:

- `ubuntu-latest`
- `macos-latest`
- `windows-latest`

Linux coverage is intentionally more explicit than a single runner:

- native build/test/smoke validation on multiple Ubuntu runner versions
- cross-target compilation check for `x86_64-unknown-linux-musl`

## Linux Local Validation

Recommended local commands:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo build
cargo test
```

Optional runtime checks:

```bash
cargo run -- snapshot --format json
cargo run -- snapshot --format prometheus
cargo run -- top --sort cpu --limit 20
```

## macOS and Windows Validation

### Preferred

Use GitHub Actions native runners:

- `ubuntu-latest`
- `macos-latest`
- `windows-latest`

This is the canonical validation path for cross-platform status.

Recommended workflow coverage:

- `cargo build --locked`
- `cargo test --locked`
- `cargo run --locked -- snapshot --format json`
- `cargo run --locked -- snapshot --format prometheus`
- Ubuntu-only linting and cross-target `cargo check`

### Local Cross-Build Attempt

If you want to try local cross-builds, first install the Rust targets:

```bash
rustup target add x86_64-apple-darwin
rustup target add x86_64-pc-windows-msvc
```

Then attempt:

```bash
cargo build --target x86_64-apple-darwin
cargo build --target x86_64-pc-windows-msvc
```

Important:

- a Rust target being installed does not guarantee a working full cross-link environment
- macOS cross-builds from Linux may still fail depending on linker/toolchain constraints
- Windows cross-builds may require additional tooling depending on the selected target

## What Counts as "Supported"

Sysray should only claim support for an OS level when:

- CI passes on that OS
- core commands build successfully
- runtime behavior is validated for the relevant collectors
- service integration is validated on that OS

Architecture-only support does not count as full support.

## Linux Distribution Scope

Sysray targets the Linux OS surface, not an explicit guarantee for every distribution flavor.

That means support depends on layers:

- collector support: relies mainly on standard Linux kernel interfaces such as `/proc`, `/proc/net`, `/proc/diskstats`, `/proc/meminfo`, `/proc/loadavg`, `statvfs`, and cgroup/PSI files when present
- build support: depends on the Rust target and the local toolchain working on that distribution
- service support: currently assumes `systemd` for the bundled install flow and user service management

Practical interpretation:

- Debian/Ubuntu/Fedora/Arch-class systems with `systemd` are the expected first-class Linux environments
- other Linux distributions may build and run successfully, but should be treated as compatible-by-validation rather than compatible-by-claim
- non-`systemd` environments are runtime-capable, but their service integration should be considered manual unless a dedicated installer is added

### Linux Environment Tiers

Use these tiers when describing Linux support:

- `First-class`: mainstream glibc distributions with standard `/proc` support and `systemd`, validated directly in CI and expected to support the bundled service flow
- `Compatible-by-validation`: Linux environments that expose the required kernel interfaces and pass local or CI validation, even if they are not part of the default release messaging
- `Best-effort`: non-`systemd`, stripped-down, or otherwise unusual Linux environments where the binary may run, but service integration or some collectors may require manual adaptation

## Current Interpretation

At the current stage:

- Linux: validated locally and in native CI
- macOS: baseline host collectors implemented and validated in native CI; broader parity still pending
- Windows: baseline host collectors implemented and validated in native CI; broader parity still pending
