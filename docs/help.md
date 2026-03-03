# Pulsar Help

This page aligns the operator help with the current CLI surface.

## Available Now

Use `pulsar --help` for the command index and `pulsar <command> --help` for command details.

Current commands:

- `pulsar`
- `pulsar tui`
- `pulsar snapshot --format json|csv|prometheus`
- `pulsar record --interval 5s --output ./captures --rotate hourly --keep-files 24`
- `pulsar server --port 9090`
- `pulsar top --sort cpu --limit 20`
- `pulsar watch --pid <PID>`
- `pulsar replay <FILE>`
- `pulsar explain <TERM> [--lang fr|en] [--audience beginner|expert]`
- `pulsar service install|status|uninstall`

Benchmark helper:

- `./scripts/benchmark-overhead-linux.sh --duration 30 --interval 5 --snapshot-count 25`
- current Linux baseline result is documented in `docs/benchmarking.md`

TUI knowledge helper:

- `/` opens reference search
- `?` toggles the technical index
- `1`..`6` switch operator presets (`overview`, `storage`, `network`, `process`, `pressure`, `full`)
- `s` toggles the system panel
- `Esc` closes search or the index pane

## Command Notes

### `record`

Current behavior:

- writes local `.jsonl` files
- can rotate raw files by hour or day
- can rotate raw files on size threshold with `--max-file-size-mb`
- can prune old raw files with `--keep-files`
- does not yet compress rotated archives

Current example:

```bash
mkdir -p ./captures
pulsar record \
  --interval 5s \
  --output ./captures \
  --rotate hourly \
  --max-file-size-mb 512 \
  --keep-files 48
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

### Planned archive compression

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
