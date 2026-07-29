# M9.2d Classic POLYLINE VERTEX Cardinality

Retrieved: 2026-07-30

## Normative boundary

- Autodesk [VERTEX (DXF)](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-0741E831-599E-4CBF-91E1-8ADBCFD6556D.htm)
  documents the thirteen numeric roles retained by M9.2c.
- Autodesk's table marks some fields optional or defaulted but does not grant
  SeaCad permission to discard duplicate, malformed, or inapplicable-looking
  source occurrences.

M9.2d adds neutral occurrence-count evidence before any canonical selection,
default application, vertex-family classification, or semantic validation.

## Implementation contract

`crates/seacad-dxf-core/src/polyline_vertex_card.rs` provides:

- thirteen fixed cards in stable documented-role order for every M9.2c VERTEX;
- explicit `Absent`, `Unique`, or `Multiple { occurrence_count }` state;
- compact source-order members referring back to M9.2c value ordinals without
  copying numeric values;
- independent cardinality and lexical validity, so invalid ASCII occurrences
  still count exactly once;
- thirteen absent cards for a retained VERTEX with no documented numeric
  groups;
- retained source, VERTEX, parent-POLYLINE, sequence-state, role, card, member,
  and underlying-value provenance;
- ASCII/Binary adapters, bounded cancellation, and fail-closed lookup bounds.

## Test evidence

`crates/seacad-dxf-core/tests/polyline_vertex_card_tests.rs` covers:

- identical ASCII/Binary cards and exact member values across every supported
  AC1009-AC1032 dialect;
- all thirteen roles, stable role order, `Absent`, `Unique`, and `Multiple`;
- invalid unique double/int16/int32 occurrences and multiple invalid doubles
  retaining cardinality independently from lexical validity;
- strict POLYLINE/VERTEX/SEQEND record locality and unrelated-group exclusion;
- thirteen absent cards for an empty VERTEX;
- cards retained in closed, interrupted, and unclosed sequences across
  `BLOCKS` and `ENTITIES`;
- source identity, card/member/record lookup bounds, cancellation, and public
  traits.

The fixtures prove physical cardinality and provenance only. They do not claim
that every field is semantically applicable to every `$ACADVER` dialect.

## Non-claims

M9.2d does not select or decode canonical values; apply defaults; interpret
flags, bulge, tangent direction, or signed face indices; classify 2D, 3D,
polygon-mesh, mesh-coordinate, or polyface-face vertices; validate coordinate,
width, or index domains; resolve polyface faces; transform OCS/WCS; assemble
geometry; edit; write; render; or diagnose conformance.

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
- `cargo +1.97.1 test --locked --workspace`: 375 passed, 0 failed, 0
  ignored, including all four focused M9.2d integration tests;
- `git diff --check`: passed.

No dependency manifest or lockfile changed. A focused production-source scan
also found none of the prohibited `unsafe`, panic, unwrap, expect, todo, or
unimplemented constructs in the new module.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/polyline_vertex_card.rs` | 285 | `03cef8d7a74c3dc4055cc4458baad6bcec65221f83416467955f44323939af88` |
| `crates/seacad-dxf-core/src/lib.rs` | 332 | `fed3a05514a0e03811f70257d3a26fd1f0106fa4dcf29ed23f1df50b4a4c6589` |
| `crates/seacad-dxf-core/tests/polyline_vertex_card_tests.rs` | 371 | `515e0ea5f7c3f6ad3cbc223c18a16e2409940bf5c3a8c3d046aee969629fcd94` |
| `docs/IMPLEMENTATION_PLAN.md` | 532 | `8014f030e54920b0bf29b0233d6722105c894c615e23628ccc58847c7f06ec72` |
| `docs/SUPPORT_MATRIX.md` | 445 | `8ffccd16642fab0ab551653e34f45da4ac6b7708f8284afb36872b53b7424ba0` |
