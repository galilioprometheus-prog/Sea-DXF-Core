# M13.1a Locked SBOM and Notice Inventory Audit

## Outcome

M13.1a adds one reproducible CycloneDX 1.6 inventory for the complete locked
SeaCad Cargo workspace graph and makes freshness/completeness a required
six-native CI gate.

## Contract

- `cargo metadata --locked --format-version 1` is the authoritative resolved
  graph.
- The inventory includes all 29 locked packages: 26 crates.io packages and
  three SeaCad workspace packages.
- Every crates.io component has a stable Cargo package URL, declared SPDX
  license expression, and exact SHA-256 checksum parsed from `Cargo.lock`.
- Every workspace component has a stable host-independent package reference
  and `LicenseRef-SeaCad-Proprietary`.
- Every resolved dependency node and the aggregate workspace root have sorted,
  duplicate-free dependency references.
- Every third-party package/version must have an exact table row in
  `THIRD_PARTY_NOTICES.md`.
- The SBOM embeds the exact `Cargo.lock` SHA-256 and Rust `1.97.1` baseline.
- Output excludes timestamps, absolute paths, Cargo registry cache paths,
  usernames, machine identifiers, and network-derived mutable metadata.
- `--write` performs intentional regeneration; `--check` fails closed when the
  committed artifact is missing, stale, incomplete, unlicensed, unnotified, or
  sourced outside the reviewed crates.io registry.
- Baseline, supplemental smoke, and scheduled full-quality CI jobs all run
  `--check`.

## Current external evidence gap

GitHub Actions scheduled runs `30425646262` on 2026-07-29 and `30516632316`
on 2026-07-30 are the first two consecutive successful full-quality runs
across Linux x64/ARM64, Windows x64/ARM64, and macOS x64/ARM64. The Q2.1
promotion rule requires twenty consecutive successful nightly runs, so the
current release evidence is exactly 2/20 and remains open.

No private corpus receipt establishing 1,000 selected files and 10 GiB has
been supplied. The committed Q2.2 values remain traversal ceilings rather than
achieved corpus evidence.

## Non-claims

M13.1a closes deterministic locked dependency and notice inventory only. It
does not validate the CycloneDX document with an external schema validator,
package standalone third-party license texts, construct or sign release
archives, prove reproducible native binaries, satisfy the private corpus
threshold, promote staged native jobs, assign a Core 1.0 version, or authorize
a release.

## Gates

- `cargo deny --locked check`
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`
- `cargo +1.97.1 run --locked -p seacad-schema-gen --bin seacad-release-evidence -- --check`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace --quiet` (592 passed, 0 failed, 0 ignored)
- `git diff --check`

All gates passed.

## Artifact receipt

| Artifact | Lines | SHA-256 |
| --- | ---: | --- |
| `.github/workflows/ci.yml` | 118 | `60f9e7841ace57904315762efce7bc74ec1acb405987368da662a16fb6238996` |
| `crates/seacad-schema-gen/Cargo.toml` | 27 | `65a2ce7346e0e3a3443d1289daf80c9b0def61b9b3c06b1afe203219a81d858b` |
| `crates/seacad-schema-gen/src/bin/seacad-release-evidence.rs` | 533 | `9717c34520277e5ce79ace852850a9286c7d8041bb3a42f9abe5f1ed5b29b7f1` |
| `release/sbom.cdx.json` | 889 | `f6f38e23a90d55c2af0d6a2a4d1072cced759c7cd315c162ecde3a6255977e14` |
| `docs/IMPLEMENTATION_PLAN.md` | 1250 | `370b909f2a05be858fff85da0ae07e6f906b47479078614a28a92c990d394c6a` |
| `docs/SUPPORT_MATRIX.md` | 1018 | `26c32edea8c0beecc8dbc38ecf42597e8d62c37366e40dd780955543654f3f15` |
| `docs/TOOLCHAIN.md` | 116 | `03361d62967ed27d81b6b33a63bb29c488fd101e3b36d478f8cffc8f61347874` |
