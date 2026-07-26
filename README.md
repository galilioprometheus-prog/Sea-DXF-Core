# SeaCad

SeaCad is a proprietary, clean-room CAD kernel project. Its first release target
is a lossless and resource-bounded DXF core for ASCII and Binary DXF AC1009
through AC1032.

## Current status

Milestone M3 is complete. SeaCad can frame raw ASCII DXF losslessly with
bounded Strict/Compatible reading, preserve exact source bytes, compute a
SHA-256 source identity, and write a separately verified byte-identical
Verbatim copy. It does not yet claim version, section, entity, or geometry
semantics; those begin at M4.

## Workspace

- `seacad-dxf-core`: bounded source, raw ASCII framing, source identity, and
  Verbatim writer today; semantics, topology, and transactions arrive later.
- `seacad-cli`: operational `inspect` and `verify` commands with English or
  Vietnamese human output and stable JSON v1. `corpus` arrives later.
- `seacad-schema-gen`: reserved for provenance-backed schema generation at M6.

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
