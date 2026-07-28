# Q2.2 offline corpus manifest and receipt harness

Date: 2026-07-28

## Scope

Q2.2 adds a bounded cross-platform harness for strict offline corpus
verification. The committed manifest is aggregate-only policy. The generated
receipt contains platform identity, public limits, aggregate scan counts, and
aggregate stable error-code counts.

This checkpoint changes no DXF parser, public core API, schema row, main
`seacad` CLI contract, dependency manifest, dependency policy, or `Cargo.lock`
entry. It does not increase format, dialect, semantic, performance, corpus
scale, or large-file support claims.

## Privacy boundary

Neither the manifest nor receipt contains:

- a corpus root, relative path, filename, or customer identity;
- a source ID, per-file hash, per-file size, or per-file outcome;
- a timestamp, username, hostname, or environment value;
- private DXF bytes.

The harness emits only aggregate selected bytes and counts. Filesystem and
manifest failures use static path-redacted messages. The ignored
`corpus-private/` directory remains the recommended local input location.
Inputs are opened read-only; the harness never writes beside or modifies them.

The read-only legacy workspace was queried only for document-level context and
the names of top-level manifest fields. No legacy source, test, fixture,
private path, hash, or corpus byte was copied, translated, or committed. All
Q2.2 test DXF inputs are newly generated temporary SeaCad fixtures.

## Contract and limits

`seacad-corpus-receipt` accepts exactly a manifest path and corpus root. Version
1 selects regular `.dxf` files case-insensitively, rejects symlinks, verifies
canonical paths remain under the root, and sorts directory entries before
processing.

The committed policy and hard implementation ceilings are:

- depth: 32;
- directory entries: 10,000;
- selected files: 1,000;
- aggregate selected bytes: 10 GiB;
- per-file parser profile: existing `Safe`;
- parser mode: existing `Strict`.

An empty corpus is not passing. Invalid files produce a completed redacted
receipt with `status="failed"`. Invalid manifests, unreadable filesystem
objects, symlinks, arithmetic overflow, or limit breaches fail closed with a
stable harness code. The 1,000-file and 10-GiB values are ceilings only, not
achieved evidence. Q2.2 deliberately adds no aggregate content commitment;
final native corpus receipts remain M13 work.

## Verification

The focused Q2.2 test target passed 6 tests covering:

- committed-manifest compatibility;
- aggregate ASCII/Binary physical counts;
- path, filename, source-ID, and hash redaction;
- invalid and empty corpus failure receipts;
- depth, entry, file, and byte limits;
- command output and exit behavior.

The required final gates passed on Windows x64:

| Command | Result |
| --- | --- |
| `cargo deny --locked check` | passed: advisories, bans, licenses, sources |
| `cargo +1.97.1 fmt --all -- --check` | passed |
| `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check` | passed |
| `cargo +1.97.1 clippy --workspace --all-targets -- -D warnings` | passed |
| `cargo +1.97.1 test --workspace` | passed: 245 tests, 0 failed |
| `git diff --check` | passed |

Native execution on the other five staged platforms remains post-push CI
evidence and is not claimed by this local receipt.

## Diff and dependency boundary

The new Rust binary has 432 non-test lines and 196 test lines. Before this
receipt, tracked documentation changes contained 48 insertions and 13
deletions. Q2.2 adds one Rust binary, one 16-line aggregate-only manifest, one
47-line contract, and this receipt.

No dependency was added or changed. `Cargo.toml`, `Cargo.lock`, `deny.toml`, all
files under `crates/seacad-dxf-core/`, and all files under `schema/` remain
unchanged.

## Artifact SHA-256

| Artifact | SHA-256 |
| --- | --- |
| `crates/seacad-cli/src/bin/seacad-corpus-receipt.rs` | `a33b970a13db7264fc1734bb630ad69a7b782683dd6d3357430ae24ec3ed53c8` |
| `corpus/offline-manifest.json` | `4b34ccb100da22874c373f63fb8ceef46522231865879b43000d15c5062e6587` |
| `docs/Q2_2_OFFLINE_CORPUS_RECEIPT_CONTRACT.md` | `fbee2b881dff76ff672221e50c1b8aa124c66a3b17f981b54c78f26620439796` |
| `README.md` | `b12a36f0b2b8f5c2880c33f1b937dc40c5953e55cbd3dd831db26a7bc3948015` |
| `docs/ARCHITECTURE.md` | `eb7cdb71556f02fc80db758751ccb7de809847428c877719611feb77505a579f` |
| `docs/IMPLEMENTATION_PLAN.md` | `93dc63abd421ae81c2c6e8dce804c0c70753b27040c01f2cf3520199df52dc55` |
| `docs/SUPPORT_MATRIX.md` | `bdec0ddcfb614560a387d15261b26aa5a04a1161ec438a3c6c570fe18ff32c63` |
| `docs/TOOLCHAIN.md` | `a6141dc44619fe7f40dff9bf42d8a20d484a5d2a799d175c78954ae9a2c43925` |
| unchanged `Cargo.lock` | `f48f459ed5a7c7b9fb9d184d78099c5c6b31b260b3d13df632dbf539ea07ad70` |
