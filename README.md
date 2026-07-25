# SeaCad

SeaCad is a proprietary, clean-room CAD kernel project. Its first release target
is a lossless and resource-bounded DXF core for ASCII and Binary DXF AC1009
through AC1032.

## Current status

Milestone M0 is a bootstrap only. No DXF parsing or format-support claim exists
yet.

## Workspace

- `seacad-dxf-core`: future DXF source, semantic, topology, transaction, and
  writer library.
- `seacad-cli`: the `seacad` verification CLI. `inspect`, `verify`, and `corpus`
  arrive in later milestones.

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

## License

Copyright (c) 2026 SeaCad. All rights reserved. See `LICENSE`.
