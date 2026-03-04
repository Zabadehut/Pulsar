# Sysray Backlog

This backlog translates product vision into concrete execution priorities.

## P0: Publicly Credible Foundation

These items must be done before making stronger public claims.

- Expand regression coverage beyond the current Linux collector baseline
- Validate service and schedule install/uninstall/status flows on each OS under stricter scenarios
- Document the actual supported feature set and platform scope
- Add a repeatable benchmark harness for CPU, memory, and snapshot overhead
- Publish measured overhead instead of aspirational performance language
- Keep docs and CLI help aligned with implemented commands

## P1: V1 Linux Hardening

- Improve process accuracy and edge-case handling
- Validate collector behavior under high process counts
- Improve exporter coverage and output consistency
- Add schema compatibility tests for recorded snapshots
- Measure and reduce snapshot cloning overhead where needed

## P2: Real Cross-Platform Support

- Implement real macOS CPU, memory, disk, network, process, and system collectors
- Implement real Windows CPU, memory, disk, network, process, and system collectors
- Add per-OS capability tests
- Define parity expectations by metric family
- Add platform-specific service and schedule installation validation

## P3: Metrics Depth

- Improve JVM awareness beyond process name heuristics
- Add containers and cgroup v2 collectors
- Add PSI on Linux
- Add NUMA metrics
- Add deep per-thread analysis
- Add synthetic health indices
- Expand local alerts beyond threshold checks

## P4: Performance and Hardware Affinity

- Build a repeatable benchmark suite
- Profile allocation hotspots
- Reduce copies in scheduler and exporters
- Define overhead budgets for idle and busy hosts
- Document measured overhead instead of aspirational claims
- Evaluate direct OS APIs where they outperform current parsing paths

## P5: Distributed / Scale Architecture

- Define agent model
- Define transport protocol
- Define ingestion and aggregation boundaries
- Define storage model and retention policy
- Define backpressure and failure semantics
- Define bounded resource usage guarantees

Without these items, "scalable to infinity" should not be used in product messaging.

## P6: Enterprise Track

- Versioning and compatibility policy
- Secure configuration and secret handling
- AuthN / AuthZ model
- RBAC
- SSO
- Audit trail
- Hardened release and support process
- Packaging, signing, and upgrade guarantees

## P7: Messaging Guardrails

What can be said now:

- Linux foundation
- Rust implementation
- single binary
- extensible cross-platform architecture

What should wait:

- production-ready on every OS
- zero overhead
- enterprise-ready
- infinitely scalable
