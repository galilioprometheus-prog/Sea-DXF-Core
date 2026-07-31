# SeaCad

SeaCad is a proprietary, clean-room CAD kernel project. Its first release target
is a lossless and resource-bounded DXF core for ASCII and Binary DXF AC1009
through AC1032.

## Current status

Release-evidence implementation is complete through M13.2g and entity-semantic
expansion is complete through M14.3c. SeaCad opens bounded lossless ASCII
and Binary DXF AC1009 through AC1032, preserves exact source identity and raw
evidence, exposes the reviewed HEADER/record/entity semantics and geometry,
plans reversible handle edits, writes verified preserve-patch outputs, and
emits strictly reparsed canonical ASCII or Binary framing. Dependency policy,
deterministic CycloneDX inventory, distributable legal files, six-native CI
staging, a redacted 1,000-file/10-GiB corpus release gate, and a manual
six-target native artifact assembly workflow, exact six-artifact receipt
aggregation, a host-independent six-platform SBOM dependency union, actionable
stale-evidence diagnostics, canonical-LF Cargo.lock identity, and one
successful six-native artifact/aggregate workflow receipt are present. Private
corpus achievement, twenty consecutive six-native nightly receipts,
signatures, and final Core 1.0 authorization remain open M13 evidence.
Budget-aware CI runs one complete Linux quality/dependency job for non-document
pushes and reserves the other five native platforms for deliberate manual runs.
The remaining documented entity families are tracked in
`docs/DXF_ENTITY_COMPLETION_PLAN.md`; raw preservation does not imply complete
typed semantics for every entity.

## Workspace

- `seacad-dxf-core`: bounded lossless ASCII/Binary framing, source identity,
  reviewed lazy semantics/topology/geometry, reversible transaction planning,
  and verified preserve-patch/canonical create-new writers.
- `seacad-cli`: operational `inspect` and `verify` commands with English or
  Vietnamese human output sourced from per-locale compile-time catalogs and
  stable JSON v1, plus the separate aggregate-only `seacad-corpus-receipt`
  offline evidence harness.
- `seacad-schema-gen`: deterministic provenance-backed schema generation,
  CycloneDX inventory, legal-bundle generation, cross-platform freshness
  verification, fail-closed native artifact assembly, and six-target receipt
  verification.

The previous CAD workspaces under `D:\Backups` are immutable research inputs.
Production code is not copied from them. Tests, fixtures, and knowledge may be
transferred only after the M1 provenance audit.

## Build

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

The repository pins Rust 1.97.1 with rustfmt, Clippy, and LLVM tools.

## Current CLI

```text
seacad inspect drawing.dxf
seacad verify drawing.dxf
seacad inspect drawing.dxf --lang vi
seacad verify drawing.dxf --json
```

English is the default. `--lang vi` localizes human help and reports. JSON
keys, statuses, and codes remain stable English identifiers. Paths are hidden
unless `--show-path` is explicit. See `docs/CLI_JSON_V1.md` for the complete
contract and `docs/SUPPORT_MATRIX.md` for the exact support boundary.

Private corpus bytes stay outside the repository. The Q2.2 harness reads the
public aggregate-only policy in `corpus/offline-manifest.json` and emits no
paths, filenames, source IDs, or per-file hashes. See
`docs/Q2_2_OFFLINE_CORPUS_RECEIPT_CONTRACT.md`.
M13 release qualification uses the separate threshold policy in
`corpus/release-manifest.json`; policy presence is not evidence that a private
corpus has passed.

## License

Copyright (c) 2026 SeaCad. All rights reserved. See `LICENSE`.
