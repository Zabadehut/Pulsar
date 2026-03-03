# Sysray Metrics Checklist

This checklist is the practical execution view of Sysray's metric surface.

Legend:

- `Yes`: implemented in a meaningful way today
- `Partial`: exists, but not yet at target depth
- `No`: not implemented yet

Cost scale:

- `S`: small
- `M`: medium
- `L`: large
- `XL`: very large / architecture-level

Value scale:

- `High`: important for public usefulness
- `Very High`: important for product differentiation
- `Strategic`: foundational for future platform or enterprise scope

## CPU

| Checklist | Linux | macOS | Windows | Current | Target Phase | Cost | Value | Notes |
|---|---|---|---|---|---|---|---|---|
| [x] Global CPU usage % | Yes | Partial | Partial | Linux complete, baseline host reading on macOS and Windows | V1 | S | Very High | core system metric |
| [x] Per-core CPU usage | Yes | No | Partial | Linux complete, Windows baseline available | V1 | S | High | useful in TUI |
| [x] Load average | Yes | Partial | No | Linux complete, macOS baseline available | V1 | S | High | semantics vary by OS |
| [x] Context switches | Yes | No | No | Implemented on Linux | V1 | S | Medium | useful for deep diagnosis |
| [x] Interrupt count | Yes | No | No | Implemented on Linux | V1 | S | Medium | useful but secondary |
| [x] iowait % | Yes | No | No | Implemented on Linux | V1 | S | High | useful for storage bottlenecks |
| [x] steal % | Yes | Partial | Partial | Linux complete, placeholder-level values elsewhere | V1 | S | Medium | more relevant on virtualized hosts |
| [ ] CPU pressure indicators | No | No | No | Not implemented | V2/V3 | M | High | likely tied to PSI or synthetic indices |

## Memory

| Checklist | Linux | macOS | Windows | Current | Target Phase | Cost | Value | Notes |
|---|---|---|---|---|---|---|---|---|
| [x] Total memory | Yes | Partial | Partial | baseline host memory collection exists on macOS and Windows | V1 | S | Very High | core metric |
| [x] Used memory | Yes | Partial | Partial | baseline host memory collection exists on macOS and Windows | V1 | S | Very High | core metric |
| [x] Free memory | Yes | Partial | Partial | baseline host memory collection exists on macOS and Windows | V1 | S | High | expected metric |
| [x] Available memory | Yes | Partial | Partial | baseline host memory collection exists on macOS and Windows | V1 | S | Very High | more meaningful than free |
| [x] Cached memory | Yes | Partial | No | partial field mapping on macOS | V1 | S | High | useful for diagnosis |
| [x] Buffers | Yes | No | No | Linux only today | V1 | S | Medium | Linux-specific relevance |
| [x] Dirty pages | Yes | No | No | Linux only today | V1 | S | Medium | useful for IO diagnosis |
| [x] Swap total / used | Yes | Partial | Partial | baseline swap reading exists on macOS and Windows | V1 | S | High | important signal |
| [x] Memory usage % | Yes | Partial | Partial | baseline host memory collection exists on macOS and Windows | V1 | S | Very High | core metric |
| [x] Memory pressure score | Yes | Partial | Partial | Shared pipeline exists | V1 | S | High | depends on base memory metrics |

## Disk / Filesystem

| Checklist | Linux | macOS | Windows | Current | Target Phase | Cost | Value | Notes |
|---|---|---|---|---|---|---|---|---|
| [x] Disk total / used / free | Yes | Partial | Partial | capacity paths exist on all three OSes | V1 | S | Very High | expected metric |
| [x] Disk usage % | Yes | Partial | Partial | capacity paths exist on all three OSes | V1 | S | Very High | expected metric |
| [x] Read IOPS | Yes | No | No | Implemented on Linux | V1 | S | High | useful for diagnosis |
| [x] Write IOPS | Yes | No | No | Implemented on Linux | V1 | S | High | useful for diagnosis |
| [x] Read throughput | Yes | No | No | Implemented on Linux | V1 | S | High | expected metric |
| [x] Write throughput | Yes | No | No | Implemented on Linux | V1 | S | High | expected metric |
| [x] Disk utilization % | Yes | No | No | Implemented on Linux | V1 | S | High | useful for saturation |
| [x] Disk await / latency | Yes | No | No | Implemented on Linux | V1/P1 | M | High | derived from `/proc/diskstats` timing counters |
| [ ] Per-filesystem detail parity | No | No | No | incomplete | V2 | M | Medium | more polish than foundation |

