# M9.1f LWPOLYLINE Record Semantics

Retrieved: 2026-07-29

## Normative boundary

- Autodesk [LWPOLYLINE (DXF)](https://help.autodesk.com/cloudhelp/2016/ENU/AutoCAD-DXF/files/GUID-748FC305-F3F2-4F74-825A-61F04D757A50.htm)
  lists group `90` as the number of vertices without marking it optional.
- The same table defines flags `70` as bit-coded with default zero and names
  only bit `1` as Closed and bit `128` as Plinegen.
- Elevation `38`, thickness `39`, and constant width `43` are optional with
  zero defaults. Extrusion `210/220/230` is optional with default `(0, 0, 1)`.
- The table says constant width `43` is not used when variable widths `40/41`
  are set. It also says each variable-width field is not used when constant
  width is set. M9.1f therefore records coexistence without inventing a
  precedence rule for malformed or contradictory input.
- Vertex coordinates `10/20` have one entry per vertex. M9.1f compares the
  usable declared count with the exact number of retained group-10 anchors,
  matching the deterministic M9.1c grouping boundary.

The Autodesk table does not publish a numeric conformance range for group `90`
beyond its signed-i32 wire type. Negative values therefore remain exact and can
compare as mismatched; they are not silently rejected by this milestone.

## Implementation contract

`crates/seacad-dxf-core/src/lightweight_polyline_record_semantic.rs` provides:

- one lazy semantic directory owning the complete M9.1e card/evidence graph;
- required vertex-count semantics with typed missing, invalid-number, and
  multiple-value failures;
- documented zero defaults for absent flags, elevation, thickness, and
  constant width, plus independent `(0, 0, 1)` extrusion defaults;
- exact signed-i32 count and signed-i16 flag values with raw provenance;
- helpers for only the documented Closed and Plinegen bits;
- observed vertex count from every retained group-10 anchor;
- `Matched`, `Mismatched`, or `NotComparable` count comparison without hiding
  the original semantic field;
- neutral width evidence states for no explicit width, constant only, variable
  only, or simultaneous constant and variable occurrences;
- shared raw-document, ASCII-document, and Binary-document adapters;
- lookup validation and cancellation before semantic directory construction.

## Test evidence

`crates/seacad-dxf-core/tests/lightweight_polyline_record_semantic_tests.rs`
covers:

- ASCII/Binary parity across all supported AC1009-AC1032 physical dialects;
- exact signed-zero and floating-bit preservation;
- matched declared/observed count and Closed/Plinegen helpers;
- missing required count and every documented record default;
- duplicate count, invalid flags/elevation, duplicate thickness, and partial
  invalid/defaulted/explicit extrusion;
- mismatched count without domain guessing;
- all four constant/variable width occurrence shapes;
- source/raw provenance, lookup bounds, cancellation, and public traits.

The parity fixtures are physical decoding evidence only. They do not claim
that LWPOLYLINE is semantically applicable to every `$ACADVER` value.

## Review-size deviation

The self-contained production module is 564 lines, 64 lines above the preferred
500-line upper target. Keeping mixed integer/double projection, count
comparison, and the width non-precedence boundary together makes the reviewed
record contract directly inspectable. No generated, test, or audit file hides
production behavior.

## Non-claims

M9.1f does not reject negative counts or widths; require observed/declared count
agreement; interpret undocumented flag bits; infer geometry from Closed or
Plinegen; choose constant/variable width precedence; publish effective segment
widths; validate extrusion; validate version/subclass applicability; transform
OCS; assemble straight or bulged segments; edit/write; render; snap; or infer
topology.

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
- workspace tests: 354 passed, 0 failed, 0 ignored, including all 3 focused
  M9.1f integration tests;
- `git diff --check` passed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/lightweight_polyline_record_semantic.rs` | 564 | `3985d54776376ed5c22f73d6cc8b9c666d9b63ac2089a3ef00947897e2617d5a` |
| `crates/seacad-dxf-core/src/lib.rs` | 302 | `9678d3bb081abd3207aea7de45fbf434e94fa85e09de5b16ce356842e353b657` |
| `crates/seacad-dxf-core/tests/lightweight_polyline_record_semantic_tests.rs` | 387 | `fd4ad9686989c060c4b4eaf7292a72142ea5838c3569df74b771861e8ea18782` |
| `docs/IMPLEMENTATION_PLAN.md` | 459 | `aa8329ed5ade6a6364f3160b1b8b1cc72a7a0e92a3f1ace4e68eadb6f78b2cc7` |
| `docs/SUPPORT_MATRIX.md` | 375 | `15dcbc300c171d397b40990b9761adfe298bdabb06873ffd4b57a5aa20c2f291` |
