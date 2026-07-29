# M9.1g LWPOLYLINE Segment Topology

Retrieved: 2026-07-29

## Normative boundary

- Autodesk [LWPOLYLINE (DXF)](https://help.autodesk.com/cloudhelp/2016/ENU/AutoCAD-DXF/files/GUID-748FC305-F3F2-4F74-825A-61F04D757A50.htm)
  defines flag bit `1` as Closed and defaults absent flags to zero.
- Autodesk [GetWidth Method](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-LT-ActiveX-Reference/files/GUID-60D35D7B-9328-4FD2-A4AB-20EDB5991F6F.htm)
  states that a closed polyline has as many segments as vertices while an open
  polyline has one fewer, and describes widths for the segment beginning at a
  selected vertex index.
- Autodesk [GetBulge Method](https://help.autodesk.com/cloudhelp/2024/PTB/AutoCAD-ActiveX-Reference/files/GUID-6AC9CF99-7230-4333-859A-1CBACB57B5BA.htm)
  defines bulge at a selected vertex as the tangent of one quarter of the arc's
  included angle between that vertex and the next. Negative values are
  clockwise, zero is straight, and one is a semicircle.
- Autodesk [Polyline.GetLineSegmentAt](https://help.autodesk.com/cloudhelp/2022/ENU/OARX-ManagedRefGuide/files/OARX-ManagedRefGuide-Autodesk_AutoCAD_DatabaseServices_Polyline_GetLineSegmentAt_int.html)
  identifies the index parameter as the start vertex of the segment.

These sources establish topology and start-vertex ownership. They do not make
malformed or duplicate flags canonical. M9.1g therefore adds a closing segment
only for one lexically usable flag value with the Closed bit set. Consecutive
adjacency remains guaranteed independently from closure evidence.

## Implementation contract

`crates/seacad-dxf-core/src/lightweight_polyline_segment.rs` provides:

- one immutable segment directory owning the complete M9.1c grouping and M9.1d
  lazy vertex-semantic graph;
- closure states that distinguish defaulted open, explicit open, explicit
  closed, invalid flags, and multiple flags;
- one source-ordered consecutive segment for every adjacent retained vertex
  pair;
- one last-to-first segment only for proven closed records with at least one
  retained vertex, preserving the documented `n` count for a one-vertex
  self-closing record;
- exact global and record-local segment ordinals plus start/end vertex ordinals;
- lazy start/end OCS semantic positions;
- start-vertex local width and bulge semantics, including signed-zero straight,
  nonzero arc-factor, and indeterminate states;
- record, segment, range, and semantic lookup without geometry allocation;
- cancellation checks and fallible segment allocation.

M9.1g also adds read-only accessors from the M9.1c vertex directory to its
retained floating and integer evidence so closure is derived without rebuilding
the evidence graph.

## Test evidence

`crates/seacad-dxf-core/tests/lightweight_polyline_segment_tests.rs` covers:

- ASCII/Binary parity across all supported AC1009-AC1032 physical dialects;
- defaulted-open `n-1` and explicitly closed `n` segment counts;
- exact consecutive and closing endpoint ordinals;
- signed-zero straight, positive/negative arc bulges, OCS endpoints, and local
  width values from the start vertex;
- invalid and duplicate flags producing only guaranteed consecutive segments;
- invalid bulge producing an indeterminate segment shape;
- open single-vertex zero-segment and closed single-vertex self-segment cases;
- source identity, record/range/segment lookup, cancellation, and public traits.

The parity fixtures are physical decoding evidence only. They do not claim
that LWPOLYLINE is semantically applicable to every `$ACADVER` value.

## Non-claims

M9.1g does not select an effective constant or variable width; reject negative
widths; require declared/observed count agreement; interpret Plinegen as
geometry; compute bulge angle, center, radius, chord, sweep, or tangent; reject
zero-length/self segments; transform OCS to WCS; validate extrusion, version,
subclass, or geometry; edit/write; render; snap; or infer higher topology.

## Required checkpoint gates

- `cargo deny --locked check`
- `cargo +1.97.1 fmt --all -- --check`
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`
- `cargo +1.97.1 clippy --locked --workspace --all-targets -- -D warnings`
- `cargo +1.97.1 test --locked --workspace`
- `git diff --check`

All checkpoint gates passed on 2026-07-29:

- dependency policy: advisories, bans, licenses, and sources all `ok`;
- formatting and generated-schema drift checks passed;
- workspace Clippy passed with warnings denied;
- workspace tests: 357 passed, 0 failed, 0 ignored, including all 3 focused
  M9.1g integration tests;
- `git diff --check` passed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/lightweight_polyline_segment.rs` | 498 | `c2fe6ebf088374816c3e846a1f607b6211d6335463059114638516ab5f8f7c68` |
| `crates/seacad-dxf-core/src/lightweight_polyline_vertex.rs` | 545 | `4ad8be32f6ed0020f07e5e59f5406e815bf953dfd6f088fb1593b38ab14d6458` |
| `crates/seacad-dxf-core/src/lib.rs` | 309 | `110fc5b3e2cb88451db52700a36fb0e70a9d8cf36ff2556ba02a6a79101fb701` |
| `crates/seacad-dxf-core/tests/lightweight_polyline_segment_tests.rs` | 373 | `c9f363e1fdf34a5d9f33f780f762d5bc2338acf38032592172b7e866bd9f395e` |
| `docs/IMPLEMENTATION_PLAN.md` | 472 | `95986854a0ed4055d4db71b10f2e335fa4e39a358bc60b43c581aff60b14e397` |
| `docs/SUPPORT_MATRIX.md` | 388 | `c10db2ab5e19d12fefb364538af6e28aa5ed9282524676c10d198f7f55f45e3a` |
