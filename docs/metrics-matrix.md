# Sysray Metrics Matrix

Legend:

- `Implemented`: real code exists and is wired
- `Partial`: some support exists, but not yet with parity-complete depth or identical OS semantics
- `Planned`: intentionally part of the roadmap, not implemented yet
- `Out of Scope (Current Phase)`: not currently targeted for the active phase

## Core Metrics

| Area | Metric / Capability | Linux | macOS | Windows | Status Notes |
|---|---|---|---|---|---|
| CPU | Global usage % | Implemented | Partial | Partial | baseline host reading exists on macOS and Windows |
| CPU | Per-core usage | Implemented | Planned | Partial | Windows exposes per-core usage, macOS broader parity pending |
| CPU | Load average | Implemented | Partial | Planned | macOS load averages exist; semantics differ by OS |
| CPU | Context switches | Implemented | Planned | Planned | Linux only today |
| CPU | Interrupts | Implemented | Planned | Planned | Linux only today |
| CPU | iowait % | Implemented | Planned | Planned | Linux only today |
| CPU | steal % | Implemented | Partial | Partial | meaningful on Linux; fallback values outside Linux still need deeper OS-native semantics |
| Memory | Total / used / free / available | Implemented | Partial | Partial | baseline host memory collection exists on macOS and Windows |
| Memory | Cache / buffers / dirty | Implemented | Partial | Planned | partial field mapping on macOS, not exposed on Windows |
| Memory | Swap total / used | Implemented | Partial | Partial | baseline swap reading exists on macOS and Windows |
| Disk | Capacity and usage % | Implemented | Partial | Partial | capacity path exists across all three OSes |
| Disk | Read/write IOPS | Implemented | Planned | Planned | Linux only today |
| Disk | Throughput | Implemented | Planned | Planned | Linux only today |
| Disk | Utilization % | Implemented | Planned | Planned | Linux only today |
| Disk | await / latency | Implemented | Planned | Planned | Linux computes await from `/proc/diskstats` times |
| Network | RX/TX throughput | Implemented | Partial | Partial | baseline interface byte counters exist on macOS and Windows |
| Network | Packet rate | Implemented | Partial | Partial | baseline packet counters exist on macOS and Windows |
| Network | Errors / drops | Implemented | Partial | Partial | baseline interface error counters exist on macOS and Windows |
| Network | TCP connection counts | Implemented | Partial | Partial | baseline connection counts exist on macOS and Windows |
| Process | Top N process list | Implemented | Partial | Partial | baseline process enumeration exists on macOS and Windows |
| Process | CPU % | Implemented | Implemented | Partial | macOS now exposes cumulative CPU time + direct `%cpu`; Windows mixes Win32 timings with perf-counter hints |
| Process | RSS / VSZ | Implemented | Partial | Partial | baseline memory values exist on macOS and Windows |
| Process | Threads count | Implemented | Partial | Partial | baseline thread counts exist on macOS and Windows |
| Process | FD count | Implemented | Implemented | Partial | macOS counts open descriptors; Windows exposes handle count as the nearest equivalent |
| Process | User attribution | Implemented | Partial | Partial | macOS user attribution exists; Windows now resolves owners when the OS allows it |
| Process | IO read/write bytes | Implemented | Implemented | Implemented | macOS uses `proc_pid_rusage`; Windows uses Win32 process IO counters |
| Process | JVM detection | Partial | Partial | Partial | simple heuristics only |
| System | Hostname / OS / kernel / uptime / CPU count | Implemented | Implemented | Implemented | real across all three; field naming still follows OS-native conventions |

## Derived / Pipeline Metrics

| Area | Metric / Capability | Linux | macOS | Windows | Status Notes |
|---|---|---|---|---|---|
| Pipeline | CPU trend percentiles | Implemented | Implemented via shared pipeline | Implemented via shared pipeline | depends on base CPU data |
| Pipeline | Memory pressure score | Implemented | Implemented via shared pipeline | Implemented via shared pipeline | depends on base memory data |
| Pipeline | Threshold alerts | Implemented | Implemented via shared pipeline | Implemented via shared pipeline | currently simple local alerts |
| Pipeline | Synthetic indices | Planned | Planned | Planned | broader health scoring not yet done |
| Pipeline | Anomaly detection | Planned | Planned | Planned | not implemented |
| Pipeline | Correlation engine | Planned | Planned | Planned | not implemented |

## Advanced Capabilities vs Vision

| Capability | Linux | macOS | Windows | Current State |
|---|---|---|---|---|
| Structured JSON export | Implemented | Implemented | Implemented | shared exporter layer |
| Prometheus export | Implemented | Implemented | Implemented | shared exporter layer |
| Record mode | Implemented | Implemented | Implemented | shared runtime |
| Replay mode | Partial | Partial | Partial | CLI mode exists but feature depth remains limited |
| Service integration | Partial | Partial | Partial | scaffolding and templates exist, with dedicated CI validation for install/status/uninstall flows |
| Containers / cgroup v2 | Partial | Out of Scope (Current Phase) | Out of Scope (Current Phase) | Linux-only support exists when available |
| PSI | Partial | Out of Scope (Current Phase) | Out of Scope (Current Phase) | Linux-only support exists when available |
| NUMA | Planned | Planned | Planned | not yet implemented |
| eBPF | Planned | Out of Scope (Current Phase) | Out of Scope (Current Phase) | not yet implemented |
| Deep per-thread analysis | Planned | Planned | Planned | not yet implemented |
| IPC monitoring | Planned | Planned | Planned | not yet implemented |
| Security events | Planned | Planned | Planned | not yet implemented |
| Application-aware universal monitoring | Partial | Planned | Planned | JVM heuristic only |
| Multi-host / distributed mode | Planned | Planned | Planned | not yet implemented |
| Enterprise controls | Planned | Planned | Planned | not yet implemented |

## Interpretation

Today, Sysray is best described as:

- a working Linux observability foundation
- a cross-platform runtime with baseline macOS and Windows host coverage
- a roadmap-bearing product, not yet a parity-complete cross-platform product

It should not yet be described as:

- full macOS support
- full Windows support
- enterprise-ready
- infinitely scalable
