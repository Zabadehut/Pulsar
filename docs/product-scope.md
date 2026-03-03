# Sysray Product Scope

This document defines what belongs to V1, V2, V3, and future enterprise scope.

The goal is to keep product messaging aligned with implementation reality.

## Principles

- V1 must be useful on its own
- Cross-platform claims must follow real implementation, not only architecture
- Community/core value must remain meaningful on its own
- Enterprise scope must not leak into V1 unless it directly improves the core
- Performance claims must be backed by measurement

## Community Core

Goal: keep the open source/community edition credible as a standalone local observability engine.

Included:

- host collectors and exporters
- TUI and local workflows
- expert local diagnostics in TUI/CLI/API for advanced operators
- Linux-specific depth such as cgroup v2 and PSI
- documented cross-platform baseline support with explicit per-OS caveats
- replay, alerting, and usability improvements that benefit all users

Community scope should remain the main place where core system observability gets better.
Enterprise scope should build on top of this base with governance and operational guarantees, not by hollowing out the core metric surface.
That includes expert workflows: pressure triage, network diagnosis, JVM-focused inspection, disk-contention analysis, and similar local deep-dive views belong in Core when they run on one host from one binary.

## V1 Foundation

Goal: a credible local observability binary with real Linux value, usable baseline host coverage on macOS and Windows, and a clean cross-platform architecture.

Included:

- CPU metrics
- Memory metrics
- Disk metrics
- Network metrics
- Process metrics
- System metadata
- TUI
- JSON export
- CSV export
- Prometheus text export
- Local recording
- Backward-compatible snapshot schema
- Computed metrics pipeline
- Service installation scaffolding
- Config-driven runtime behavior
- Explicit documentation of per-OS capability gaps

V1 quality targets:

- reliable Linux behavior
- honest per-OS product claims in docs, README, and CI
- no scattered OS-specific logic in collectors
- basic CLI usability
- schema evolution without breaking replay
- buildable on Linux, macOS, and Windows

Explicitly not required for V1:

- parity-complete macOS support
- parity-complete Windows support
- distributed architecture
- enterprise auth and governance
- eBPF
- cgroup v2 depth
- PSI
- anomaly detection beyond simple thresholding

## V2 Depth

Goal: close the remaining parity gaps after the V1 baseline and add richer system/application signals.

Included:

- parity-complete macOS collectors and runtime validation
- parity-complete Windows collectors and runtime validation
- stronger JVM awareness
- container and cgroup v2 metrics
- replay mode
- richer alerts
- more complete TUI workflows
- specialist local operator views and deeper single-host diagnostics
- first performance benchmark suite

Optional V2 candidates:

- PSI
- deep per-thread analysis
- IPC metrics
- synthetic health indices

## V3 Platform

Goal: evolve from local host observability to a larger observability platform.

Included:

- multi-host architecture
- transport and aggregation model
- retention model
- web/dashboard surface
- API hardening
- plugin story
- richer application correlation
- fleet-level expert workflows built on top of local diagnostics

Likely additions:

- eBPF on Linux
- anomaly detection engine
- correlation engine
- fleet-level health views

## Enterprise Scope

Goal: paid or enterprise-grade operational guarantees and governance.

Included:

- hardened deployment model
- version/support policy
- long-term compatibility guarantees
- RBAC
- SSO
- auditability
- secure configuration handling
- release process maturity
- support workflows
- SLA-oriented operations
- multi-host policy, access control, and shared operational history

Enterprise scope is not a synonym for "more metrics" or "expert mode".
It primarily means operational guarantees, governance, supportability, controlled rollout, and cross-host operational control.

Explicit separation:

- Core/expert: local deep diagnostics, richer TUI modes, per-host advanced analysis
- Enterprise: RBAC, SSO, fleet policy, retention governance, multi-host correlation, auditability

## Infinite Scale Clarification

Sysray is not "infinitely scalable" in V1 or V2.

That phrase only becomes meaningful once the product has:

- an agent model
- a transport layer
- aggregation
- storage
- multi-tenant isolation
- retention
- backpressure
- bounded resource usage guarantees

Until then, Sysray should be described as a local observability engine with a path toward distributed operation.