## Network

| Checklist | Linux | macOS | Windows | Current | Target Phase | Cost | Value | Notes |
|---|---|---|---|---|---|---|---|---|
| [x] RX bytes/sec | Yes | Partial | Partial | baseline interface byte counters exist on macOS and Windows | V1 | S | Very High | core metric |
| [x] TX bytes/sec | Yes | Partial | Partial | baseline interface byte counters exist on macOS and Windows | V1 | S | Very High | core metric |
| [x] RX packets/sec | Yes | Partial | Partial | baseline packet counters exist on macOS and Windows | V1 | S | High | useful for interface analysis |
| [x] TX packets/sec | Yes | Partial | Partial | baseline packet counters exist on macOS and Windows | V1 | S | High | useful for interface analysis |
| [x] RX/TX errors | Yes | Partial | Partial | baseline interface error counters exist on macOS and Windows | V1 | S | High | important signal |
| [x] RX/TX drops | Yes | No | Partial | Windows exposes discarded packet counts; macOS path is incomplete | V1 | S | High | important signal |
| [x] Total TCP connections | Yes | Partial | Partial | baseline connection counts exist on macOS and Windows | V1 | S | Medium | useful but basic |
| [x] Established TCP connections | Yes | Partial | Partial | baseline connection counts exist on macOS and Windows | V1 | S | Medium | useful but basic |
| [ ] UDP / socket family depth | No | No | No | not implemented | V2 | M | Medium | optional deeper view |

## Process / Application

| Checklist | Linux | macOS | Windows | Current | Target Phase | Cost | Value | Notes |
|---|---|---|---|---|---|---|---|---|
| [x] Top N process listing | Yes | Partial | Partial | baseline process enumeration exists on macOS and Windows | V1 | S | Very High | core workflow |
| [x] Process CPU % | Yes | Yes | Partial | macOS now exposes cumulative CPU time + direct `%cpu`; Windows mixes Win32 timings with perf-counter hints | V1 | S | Very High | core workflow |
| [x] Process RSS | Yes | Partial | Partial | baseline memory values exist on macOS and Windows | V1 | S | High | expected metric |
| [x] Process VSZ | Yes | Partial | Partial | baseline memory values exist on macOS and Windows | V1 | S | Medium | expected but less critical |
| [x] Process thread count | Yes | Partial | Partial | baseline thread counts exist on macOS and Windows | V1 | S | High | useful metric |
| [x] Process FD count | Yes | Yes | Partial | macOS counts open descriptors; Windows exposes handle count as the nearest equivalent | V1 | S | High | strong differentiator vs basic tools |
| [x] Process owner | Yes | Partial | Partial | macOS user attribution exists; Windows now resolves owners when the OS allows it | V1 | S | Medium | useful for admins |
| [x] Process read/write bytes | Yes | Yes | Yes | implemented on Linux, macOS, and Windows with OS-native counters | V1 | S | High | useful for heavy hitters |
| [x] Basic JVM detection | Yes | Partial | Partial | simple heuristic only | V1 | S | Medium | present but shallow |
| [ ] Strong JVM awareness | No | No | No | not implemented | V2 | M | Very High | large product differentiator |
| [ ] Deep per-thread analysis | No | No | No | not implemented | V2/V3 | L | Very High | technically valuable |
| [ ] Python / app-runtime awareness | No | No | No | not implemented | V2/V3 | L | High | broadens app-aware story |

## System

| Checklist | Linux | macOS | Windows | Current | Target Phase | Cost | Value | Notes |
|---|---|---|---|---|---|---|---|---|
| [x] Hostname | Yes | Yes | Yes | real across all three with OS-native sources | V1 | S | Medium | basic metadata |
| [x] OS name / version | Yes | Yes | Yes | real across all three with richer native version fields | V1 | S | Medium | basic metadata |
| [x] Kernel version | Yes | Yes | Yes | real across all three | V1 | S | Medium | basic metadata |
| [x] Uptime | Yes | Yes | Yes | real across all three with native timers | V1 | S | Medium | useful metadata |
| [x] CPU count | Yes | Yes | Yes | real across all three | V1 | S | Medium | useful metadata |
| [x] Architecture | Yes | Partial | Partial | real across all three | V1 | S | Low | low complexity |

