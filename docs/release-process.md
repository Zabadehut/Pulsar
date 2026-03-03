# Release Process

## GitHub Environment

Create a protected GitHub environment named `release` and store the signing secrets there.
That keeps the private key scoped to the publish job instead of exposing it to every build job.

Required secrets for signed releases:

- `SYSRAY_GPG_PRIVATE_KEY`: ASCII-armored private key, base64-encoded before upload
- `SYSRAY_GPG_KEY_ID`: key identifier used by `gpg --local-user`

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

- runs `./scripts/build-complete.sh` on Linux, macOS, and Windows
- uploads the generated `dist/` artifacts to the workflow run
- imports the GPG key in the `release` environment when both signing secrets are present
- signs the checksum files when a key is available
- publishes the archives, checksums, and checksum signatures to the GitHub Release

If the signing secrets are absent, the workflow still publishes the release artifacts, but without `*.SHA256SUMS.asc`.

## Local Verification

The same local command remains the source of truth for release assembly:

```bash
./scripts/build-complete.sh
```

If `SYSRAY_GPG_KEY_ID` is set and the matching private key is available in the local GPG keyring, the script also emits `dist/*.SHA256SUMS.asc`.

## Linux User Install

For local Linux installs, prefer the stable user-level install script:

```bash
./scripts/install-linux-user.sh
```

It installs the bundled release binary to `~/.local/bin/sysray` and reinstalls the user service against that path, which avoids services being pinned to `target/debug/sysray`.
