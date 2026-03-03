# Sysray Enterprise Cheatsheet

This sheet captures the intended enterprise posture for Sysray without overstating current implementation.

## Positioning

- enterprise builds on top of Sysray Core
- enterprise means operational guarantees and governance first
- enterprise is not a synonym for "more metrics"

## Enterprise Scope

- hardened deployment model
- version and support policy
- long-term compatibility guarantees
- RBAC
- SSO
- auditability
- secure configuration handling
- release process maturity
- support workflows
- SLA-oriented operations

## What Enterprise Should Add

- stronger release discipline
- controlled rollout and upgrade policy
- supportable packaging and installation contracts
- audit-ready configuration and access model
- documented compatibility windows
- retention and operational governance once multi-host exists
- fleet-level correlation and policy once multi-host exists

## What Enterprise Should Not Replace

- core host collectors
- TUI and local workflows
- expert single-host diagnostics
- baseline exporters
- cross-platform baseline support
- Linux-first depth already promised to the community

## Current Honest Status

- enterprise controls are planned, not implemented
- distributed retention and orchestration are not implemented
- current repo state is still V1/V2 foundation work
- local expert analysis should be treated as Core scope, not as Enterprise scope

## Enterprise Deployment Expectations

When enterprise mode exists, it should provide:

- signed artifacts
- versioned upgrade guidance
- supportable config migration rules
- rollback procedures
- documented runner and OS support policy
- installation validation per OS

## Enterprise Recording And Retention Policy

This is the recommended target shape for enterprise-grade local recording.

### Rotation policy

- hourly rotation on high-frequency nodes
- daily rotation on standard nodes
- emergency rotation on size threshold breach
- separate retention policy for raw `.jsonl` versus compressed archives

### Compression policy

- compress rotated segments only
- keep the active segment writable and uncompressed
- prefer deterministic archive output for supportability
- enforce a maximum archive size budget per host

### Example target policy

```text
interval: 5s
rotation: hourly
max_file_size: 512MB
compression: zip
retention_local_archives: 7d
retention_local_raw: active-only
```

## Enterprise CLI Surface

Raw rotation, raw retention, and closed-segment zip compression are already present in the CLI. Only the standalone archive command remains planned.

```bash
sysray record \
  --interval 5s \
  --output /var/lib/sysray/captures \
  --rotate hourly \
  --max-file-size-mb 512 \
  --keep-files 168 \
  --compress zip

sysray archive zip \
  --input /var/lib/sysray/captures/pulsar_20260303_140000.jsonl \
  --output /var/lib/sysray/captures/pulsar_20260303_140000.jsonl.zip
```

## Rust-Only Compression Requirement

If Sysray adds archive compression, enterprise should require:

- Rust-native implementation
- no dependence on OS archive utilities
- same behavior on Linux, macOS, and Windows
- predictable exit codes and integrity validation
- CI coverage on all supported OSes

## Support Language To Use

Use:

- enterprise-grade operational model
- controlled rollout path
- supportability and governance roadmap
- fleet policy and shared history

Avoid even later:

- implying that local expert diagnostics require Enterprise

Avoid until implemented:

- enterprise-ready
- fully hardened
- production-certified on every OS
- infinite retention scale
