# M13.2b native artifact contract

## Purpose

M13.2b defines a reviewable create-new artifact directory for each of the six
native CI targets. It packages the CLI and already-verified release evidence
without changing DXF behavior or assigning a release version.

## Reviewed targets

- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-pc-windows-msvc`
- `aarch64-pc-windows-msvc`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`

No other target is accepted. Windows packages use `bin/seacad.exe`; the other
four use `bin/seacad`.

## Directory and receipt

The package name is
`seacad-dxf-core-<workspace-version>-<target>`. Its payload is:

- one nonempty native CLI under `bin/`;
- the repository `README.md`;
- the deterministic CycloneDX inventory as `sbom.cdx.json`;
- the complete generated legal tree under `legal/`.

`RELEASE_RECEIPT.json` uses
`seacad-native-artifact-receipt/v1`. It records schema version, package,
workspace version, exact target, 40-character lowercase source commit, pinned
Rust version, and every payload's portable relative path, byte count, and
SHA-256. Payload entries are lexicographically sorted. The receipt does not
hash itself.

The assembler streams copies and hashes through a fixed 64-KiB buffer. It
rejects an existing destination, empty executable, symlinked or non-regular
source, symlinked/non-regular legal entry, duplicate destination, output inside
the legal source tree, non-portable path, legal depth above 16, or more than
4,096 legal entries. Failure after directory creation removes only that newly
created directory.

## Workflow boundary

The workflow is manual-dispatch only, has `contents: read`, disables checkout
credential persistence, uses exact action commit pins, and builds on the six
native hosted runners. Each job verifies generated schemas/release evidence,
runs all workspace tests, builds the CLI with Rust 1.97.1, assembles the
directory, and uploads it for 14-day retention.

The upload service's archive representation and digest are outside the receipt
contract. The committed workflow proves the process definition, not a
successful six-target run.

## Non-claims

M13.2b does not create or push a Git tag, publish a GitHub Release, assign Core
1.0, sign artifacts, prove reproducible builds, provide installers, or retain
artifacts permanently. It does not satisfy the private-corpus threshold or the
twenty-night native CI requirement. Those remain independent release evidence.
