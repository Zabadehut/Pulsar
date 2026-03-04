# Sysray Execution Roadmap

This roadmap turns the metric checklist and backlog into a concrete delivery order.

It answers one question:

What should be built next, in what order, and why?

## Rules

- Finish credibility before chasing breadth
- Do not claim support before validation exists
- Do not add advanced metrics before core metrics are trustworthy
- Do not start distributed or enterprise work before single-host quality is real

## Phase 0: Credibility Baseline

Goal: make the current Linux-first foundation defensible.

Order:

1. Add benchmark harness for collection and snapshot overhead
2. Document measured overhead in public docs
3. Harden service validation on Linux and keep OS-specific constraints explicit
4. Expand regression coverage around stressed Linux hosts
5. Keep CLI help and docs aligned with implemented commands

Exit criteria:

- `cargo fmt`, `clippy`, `build`, and `test` pass
- Linux collectors have broader regression coverage
- backward compatibility stays tested
- measured overhead exists in documented form

What can be said after this phase:

- Sysray is a tested Linux observability foundation
- low-overhead goals are being measured, not guessed

## Phase 1: Complete V1 Linux Quality

Goal: close the most visible Linux metric gaps.

Order:

1. Implement real disk await / latency
2. Improve process accuracy under edge cases
3. Validate behavior on busy hosts with many processes/interfaces/disks
4. Improve exporter consistency and field coverage
5. Validate TUI behavior against real data

Exit criteria:

- Linux metric surface is internally consistent
- no known shallow placeholder remains inside claimed V1 Linux metrics
- top-level commands are stable on Linux

What can be said after this phase:

- Sysray delivers a credible Linux local-observability V1

## Phase 2: Real Cross-Platform Support

Goal: convert architecture-level cross-platform design into actual support.

Order:

1. Implement macOS system + CPU + memory
2. Implement macOS disk + network + process
3. Validate macOS runtime commands and service integration
4. Implement Windows system + CPU + memory
5. Implement Windows disk + network + process
6. Validate Windows runtime commands and service integration
7. Define platform parity expectations per metric family

Exit criteria:

- CI passes on Linux, macOS, and Windows
- core commands build and run on all three OS families
- platform limitations are documented explicitly

What can be said after this phase:

- Sysray is a real cross-platform local observability tool

## Phase 3: High-Value Differentiators

Goal: add the first set of metrics that clearly separate Sysray from legacy tools.

Order:

1. Improve JVM awareness
2. Deepen replay mode UX and operator workflow
3. Add cgroup v2 / container metrics on Linux
4. Add PSI on Linux
5. Add synthetic health indices
6. Expand alerts beyond simple threshold checks

Exit criteria:

- Sysray has at least two modern differentiators beyond legacy host monitoring
- replay and derived metrics are useful in real workflows

What can be said after this phase:

- Sysray is more than a local host monitor
- Sysray offers modern Linux-specific observability depth

## Phase 4: Performance and Hardware Affinity

Goal: make the performance story measurable and defensible.

Order:

1. Profile allocations and hot paths
2. Reduce scheduler and exporter copying where it matters
3. Define overhead budgets for idle and loaded systems
4. Benchmark against representative workloads
5. Document findings publicly

Exit criteria:

- overhead claims are backed by numbers
- performance regressions can be detected

What can be said after this phase:

- Sysray is performance-conscious with measured overhead

## Phase 5: Deep System and App Insight

Goal: extend visibility into advanced workloads and runtimes.

Order:

1. Add deep per-thread analysis
2. Add NUMA metrics
3. Add runtime-aware application signals beyond JVM
4. Evaluate IPC visibility
5. Evaluate optional eBPF path for Linux

Exit criteria:

- advanced diagnostics meaningfully improve root-cause analysis

What can be said after this phase:

- Sysray provides deeper diagnosis than legacy local monitoring tools

## Phase 6: Distributed Architecture

Goal: move from local observability engine to scalable platform.

Order:

1. Define agent responsibilities
2. Define wire protocol and transport
3. Define aggregation model
4. Define storage and retention boundaries
5. Define backpressure and failure semantics
6. Prototype multi-host topology

Exit criteria:

- there is a bounded and coherent distributed architecture

What can be said after this phase:

- Sysray has a real path toward distributed scale

Until this phase is done, "scalable to infinity" should not appear in messaging.

## Phase 7: Enterprise Track

Goal: add governance, supportability, and operational guarantees.

Order:

1. Versioning and compatibility policy
2. Secure configuration handling
3. AuthN / AuthZ model
4. RBAC
5. SSO
6. Auditability
7. Release hardening, signing, support process

Exit criteria:

- deployment, upgrade, access control, and supportability are enterprise-capable

What can be said after this phase:

- Sysray has an enterprise-grade operational story

## Recommended Immediate Sequence

If execution starts now, the next exact order should be:

1. Linux collector tests
2. Benchmark harness for overhead
3. Measured overhead docs
4. Linux service and schedule flow validation hardening
5. Stress validation for busy Linux hosts
6. macOS CPU/memory/system parity hardening
7. macOS disk/network/process parity hardening
8. Windows CPU/memory/system parity hardening
9. Windows disk/network/process parity hardening
10. Better JVM awareness
11. cgroup v2
12. PSI

## What Not To Pull Forward Too Early

Do not prioritize these before the phases above:

- anomaly detection
- correlation engine
- eBPF
- distributed mode
- enterprise controls

These are high-value later, but they are not the fastest path to a strong product.
