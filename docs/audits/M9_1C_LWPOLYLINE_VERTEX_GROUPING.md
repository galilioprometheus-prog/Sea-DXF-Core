# M9.1c LWPOLYLINE Vertex Grouping

Retrieved: 2026-07-29

## Normative boundary

- Autodesk [LWPOLYLINE (DXF)](https://help.autodesk.com/cloudhelp/2016/ENU/AutoCAD-DXF/files/GUID-748FC305-F3F2-4F74-825A-61F04D757A50.htm)
  identifies repeated OCS vertex coordinates `10/20`, repeated start/end widths
  `40/41`, repeated bulges `42`, and vertex identifiers `91` as per-vertex
  values.
- The table identifies group `10` as the X component of each repeated vertex
  coordinate. M9.1c uses each exact retained group-10 occurrence as the only
  conservative vertex anchor.

The Autodesk table describes valid entity fields, not recovery from arbitrary
malformed ordering. M9.1c therefore publishes its fail-closed grouping rule:
subsequent vertex-scoped evidence belongs to the current group-10 anchor until
the next group `10`; evidence before any anchor remains orphaned. No missing
anchor is invented.

## Implementation contract

`crates/seacad-dxf-core/src/lightweight_polyline_vertex.rs` provides:

- one owned grouping directory over retained M9.1a floating and M9.1b integer
  evidence with source-identity and record-alignment checks;
- one vertex entry per exact OCS-X group-10 anchor, ordered by raw occurrence;
- six fixed cards per vertex for OCS X/Y, start/end width, bulge, and identifier;
- `Absent`, `Unique`, or `Multiple { occurrence_count }` independently from
  the lexical result retained by the underlying value;
- compact card members that resolve back to the exact floating or integer value;
- source-ordered orphan members for vertex-scoped values before the first X;
- entity-level elevation, thickness, constant width, extrusion, declared count,
  and flags excluded from vertex grouping but retained by M9.1a/b;
- record, vertex, card, member, and orphan lookup without copying raw values;
- cancellation checks and fallible allocation throughout construction.

## Test evidence

`crates/seacad-dxf-core/tests/lightweight_polyline_vertex_tests.rs` covers:

- ASCII/Binary parity across all supported AC1009-AC1032 physical dialects;
- two anchored vertices with fixed role order and exact floating/integer values;
- absent and duplicate per-vertex roles;
- floating and integer lexical failures retained behind unique cards;
- mixed floating/integer pre-anchor orphan order and typed resolution;
- entity-level values excluded from vertex cards;
- a recognized record with no anchors producing zero vertices and zero orphans;
- record/card/member lookup, cancellation, and public traits.

The parity fixtures are physical decoding evidence only. They do not claim that
LWPOLYLINE is semantically applicable to every `$ACADVER` value.

## Review-size deviation

The self-contained production module is 535 lines, 35 lines above the preferred
500-line upper target. Keeping the six-card builder, public lookup contract,
and malformed orphan policy together makes the evidence boundary directly
reviewable; splitting types from construction would not reduce milestone scope
or behavior. No generated, test, or audit file is used to hide production code.

## Non-claims

M9.1c does not apply defaults; select canonical members; require or validate X/Y;
compare declared and observed vertex counts; interpret flags, bulges, widths, or
identifiers; require identifier uniqueness; validate version applicability;
transform OCS; assemble straight or arc segments; close a polyline; edit/write;
render; snap; or infer topology.

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
- workspace tests: 345 passed, 0 failed, 0 ignored, including all 3 focused
  M9.1c integration tests;
- `git diff --check` passed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/lightweight_polyline_vertex.rs` | 535 | `3ce6f5c4dc94566d713677c749c2b71779b17597359288bba7e228552718081c` |
| `crates/seacad-dxf-core/src/lib.rs` | 282 | `a15f9c01651dca731f14fc9dbadb13a0c2e519fc9820d8931ff7dd525948536e` |
| `crates/seacad-dxf-core/tests/lightweight_polyline_vertex_tests.rs` | 386 | `58e8bc19302c6c131cfb2babd61a5316062e5bcc43e75fec05acf365481a0f72` |
| `docs/IMPLEMENTATION_PLAN.md` | 426 | `859bc12d48eeb174687e183116b2dce435547eef58d5e25fbef206e92e95b5bb` |
| `docs/SUPPORT_MATRIX.md` | 338 | `6f2da1f470a50557343b6b72ef9e240682d7330e1e8c942a3d68059a0e907e9e` |
