# GitHub Milestones and Issues

This document is a ready-to-create GitHub planning breakdown based on the execution roadmap.

Use it as the source for milestones, issues, and project board columns.

## Milestone 1: Linux Credibility Baseline

Goal: make the current Linux foundation testable, measurable, and publicly defensible.

Suggested issues:

1. Add benchmark harness for collection overhead
2. Document measured local overhead in docs
3. Expand Linux collector regression coverage beyond the current baseline
4. Validate `pulsar service install/status/uninstall` on Linux user services under stricter scenarios
5. Keep CLI help and docs aligned with implemented commands
6. Stress-test Linux behavior on busy hosts

Definition of done:

- tests remain green and cover stressed Linux scenarios
- compatibility behavior remains verified
- overhead is measured instead of assumed

## Milestone 2: Complete V1 Linux Quality

Goal: close the most visible Linux quality gaps.

Suggested issues:

1. Implement real disk await / latency computation
2. Improve process collector accuracy and edge-case handling
3. Add stress validation for many processes and interfaces
4. Improve exporter consistency across JSON, CSV, and Prometheus output
5. Validate TUI rendering against real collector data

Definition of done:

- Linux metric surface matches claimed V1 depth
- no known shallow placeholder remains in core Linux metrics

## Milestone 3: Real macOS Support

Goal: turn macOS from architecture-only to implemented support.

Suggested issues:

1. Implement macOS system collector
2. Implement macOS CPU collector
3. Implement macOS memory collector
4. Implement macOS disk collector
5. Implement macOS network collector
6. Implement macOS process collector
7. Validate TUI and snapshot commands on macOS
8. Validate `launchd` service install/status/uninstall flow

Definition of done:

- CI passes on macOS
- core commands build and run
- documented limitations are explicit

## Milestone 4: Real Windows Support

Goal: turn Windows from architecture-only to implemented support.

Suggested issues:

1. Implement Windows system collector
2. Implement Windows CPU collector
3. Implement Windows memory collector
4. Implement Windows disk collector
5. Implement Windows network collector
6. Implement Windows process collector
7. Validate TUI and snapshot commands on Windows
8. Validate Task Scheduler service install/status/uninstall flow

Definition of done:

- CI passes on Windows
- core commands build and run
- documented limitations are explicit

## Milestone 5: High-Value Differentiators

Goal: move beyond parity and add strong reasons to use Pulsar over legacy tools.

Suggested issues:

1. Improve JVM awareness beyond process-name heuristics
2. Deepen replay mode UX and operator workflow
3. Add Linux cgroup v2 collector
4. Add Linux PSI collector
5. Add synthetic health indices
6. Expand alerts beyond static thresholds

Definition of done:

- Pulsar has at least two strong differentiators visible to users

## Milestone 6: Performance and Hardware Affinity

Goal: support performance claims with evidence.

Suggested issues:

1. Profile scheduler allocation hotspots
2. Reduce snapshot copying where meaningful
3. Define idle and loaded overhead budgets
4. Add repeatable benchmarks
5. Publish benchmark results in docs

Definition of done:

- overhead claims are measurable and documented

## Milestone 7: Deep Diagnostics

Goal: increase depth for advanced operators and power users.

Suggested issues:

1. Add deep per-thread analysis
2. Add NUMA metrics
3. Add broader runtime-aware application signals
4. Investigate IPC visibility
5. Evaluate optional Linux eBPF path

Definition of done:

- Pulsar can explain more than basic host saturation

## Milestone 8: Distributed Platform

Goal: define and prototype the move from local engine to scalable platform.

Suggested issues:

1. Define agent model
2. Define transport protocol
3. Define aggregation layer
4. Define storage and retention boundaries
5. Define backpressure and failure semantics
6. Prototype multi-host topology

Definition of done:

- distributed architecture is coherent and bounded

## Milestone 9: Enterprise Track

Goal: add governance and operational guarantees.

Suggested issues:

1. Define versioning and compatibility policy
2. Secure configuration and secret handling
3. Define auth model
4. Add RBAC
5. Add SSO
6. Add auditability
7. Harden release and support process

Definition of done:

- enterprise claims are backed by real controls and processes

## Milestone 10: Community/Core Health

Goal: keep the open source core useful, understandable, and contributor-friendly while the product expands.

Suggested issues:

1. Define contribution workflow and review expectations
2. Document community-safe roadmap boundaries versus enterprise scope
3. Tag issues suitable for first-time and recurring contributors
4. Add sample captures and reproducible local validation flows
5. Publish compatibility and deprecation guidance for snapshot schema changes

Definition of done:

- contributors can find and land meaningful core improvements
- community/core scope remains explicit as enterprise work grows

## Suggested Labels

- `area:collectors`
- `area:platform`
- `area:pipeline`
- `area:exporters`
- `area:tui`
- `area:service`
- `area:docs`
- `area:ci`
- `type:bug`
- `type:feature`
- `type:test`
- `type:benchmark`
- `type:refactor`
- `platform:linux`
- `platform:macos`
- `platform:windows`
- `priority:p0`
- `priority:p1`
- `priority:p2`
