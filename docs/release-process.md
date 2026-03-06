# Release Process

## GitHub Environment

Create a protected GitHub environment named `release` and store the signing secrets there.
That keeps the private key scoped to the publish job instead of exposing it to every build job.

Required secrets for signed releases:

- `SYSRAY_GPG_PRIVATE_KEY`: ASCII-armored private key, base64-encoded before upload
- `SYSRAY_GPG_KEY_ID`: key identifier used by `gpg --local-user`

Optional platform trust/signing secrets:

- Windows Authenticode:
- `SYSRAY_WINDOWS_SIGN_PFX_BASE64`: base64-encoded `.pfx` certificate payload
- `SYSRAY_WINDOWS_SIGN_PFX_PASSWORD`: password for the `.pfx`
- `SYSRAY_WINDOWS_SIGN_TIMESTAMP_URL`: RFC3161 timestamp URL (defaults to `http://timestamp.digicert.com`)
- macOS code signing:
- `SYSRAY_MACOS_SIGN_CERT_BASE64`: base64-encoded `.p12` Developer ID certificate payload
- `SYSRAY_MACOS_SIGN_CERT_PASSWORD`: password for the `.p12`
- `SYSRAY_MACOS_SIGN_IDENTITY`: exact `codesign` identity (for example `Developer ID Application: Your Company (TEAMID)`)

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
- publishes a Linux `.rpm` when the Linux runner has `rpmbuild`
- publishes a Windows `.exe` in addition to the Windows `.zip`
- signs the Windows standalone executable with Authenticode when Windows signing secrets are present
- signs the macOS standalone binary with `codesign` when macOS signing secrets are present
- imports the GPG key in the `release` environment when both signing secrets are present
- signs the checksum files when a key is available
- publishes the archives, checksums, and checksum signatures to the GitHub Release

If the signing secrets are absent, the workflow still publishes release artifacts, but:

- without `*.SHA256SUMS.asc` when GPG secrets are missing
- without Authenticode when Windows certificate secrets are missing
- without `codesign` identity when macOS certificate secrets are missing

## Local Verification

The same local command remains the source of truth for release assembly:

```bash
./scripts/build-complete.sh
```

Optional local signing variables recognized by the script:

- `SYSRAY_GPG_KEY_ID` for checksum signatures (`dist/*.SHA256SUMS.asc`)
- `SYSRAY_WINDOWS_SIGN_PFX_BASE64` + `SYSRAY_WINDOWS_SIGN_PFX_PASSWORD` for Windows Authenticode
- `SYSRAY_MACOS_SIGN_CERT_BASE64` + `SYSRAY_MACOS_SIGN_CERT_PASSWORD` + `SYSRAY_MACOS_SIGN_IDENTITY` for macOS `codesign`

## Linux User Install

For local Linux installs, prefer the stable user-level install script:

```bash
./scripts/install-linux-user.sh
```

It installs the bundled release binary to `~/.local/bin/sysray` and reinstalls both the user service and recurring schedule against that path, which avoids automation being pinned to `target/debug/sysray`.
