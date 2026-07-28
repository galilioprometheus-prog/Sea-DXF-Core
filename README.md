# SeaCad

SeaCad is a proprietary, clean-room CAD kernel project. Its first release target
is a lossless and resource-bounded DXF core for ASCII and Binary DXF AC1009
through AC1032.

## Current status

Checkpoint M6.5p and the Q1 dependency-policy gate are complete. SeaCad opens
bounded lossless ASCII and Binary DXF AC1009 through AC1032, preserves exact
source bytes under a one-pass SHA-256 identity, indexes section envelopes and
group-zero records, resolves the reviewed text-storage and escape layers, and
writes a separately verified byte-identical Verbatim copy. The shared lazy
HEADER view exposes 166 provenance-backed schema fields, including exact
numeric tuples, flags, dates, elapsed times, and handles, without applying
unreviewed defaults or enum meanings. Broader record semantics, entities,
geometry, transactions, preserve-patch, and canonical writers remain future
milestones.

## Workspace

- `seacad-dxf-core`: bounded source, raw ASCII framing, source identity, typed
  dialect/encoding policy discovery, section/group-0 indexes, and Verbatim
  writer today; broader semantics, topology, and transactions arrive later.
- `seacad-cli`: operational `inspect` and `verify` commands with English or
  Vietnamese human output and stable JSON v1. `corpus` arrives later.
- `seacad-schema-gen`: deterministic provenance-backed schema generation and
  cross-platform generated-output verification.

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

## License

Copyright (c) 2026 SeaCad. All rights reserved. See `LICENSE`.
