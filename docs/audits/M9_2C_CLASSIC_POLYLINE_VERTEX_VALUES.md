# M9.2c Classic POLYLINE VERTEX Values

Retrieved: 2026-07-30

## Normative boundary

- Autodesk [VERTEX (DXF)](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-0741E831-599E-4CBF-91E1-8ADBCFD6556D.htm)
  documents location groups `10/20/30`, widths `40/41`, bulge `42`, curve-fit
  tangent direction `50`, flags `70`, polyface indices `71`-`74`, and vertex
  identifier `91`.
- Autodesk [Group Code Value Types Reference](https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-2553CF98-44F6-4828-82DD-FE3BC7448113.htm)
  defines `10`-`59` as double-precision families, `60`-`79` as signed 16-bit
  integers, and `90`-`99` as signed 32-bit integers.

M9.2c retains those thirteen roles exactly as numeric wire evidence. Published
meanings, defaults, flags, coordinate systems, and index-sign behavior are
recorded for provenance but are not yet applied as semantic policy.

## Implementation contract

`crates/seacad-dxf-core/src/polyline_vertex_value.rs` provides:

- one entry for every VERTEX retained by M9.2a, including vertices in closed,
  interrupted, and unclosed sequences;
- seven exact binary64 roles, five exact signed-16-bit roles, and one exact
  signed-32-bit role without numeric-domain merging;
- source-order occurrences, duplicates, raw groups, raw spans, exact Binary
  values, and typed ASCII syntax/range failures;
- an empty slice for retained VERTEX records without documented numeric groups;
- strict record locality so POLYLINE, adjacent VERTEX, and SEQEND groups cannot
  enter a VERTEX value slice;
- retained parent-POLYLINE identity and sequence-local vertex ordinal;
- retained M9.2a topology, source identity, raw-ordinal, parent-POLYLINE, and
  group-occurrence lookups, ASCII/Binary adapters, and bounded cancellation.

## Test evidence

`crates/seacad-dxf-core/tests/polyline_vertex_value_tests.rs` covers:

- all thirteen roles with exact ASCII/Binary parity across every supported
  AC1009-AC1032 dialect;
- signed-zero and ordinary binary64 bit preservation, signed-int16 values,
  and the signed-int32 maximum;
- duplicate doubles, double syntax/range failures, int16 range failure and
  minimum, int32 range failure, and an unrelated numeric group;
- exclusion of same-numbered groups from the parent POLYLINE and SEQEND;
- empty VERTEX value slices;
- retained values for closed, interrupted, and unclosed sequences in both
  `BLOCKS` and `ENTITIES` sections;
- source identity, parent and vertex lookup bounds, cancellation, and public
  traits.

The fixtures prove physical decoding and record locality only. They do not
claim that every field is semantically applicable to every `$ACADVER` dialect.

## Non-claims

M9.2c does not select canonical occurrences; apply width or bulge defaults;
interpret flags, bulge, tangent direction, or negative-index edge visibility;
classify 2D, 3D, polygon-mesh, mesh-coordinate, or polyface-face vertices;
validate coordinate/width/index domains; resolve polyface faces; transform
OCS/WCS; assemble geometry; edit; write; render; or diagnose conformance.

## Required checkpoint gates

- `cargo deny --locked check`
- `cargo +1.97.1 fmt --all -- --check`
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`
- `cargo +1.97.1 clippy --locked --workspace --all-targets -- -D warnings`
- `cargo +1.97.1 test --locked --workspace`
- `git diff --check`

All checkpoint gates passed on 2026-07-30:

- `cargo deny --locked check`: advisories, bans, licenses, and sources passed;
- `cargo +1.97.1 fmt --all -- --check`: passed;
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`: generated
  schema matched the committed artifacts;
- `cargo +1.97.1 clippy --locked --workspace --all-targets -- -D warnings`:
  passed with warnings denied;
- `cargo +1.97.1 test --locked --workspace`: 371 passed, 0 failed, 0
  ignored, including all four focused M9.2c integration tests;
- `git diff --check`: passed.

No dependency manifest or lockfile changed. A focused production-source scan
also found none of the prohibited `unsafe`, panic, unwrap, expect, todo, or
unimplemented constructs in the new module.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/polyline_vertex_value.rs` | 381 | `392974a0a581e61f33181462f6b91ccf6905069c24d73607118f248e8d8a2d45` |
| `crates/seacad-dxf-core/src/lib.rs` | 327 | `6fcb7dcb18106c19b33840d9ceb9b18c1a8b67422f4121b210b0c1da0b4bb27a` |
| `crates/seacad-dxf-core/tests/polyline_vertex_value_tests.rs` | 328 | `0abc134c51c53e4e98f4a5a5ecf74b40338fedd7c00ab92bb727aacb7e4f3121` |
| `docs/IMPLEMENTATION_PLAN.md` | 521 | `b1e7a545eacf95183e201a01ffb9ad2290656170e5987470853c6bd4874180c4` |
| `docs/SUPPORT_MATRIX.md` | 436 | `1229c6e0213011e188e6c3931eccdaee999085223b3ace43ef880d7a0fa8cbee` |
