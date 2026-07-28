# Architecture

## Core boundary

`seacad-dxf-core` owns DXF source access, lossless group records, dialects,
typed semantics, topology, transactions, and writers. It does not own rendering,
selection UX, snapping, CAD commands, geometry booleans, filesystem resolution
for external references, scripting, plugins, or GUI code.

`seacad-cli` is the first consumer and verification shell. It may format human
and versioned JSON reports and aggregate offline corpus receipts, but it must
not implement parsing or semantic rules. The corpus harness delegates every
selected file to the strict DXF core and owns only bounded traversal,
aggregation, and privacy redaction.

## Data flow

```text
file/bytes -> bounded source -> raw records -> indexes -> lazy semantic views
                                      |                       |
                                      +-> verbatim writer     +-> transaction
                                                                  |
                                                       validated new snapshot
```

Raw bytes are the source of truth. Semantic and geometric models are derived,
source-anchored views. Unsupported and malformed content remains explicit and
is never silently dropped.

## Future boundaries

After DXF Core 1.0, a format-neutral command API will serve Luau, Python,
CadLisp, sandboxed WASM plugins, and the GUI. These hosts must depend on the
command/core interfaces; the core must not depend on them.

## Implementation language boundary

DXF Core 1.0 is Rust-first and keeps its parsing, preservation, semantics,
transactions, and writers in safe Rust. C or C++ is not introduced for
speculative performance: streaming layout, allocation counts, indexing, cache
budgets, and algorithms are measured before language-level escape hatches are
considered.

A future native dependency requires either a reproducible benchmark proving a
specific bottleneck or a uniquely capable licensed library that is impractical
to replace. It must be isolated behind a small C ABI or a separate process;
C++ ABI types never cross a SeaCad public boundary. Any `unsafe` code is
confined to a separately audited bridge crate while `seacad-dxf-core` retains
`#![forbid(unsafe_code)]`.

Script and plugin languages do not alter this boundary. Luau, Python, and
CadLisp use the automation API; WASM uses WIT. They cannot mutate parser memory
directly and all CAD edits still pass through validated transactions.
