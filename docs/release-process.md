# Release Process

## GitHub Secrets

GitHub release signing requires these repository or organization secrets:

- `PULSAR_GPG_PRIVATE_KEY`: ASCII-armored private key, base64-encoded before upload
- `PULSAR_GPG_KEY_ID`: key identifier used by `gpg --local-user`

Example to prepare the secret payload locally:

```bash
gpg --armor --export-secret-keys YOUR_KEY_ID | base64 -w 0
```

On macOS, use:

```bash
gpg --armor --export-secret-keys YOUR_KEY_ID | base64
```

## Tag Release

Push a semantic version tag such as `v0.4.0` to trigger the release workflow. The tag suffix must match the version in `Cargo.toml`:

```bash
git tag v0.4.0
git push origin v0.4.0
```

The release workflow:

- validates the signing secrets
- imports the GPG key
- runs `./scripts/build-complete.sh` on Linux, macOS, and Windows
- uploads the generated `dist/` artifacts to the workflow run
- publishes the archives, checksums, and checksum signatures to the GitHub Release

## Local Verification

The same local command remains the source of truth for release assembly:

```bash
./scripts/build-complete.sh
```

If `PULSAR_GPG_KEY_ID` is set and the matching private key is available in the local GPG keyring, the script also emits `dist/*.SHA256SUMS.asc`.

## Linux User Install

For local Linux installs, prefer the stable user-level install script:

```bash
./scripts/install-linux-user.sh
```

It installs the bundled release binary to `~/.local/bin/pulsar` and reinstalls the user service against that path, which avoids services being pinned to `target/debug/pulsar`.