## Derived / Smart Metrics

| Checklist | Linux | macOS | Windows | Current | Target Phase | Cost | Value | Notes |
|---|---|---|---|---|---|---|---|---|
| [x] CPU trend percentiles | Yes | Partial | Partial | shared pipeline exists | V1 | S | High | depends on CPU collection |
| [x] Memory pressure score | Yes | Partial | Partial | shared pipeline exists | V1 | S | High | useful signal |
| [x] Threshold alerts | Yes | Partial | Partial | shared pipeline exists | V1 | S | High | simple implementation |
| [ ] Synthetic health indices | No | No | No | not implemented | V2 | M | High | useful for non-experts |
| [ ] Anomaly detection | No | No | No | not implemented | V3 | L | Strategic | must be measured, not hand-wavy |
| [ ] Correlation engine OS ↔ app | No | No | No | not implemented | V3 | XL | Strategic | platform-level feature |

## Infrastructure / Modern System Signals

| Checklist | Linux | macOS | Windows | Current | Target Phase | Cost | Value | Notes |
|---|---|---|---|---|---|---|---|---|
| [ ] Containers / cgroup v2 | Partial | No | No | Linux support exists when available | V2 | L | Very High | strong differentiator |
| [ ] PSI | Partial | No | No | Linux support exists when available | V2 | M | Very High | excellent Linux differentiation |
| [ ] NUMA metrics | No | No | No | not implemented | V2/V3 | M | Medium | useful for high-end hosts |
| [ ] IPC monitoring | No | No | No | not implemented | V3 | L | Medium | niche but advanced |
| [ ] Security events | No | No | No | not implemented | V3/Enterprise | XL | Strategic | broad scope |
| [ ] eBPF option | No | No | No | not implemented | V3 | XL | Strategic | Linux-only advanced track |

## Runtime / Product Capabilities

| Checklist | Linux | macOS | Windows | Current | Target Phase | Cost | Value | Notes |
|---|---|---|---|---|---|---|---|---|
| [x] TUI mode | Yes | Partial | Partial | build and smoke validated in CI, Linux remains the deepest runtime path | V1 | M | Very High | flagship UX |
| [x] JSON export | Yes | Partial | Partial | shared exporter with native CI smoke tests | V1 | S | High | expected |
| [x] CSV export | Yes | Partial | Partial | shared exporter path, less explicitly smoke-tested | V1 | S | Medium | compatibility story |
| [x] Prometheus text export | Yes | Partial | Partial | shared exporter with native CI smoke tests | V1 | S | High | strong integration point |
| [x] Record mode | Yes | Partial | Partial | shared runtime, Linux remains the deepest runtime path | V1 | M | High | useful ops feature |
| [x] Replay mode | Partial | Partial | Partial | basic replay command exists, but feature depth and UX remain limited | V2 | M | High | important differentiator |
| [x] Service install scaffolding | Partial | Partial | Partial | templates + CLI exist, with dedicated CI validation for install/status/uninstall flows | V1/P1 | M | Medium | native managers still have OS-specific constraints |
| [ ] Multi-host / distributed mode | No | No | No | not implemented | V3 | XL | Strategic | entirely different architecture |
| [ ] Enterprise controls | No | No | No | not implemented | Enterprise | XL | Strategic | not V1 work |

## Immediate Gaps Worth Closing First

These are the highest-leverage missing or partial items relative to current product claims:

- [ ] Broader Linux collector test coverage
- [ ] Deeper macOS collector parity
- [ ] Deeper Windows collector parity
- [ ] Replay mode
- [ ] Better JVM awareness
- [ ] Measured overhead benchmarks
- [ ] Container / cgroup v2 support
- [ ] PSI on Linux

## Recommendation

Near-term execution order:

1. Finish V1 quality gaps on Linux
2. Validate and complete real macOS and Windows support
3. Add differentiating modern Linux metrics such as cgroup v2 and PSI
4. Only then invest in larger platform features such as anomaly detection, correlation, and distributed scale
