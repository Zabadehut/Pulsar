# Sysray Benchmarking

This document defines the current local benchmark harness for Sysray.

## Goal

Measure overhead before making performance claims.

The current benchmark focus is Linux-first and answers two concrete questions:

- what does the long-running `sysray record` path cost?
- what does repeated one-shot `sysray snapshot --format json` cost?

## Harness

Use the project script:

```bash
./scripts/benchmark-overhead-linux.sh
```

Useful flags:

```bash
./scripts/benchmark-overhead-linux.sh \
  --duration 30 \
  --interval 5 \
  --snapshot-count 25
```

Artifacts are written under:

```text
.benchmarks/<UTC_RUN_ID>/
```

Important files:

- `summary.md`: human-readable summary
- `summary.csv`: tabular data for later comparison
- `samples/`: raw `/proc` snapshots for resident-process measurements
- `*.stdout.log` and `*.stderr.log`: captured command output

## Current Scenarios

### `sysray / resident`

Runs:

```bash
sysray record --interval <N>s --output <dir>
```

This approximates the steady-state cost of the local recording path.

Measured from `/proc/<pid>`:

- average CPU %
- RSS / VSZ
- read bytes
- write bytes
- FD count

### `sysray / snapshot_json`

Runs repeated one-shot snapshots:

```bash
sysray snapshot --format json
```

This captures collection + serialization overhead for the one-shot path.

Measured by sampling each short-lived snapshot process through `/proc` across the repeated workload:

- elapsed seconds
- average CPU %
- max RSS
- max VSZ
- cumulative read/write bytes
- max FD count

## Linux Comparators

When available, the harness also runs:

- `nmon`
- `vmstat`
- `sar`

These are not treated as semantically identical products. They are only rough local-overhead reference points.

## How To Read Results

- Compare `sysray / resident` against other resident tools for steady-state overhead.
- Compare `sysray / snapshot_json` across commits to detect regressions in one-shot collection and serialization.
- Treat busy hosts separately from idle hosts. A single idle-machine number is not enough.

## Recommended Baseline Runs

Run the harness at least in these situations:

1. Idle Linux workstation or VM
2. Host with high process count
3. Host with multiple disks and active network traffic
4. Before and after collector or scheduler performance changes

## Limits

- Linux only for now
- no CI gating yet
- snapshot loop reports max RSS for the repeated workload, not a daemon steady-state footprint
- comparisons with external tools are directional, not product-equivalence claims

## Current Measured Baseline

Reference run captured on March 3, 2026 on `rocky9-workstation-master`:

- kernel: `5.14.0-611.34.1.el9_7.x86_64`
- CPU count: `4`
- command: `./scripts/benchmark-overhead-linux.sh --duration 30 --interval 5 --snapshot-count 25`
- artifact root: `.benchmarks/20260303T102514Z/`

Key results:

- `sysray / resident`: `0.35%` average CPU, `13352 KB` RSS, `286220 KB` VSZ, `49152` write bytes
- `sysray / snapshot_json`: `0.83%` average CPU across `25` snapshots, `13212 KB` max RSS, `286164 KB` max VSZ
- `vmstat / resident`: `3444 KB` RSS on the same host for rough directional comparison

Interpretation:

- the resident `record` path is lightweight enough to support a credible Linux low-overhead claim
- the one-shot snapshot path is also inexpensive on this host, though it should still be checked on busier systems
- this baseline is from an apparently quiet host, so it should not be treated as a stressed-host ceiling
