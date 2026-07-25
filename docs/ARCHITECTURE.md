# Architecture

## Core boundary

`seacad-dxf-core` owns DXF source access, lossless group records, dialects,
typed semantics, topology, transactions, and writers. It does not own rendering,
selection UX, snapping, CAD commands, geometry booleans, filesystem resolution
for external references, scripting, plugins, or GUI code.

`seacad-cli` is the first consumer and verification shell. It may format human
and versioned JSON reports, but it must not implement parsing or semantic rules.

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
