# Pulsar Help

This page aligns the operator help with the current CLI surface.

## Available Now

Use `pulsar --help` for the command index and `pulsar <command> --help` for command details.

Current commands:

- `pulsar`
- `pulsar tui`
- `pulsar snapshot --format json|csv|prometheus`
- `pulsar inventory --format table|json`
- `pulsar record --interval 5s --output ./captures --rotate hourly --keep-files 24 --compress zip`
- `pulsar server --port 9090`
- `pulsar top --sort cpu --limit 20`
- `pulsar watch --pid <PID>`
- `pulsar replay <FILE>`
- `pulsar explain <TERM> [--lang fr|en] [--audience beginner|expert]`
- `pulsar service install|status|uninstall`

HTTP inventory helper:

- `/snapshot` returns the full structured snapshot
- `/inventory` returns host, disk, and network inventory hints for structure/protocol/media/topology

Benchmark helper:

- `./scripts/benchmark-overhead-linux.sh --duration 30 --interval 5 --snapshot-count 25`
- current Linux baseline result is documented in `docs/benchmarking.md`

TUI knowledge helper:

- `/` opens reference search
- `?` toggles the technical index
- `l` opens the live logs pane
- `L` adds a watched path or pattern to the live logs pane
- `e` toggles the logs pane between `all` and `errors` focus
- `1`..`6` switch operator presets (`overview`, `io`, `network`, `process`, `pressure`, `full`)
- `7`..`0` open expert local diagnostics (`pressure+`, `network+`, `jvm+`, `disk+`)
- `g` opens `inventory+` for a local disk tree / stack view
- `-` returns from the expert submenu to the normal monitoring layout
- `v` toggles compact vs detailed views
- `i` switches the TUI language and keeps the index aligned with it (`fr` / `en`)
- `k` toggles the Linux panel
- `s` toggles the system panel
- `Esc` closes search or the index pane

Index behavior:

- the technical index is localized with the TUI language
- expert local views also bias the index toward their own diagnostics and terms
- the expert drill-down body now stays aligned with the same localized terms as the index
- `pressure+` exposes pressure paths and pressured processes
- `network+` exposes interface ranking and socket/TCP state breakdown
- `network+` also exposes session lenses (`handshake`, `closing backlog`, `loss path`)
- the standard and expert network views now expose portable `topology`, `family`, and `medium` hints per interface
- `jvm+` exposes JVM hotspots plus runtime profiles (`role`, dominant pressure, heap hint)
- `pressure+` also exposes pressure lenses (`reclaim`, `swap`, `host/cgroup gap`, stall mixes)
- `disk+` exposes hot disks plus waiters/IO correlation
- `disk+` also exposes contention lenses (`busy`, `latency`, `queue`, waiter pressure`)
- `inventory+` exposes a local `lsblk`-like reading with tree, volume kind, filesystem, stack path, refs, and flags
- TUI intent split: `io` preset = broad storage-focused dashboard, `disk+` = contention/perf drill-down, `inventory+` = topology/inventory drill-down
- Linux inventory enrichment now also tries to recognize `LVM`, `LUKS`, `multipath`, `md`, and remote filesystems like `NFS`/`SMB`
- remote filesystem inventory is now modeled across Linux, macOS, and Windows when the OS exposes enough information
- CSV and Prometheus exports now also surface disk inventory categories such as `volume_kind`, `filesystem_family`, relation counts, stack depth, and disk flags
- the alerts panel now also surfaces recent native OS events (`info`, `warning`, `error`) when the current OS exposes them naturally
- the live logs pane merges those native OS events with tailed file targets from recent files under the watched paths/patterns
- watched files are now followed incrementally with offsets, and truncation/rotation is detected so the pane behaves more like a multi-file `tail`
- the logs pane prioritizes files that are actively being written and can be switched to an error-focused view for faster triage
- the standard disk views now expose `structure`, `proto`, and `media` hints to make cross-OS storage paths easier to read
- disk inventory is moving toward an `lsblk`-like model with `parent`, `filesystem`, `uuid`, `label`, `model`, `serial`, `refs`, `mounts`, and `children`
- `disk+` now also surfaces stack and stable-ref cues for the hottest path so UUID/ref/parentage stay visible in the TUI
- `/inventory` now returns richer host, disk, and network inventory details for API consumers, including logical stacks and Linux-specific disk flags when available

Product boundary reminder:

- richer local operator diagnostics belong in Pulsar Core
- enterprise scope starts at governance, fleet policy, shared history, and access control

## Command Notes

### `record`

Current behavior:

- writes local `.jsonl` files
- can rotate raw files by hour or day
- can rotate raw files on size threshold with `--max-file-size-mb`
- can prune old local segments with `--keep-files`
- can compress closed segments with `--compress zip`

Current example:

```bash
mkdir -p ./captures
pulsar record \
  --interval 5s \
  --output ./captures \
  --rotate hourly \
  --max-file-size-mb 512 \
  --keep-files 48 \
  --compress zip
```

### `snapshot`

Examples:

```bash
pulsar snapshot --format json
pulsar snapshot --format csv
pulsar snapshot --format prometheus
```

### `service`

Examples:

```bash
pulsar service install
pulsar service status
pulsar service uninstall
```

Developer reminder on Linux:

- `cargo build` updates `target/debug/pulsar` only
- the `systemd --user` service usually runs `~/.local/bin/pulsar` via `~/.local/share/pulsar/pulsar-service.sh`
- after local code changes, reinstall the binary before restarting the service

```bash
./scripts/install-linux-user.sh
systemctl --user restart pulsar.service
systemctl --user status pulsar.service --no-pager
journalctl --user -u pulsar.service -n 50 --no-pager
```

OS mapping:

- Linux: `systemd --user`
- macOS: `launchd`
- Windows: Task Scheduler

## Planned, Not In CLI Yet

These shapes are documented for roadmap clarity only. They do not exist in the current binary help.

### Planned standalone archive command

```bash
pulsar archive zip \
  --input ./captures/pulsar_20260303_140000.jsonl \
  --output ./captures/pulsar_20260303_140000.jsonl.zip
```

Constraints for that future archive path:

- Rust-native implementation
- no OS archive utility dependency
- same behavior on Linux, macOS, and Windows

## Documentation Map

- `docs/community-cheatsheet.md`
- `docs/benchmarking.md`
- `docs/reference-architecture.md`
- `docs/enterprise-cheatsheet.md`
- `docs/cross-os-cheatsheet.md`
- `docs/product-scope.md`
- `docs/metrics-matrix.md`
