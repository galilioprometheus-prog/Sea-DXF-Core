# M13.2c six-native receipt contract

## Purpose

M13.2c verifies the artifacts produced by all six native jobs after they have
passed through GitHub artifact storage. It emits one bounded aggregate receipt
only when the downloaded matrix agrees with the same source checkout.

## Exact matrix

The input root must contain only these package directories:

- `seacad-dxf-core-0.0.0-aarch64-apple-darwin`
- `seacad-dxf-core-0.0.0-aarch64-pc-windows-msvc`
- `seacad-dxf-core-0.0.0-aarch64-unknown-linux-gnu`
- `seacad-dxf-core-0.0.0-x86_64-apple-darwin`
- `seacad-dxf-core-0.0.0-x86_64-pc-windows-msvc`
- `seacad-dxf-core-0.0.0-x86_64-unknown-linux-gnu`

Missing, duplicate, extra, symlinked, or non-directory root entries fail.
Package/version names follow the current workspace version and therefore
require intentional contract updates when that version changes.

## Per-artifact verification

Each `RELEASE_RECEIPT.json` is limited to 1 MiB and must use
`seacad-native-artifact-receipt/v1` with schema version 1, the exact package,
target, requested 40-character lowercase commit, workspace version, and Rust
1.97.1. Unknown JSON fields fail.

Payload paths must be portable, unique, strictly sorted, and exclude the
receipt itself. The package tree must contain exactly those paths. Every entry
must be a real regular file or directory; package traversal is limited to
8,192 entries, depth 20, 4,096 payload files, and 2 GiB of payload per
artifact. Each payload's streamed byte count and SHA-256 must match its
receipt.

The target-specific executable, README, SBOM, and legal manifest are required.
The expected payload set is independently rebuilt from the checkout's complete
`release/legal/` tree. README, SBOM, and every legal file must also match the
checkout by exact byte count and SHA-256. The native executable remains bound
to its target receipt because its bytes legitimately differ across runners.

## Aggregate receipt

`SIX_NATIVE_RECEIPT.json` is create-new and must be directly inside the input
root. Its contract is `seacad-six-native-artifact-receipt/v1`. It records:

- schema/workspace/Rust versions and requested source commit;
- six target/package rows in stable target order;
- exact SHA-256 of each per-artifact receipt;
- per-artifact payload file/byte totals;
- complete matrix payload file/byte totals.

The aggregate does not hash itself or the artifact service's ZIP envelope.

## Workflow and non-claims

The aggregation job depends on all six package jobs, uses exact checkout,
download, and upload action commit pins, then uploads only the aggregate
receipt for 14 days. The workflow remains manual-dispatch and
`contents: read`.

The committed contract does not prove a successful workflow run, preserve an
artifact permanently, create a release/tag, sign payloads, prove reproducible
native builds, satisfy the private-corpus or twenty-night gates, or authorize
Core 1.0.
