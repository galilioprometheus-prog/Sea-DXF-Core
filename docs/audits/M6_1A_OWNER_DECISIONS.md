# M6.1a owner decision receipt

Decision date: 2026-07-27

The owner approved the recommended M6 direction with the instruction `làm đi`:

- use a Rust-only schema generator with the already audited pinned Serde stack;
- keep ordered schema family files and begin with three HEADER variables;
- later use one borrowed semantic adapter for ASCII and Binary raw documents;
- treat Fornjot, Truck, vcad, and Open CAD Studio as reference/oracle only for
  DXF Core 1.0, with no copied source or runtime dependency;
- defer geometry-kernel selection until a separate post-semantics milestone;
- retain an out-of-process WASM/WIT plugin design with paged document access;
- require a 1 GiB benchmark before making a large-file support claim.

This checkpoint implements only the first two bullets. It does not introduce a
semantic view, geometry kernel, plugin host, GUI, or large-file support claim.
