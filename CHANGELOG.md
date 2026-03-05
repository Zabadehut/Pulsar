# Changelog

All notable changes to this project will be documented in this file.

The format is inspired by Keep a Changelog, and this project aims to follow Semantic Versioning once public releases begin.

## [Unreleased]

## [0.5.9] - 2026-03-05

### Changed

- Release replay for the cross-OS metrics and TUI compatibility batch
- Project version advanced to `0.5.9`

## [0.5.8] - 2026-03-05

### Added

- Cross-OS capability-aware metric flags from platform readers to collectors and TUI rendering
- Portable `load 1/5/15` derived demand smoothing on Windows with explicit `native` vs `derived` source semantics

### Changed

- TUI metric cards now show `n/a` instead of misleading zero values when an OS does not expose a given metric family
- Gauge warning style now follows the active theme instead of using a hard-coded color
- macOS command collection now forces stable `C` locale parsing and uses fallback absolute command paths for service contexts
- Linux schedule timers now use portable `OnCalendar=*-*-* HH:MM:SS` expressions
- Project version advanced to `0.5.8`

## [0.5.7] - 2026-03-04

### Added

- Real terminal-derived README screenshots for `overview`, `pressure+`, `network+`, `jvm+`, and `disk+`
- Cross-OS disk identity hints for structure, protocol, and media
- Deeper disk readability in standard and expert TUI views
- Cross-OS disk inventory collection with parent/child topology, filesystem, stable refs, model, serial, transport, and mount metadata
- Richer `/inventory` payloads for disk topology consumers and operators
- New `inventory+` expert TUI view for local disk tree and logical stack reading
- Linux `sysfs` enrichment for scheduler, rotational/removable/read-only flags, and holder/slave links
- Real README capture for `inventory+`
- Linux remote filesystem inventory entries for `NFS` / `SMB` / `SSHFS`-style mounts
- macOS and Windows remote filesystem inventory entries when the OS exposes network share mappings
- Linux optional enrichers for `dmsetup`, `mdadm`, and `btrfs filesystem show` when available
- CSV and Prometheus export of disk inventory categories and relation counts
- `sysray install` now bootstraps the stable binary path plus native service and recurring schedule setup
- Cross-platform `sysray uninstall` command to remove native automation, the stable installed binary, and the managed `PATH` entry
- CI validation coverage for the full `install` -> `uninstall` bootstrap flow on Unix and Windows

### Changed

- Project version advanced to `0.5.7`
- The TUI footer now reads the package version directly from Cargo metadata
- `disk+` and detailed disk tables now surface filesystem, parentage, and stable-ref cues closer to an `lsblk`-style reading
- The shared reference catalog now covers volume kind and logical stack terminology for the new disk inventory view
- The `storage` preset is now presented as `io` in the TUI to avoid overlap with `disk+` and `inventory+`
- The header now distinguishes `index:on`, `index:off`, and active search terms instead of showing `index:off` whenever no query is active
- Linux repo install automation now reinstalls the recurring native schedule by default
- Install and validation docs now describe the automatic bootstrap and teardown flows explicitly

## [0.4.0] - 2026-03-03

### Added

- Real `pressure+` screenshot in the README
- Cross-platform disk identity hints for structure, protocol, and media
- Richer disk tables in standard and expert views

### Changed

- Disk observability is now easier to read across Linux, macOS, and Windows with portable heuristics

## [0.3.0] - 2026-03-03

### Added

- Real screenshot generation pipeline for TUI README assets
- Deeper `pressure+` and `disk+` expert diagnostics
- Additional `jvm+` and `disk+` screenshots in the README

### Changed

- Expert local diagnostics now have deeper parity across pressure, network, JVM, and disk domains

## [0.2.0] - 2026-03-03

### Added

- Cross-platform `platform/` abstraction layer with Linux implementation and macOS/Windows stubs
- Scheduler-integrated computed metrics pipeline
- Backward-compatible snapshot deserialization via `serde(default)`
- Service management scaffolding for Linux, macOS, and Windows
- Initial public-facing `README.md`

### Changed

- Collectors now route OS-specific logic through `src/platform/`
- Configuration extended with pipeline settings and thresholds
- Snapshot schema extended with computed metrics and alerts

### Notes

- Linux is currently the only platform with real collector implementations
- macOS and Windows remain architecture-ready but incomplete
