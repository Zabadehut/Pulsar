# Changelog

All notable changes to this project will be documented in this file.

The format is inspired by Keep a Changelog, and this project aims to follow Semantic Versioning once public releases begin.

## [Unreleased]

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

### Changed

- Project version advanced to `0.4.0`
- The TUI footer now reads the package version directly from Cargo metadata
- `disk+` and detailed disk tables now surface filesystem, parentage, and stable-ref cues closer to an `lsblk`-style reading
- The shared reference catalog now covers volume kind and logical stack terminology for the new disk inventory view
- The `storage` preset is now presented as `io` in the TUI to avoid overlap with `disk+` and `inventory+`
- The header now distinguishes `index:on`, `index:off`, and active search terms instead of showing `index:off` whenever no query is active

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
