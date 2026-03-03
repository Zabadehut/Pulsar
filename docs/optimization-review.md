# Sysray Optimization Review

This document tracks where optimization effort is justified, where it is premature, and which assumptions should be challenged.

## Objective

Sysray should be fast because it is designed carefully and measured honestly.

That means:

- optimize only after identifying a real cost
- keep architectural simplicity where it does not hurt
- challenge attractive claims such as "zero overhead" unless backed by numbers

## Current Assumptions To Challenge

### 1. Snapshot cloning is probably "fine"

Current state:

- the scheduler clones the snapshot before publishing it

Question:

- is this cheap enough once process lists and disk/network/process counts grow?

Cost to investigate:

- `S`

Cost to optimize if needed:

- `M` to `L`

Potential payoff:

- lower allocation pressure and lower publish latency

Recommendation:

- measure before redesigning ownership flow

### 2. `/proc` parsing overhead is negligible

Current state:

- Linux collectors parse `/proc` and related files directly

Question:

- does this stay cheap on hosts with many processes, interfaces, disks, or heavy container density?

Cost to investigate:

- `S`

Cost to optimize if needed:

- `M`

Potential payoff:

- lower CPU usage on large hosts

Recommendation:

- add benchmarks on small, medium, and busy hosts

### 3. Process collection depth is good enough

Current state:

- process collector does direct `/proc` traversal and simple JVM heuristics

Question:

- is the cost acceptable versus the diagnostic value?

Cost to investigate:

- `S`

Cost to improve:

- `M` for heuristics
- `L` for deep per-thread/runtime-aware views

Potential payoff:

- stronger differentiation from legacy tools

Recommendation:

- keep top-N efficient first, deepen later

### 4. Cross-platform support is mostly a collector problem

Current state:

- architecture is ready, implementations are not

Question:

- how much product surface changes once macOS and Windows become real?

Cost to investigate:

- `S`

Cost to deliver:

- `L`

Potential payoff:

- credible cross-platform story

Recommendation:

- do not underestimate testing and service integration cost

### 5. More metrics automatically make the product better

Current state:

- roadmap contains many advanced metrics

Question:

- which metrics materially improve diagnosis versus merely expanding the matrix?

Cost to investigate:

- `S`

Cost to implement:

- varies from `M` to `XL`

Potential payoff:

- clearer prioritization and less scope creep

Recommendation:

- prioritize metrics with strong diagnostic leverage:
  - disk latency
  - cgroup v2
  - PSI
  - replay
  - JVM depth

## Optimization Priority Table

| Area | Current Pain | Investigation Cost | Fix Cost | Recommendation |
|---|---|---:|---:|---|
| Snapshot cloning | Unknown | S | M-L | Measure first |
| `/proc` parsing | Unknown | S | M | Benchmark first |
| Process collector | Medium risk on busy hosts | S | M-L | Measure and cap work |
| Exporter allocations | Low | S | M | Defer until measured |
| TUI rendering | Unknown | S | M | Profile under live updates |
| Service integration | Product risk, not perf risk | S | M | Validate behavior, not micro-optimize |

## Recommended Next Measurements

1. Time one full collection cycle on an idle Linux machine
2. Time one full collection cycle with many processes
3. Measure snapshot size and clone cost
4. Measure `top`-style refresh cost under sustained updates
5. Compare collector costs by subsystem: CPU, memory, disk, network, process

## What To Avoid Right Now

- rewriting stable code for stylistic performance assumptions
- adding eBPF before basic metrics are fully trustworthy
- building distributed architecture before single-host overhead is measured
- promising "minimum overhead" without benchmark data

## Best Current Optimization Bets

If time is limited, the best near-term bets are:

1. benchmark current collectors
2. complete disk latency metrics
3. reduce unnecessary work in process collection
4. validate real-world behavior before deeper refactors
