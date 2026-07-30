# M13.1b Distributable Legal Bundle Audit

## Outcome

M13.1b packages the complete legal-file surface of the current locked Cargo
graph into a deterministic, host-independent directory whose content and file
set are enforced by the existing release-evidence CI gate.

## Contract

- The bundle includes canonical-LF project `LICENSE`, `NOTICE`, and
  `THIRD_PARTY_NOTICES.md`, independent of checkout line endings.
- For each of the 26 crates.io packages, the generator reads the Cargo-extracted
  source root selected by `cargo metadata --locked`.
- Every regular root filename beginning with `LICENSE`, `LICENCE`, `COPYING`,
  `UNLICENSE`, or `NOTICE` is retained byte-for-byte.
- A package with no matching legal artifact fails closed.
- The current graph yields 54 package artifacts plus three project files.
- The bundle explicitly retains `encoding_rs` `LICENSE-WHATWG`, `unicode-ident`
  `LICENSE-UNICODE`, and `memchr` `COPYING`/`UNLICENSE`.
- `manifest.json` records package/version, declared SPDX expression, exact
  `Cargo.lock` crate SHA-256, and every file's relative path, byte count, and
  SHA-256.
- The manifest records the exact complete `Cargo.lock` SHA-256.
- Package/version path tokens are restricted to portable ASCII characters.
- The committed tree walk rejects symlinks, non-regular entries, missing files,
  unexpected stale files, and any byte mismatch.
- Output paths, ordering, JSON formatting, and raw artifact bytes are stable
  across the six CI platforms.

## Evidence

- The generated manifest accounts 26 packages, 54 package legal artifacts,
  three project files, and one manifest: 58 files total.
- The committed bundle is approximately 300 KiB and contains no build output,
  source code, registry path, username, or host identifier.
- Two independent in-process builds compare equal at both raw-file map and
  manifest byte levels.
- Negative tests prove one unexpected file and one modified expected file fail
  with distinct typed release-evidence errors.
- The normal `--check` command proves the committed file set and all bytes equal
  the currently selected locked crate sources.

## Non-claims

M13.1b packages legal artifacts for the current locked Cargo graph. It does not
provide legal advice, replace upstream license obligations, validate an
installer, build or sign native archives, prove reproducible binaries, satisfy
the private corpus threshold, complete twenty consecutive native nightlies,
assign a Core 1.0 version, or authorize a release.

## Gates

- `cargo deny --locked check`
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`
- `cargo +1.97.1 run --locked -p seacad-schema-gen --bin seacad-release-evidence -- --check`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace --quiet` (594 passed, 0 failed, 0 ignored)
- `git diff --check`

All gates passed.

## Artifact receipt

The legal manifest is the transitive receipt for all 57 payload files. Its own
receipt and the generator/documentation receipts follow.

| Artifact | Lines | SHA-256 |
| --- | ---: | --- |
| `.gitattributes` | 18 | `175b7b62dba7ddae0084219b33bbdb1639f0e6c806372298b33dc5c82c714c92` |
| `crates/seacad-schema-gen/src/bin/seacad-release-evidence.rs` | 940 | `9ccd832c22908e188a4ec1f956347f2b62dc30327572e44a90f04463f348c4f4` |
| `release/legal/manifest.json` | 501 | `4a3890e40eb54ebe8d23c005c9dd06ccfb3044ef3964cedcda5445b4ade9ccf7` |
| `docs/IMPLEMENTATION_PLAN.md` | 1264 | `df8aebf40594f5f8eaaff65bdfe93f3dcbcd7696662b51301099a944de1a86fb` |
| `docs/SUPPORT_MATRIX.md` | 1026 | `61f81ca64f13b56ea73d4fdbf60fbad69b371fef5adaa2dd5c1ea49ca2cbc08f` |
| `docs/TOOLCHAIN.md` | 135 | `ad42ec3d1cf670f603cbb4f1ee9dc060705c9a1664be1fab350e071cf32aa564` |
