# M9.1h LWPOLYLINE OCS Segment Geometry

Retrieved: 2026-07-29

## Normative boundary

- Autodesk [GetBulge Method](https://help.autodesk.com/cloudhelp/2024/PTB/AutoCAD-ActiveX-Reference/files/GUID-6AC9CF99-7230-4333-859A-1CBACB57B5BA.htm)
  defines bulge as the tangent of one quarter of the included angle for the arc
  from the selected vertex to the next. Negative is clockwise, zero is straight,
  and one is a semicircle.
- Autodesk [Polyline.GetLineSegmentAt](https://help.autodesk.com/cloudhelp/2022/ENU/OARX-ManagedRefGuide/files/OARX-ManagedRefGuide-Autodesk_AutoCAD_DatabaseServices_Polyline_GetLineSegmentAt_int.html)
  identifies the indexed vertex as the segment start and returns the line in
  world coordinates. M9.1h deliberately stays in the polyline's OCS because
  extrusion transformation is not implemented yet.
- Autodesk [LWPOLYLINE (DXF)](https://help.autodesk.com/cloudhelp/2016/ENU/AutoCAD-DXF/files/GUID-748FC305-F3F2-4F74-825A-61F04D757A50.htm)
  identifies vertex coordinates as OCS and group `42` as each vertex's optional
  bulge with default zero.

For start `P0`, end `P1`, chord length `c`, midpoint `m`, left unit normal `n`,
and nonzero bulge `b`, M9.1h derives:

- signed sweep `theta = 4 * atan(b)`;
- radius `r = c * (abs(b) + 1 / abs(b)) / 4`;
- center `m + n * c * (1 / b - b) / 4`.

These equations follow algebraically from Autodesk's quarter-angle definition.
They are SeaCad derivations, not quoted Autodesk implementation formulas.

## Implementation contract

`crates/seacad-dxf-core/src/lightweight_polyline_segment_geometry.rs` provides:

- one lazy geometry directory owning the complete M9.1g topology/semantic graph;
- exact semantic OCS endpoints for zero-bulge straight segments, including
  signed-zero bulge and zero-length straight segments;
- finite derived center, positive radius, signed sweep in radians, original
  bulge, and exact endpoints for nonzero-bulge circular segments;
- counterclockwise positive and clockwise negative orientation;
- typed `StartPositionUnavailable`, `EndPositionUnavailable`,
  `BulgeUnavailable`, `DegenerateArcChord`, and
  `NonFiniteDerivedGeometry` outcomes;
- stable midpoint arithmetic and `hypot` chord calculation;
- a final finiteness/positive-radius/nonzero-sweep barrier preventing NaN or
  infinity from reaching successful geometry;
- shared raw-document, ASCII-document, and Binary-document adapters;
- cancellation before and after constructing the retained segment graph.

Derived center/radius/sweep values are computed binary64 results. They retain
their produced bits through `DxfDouble`, but are not raw source provenance and
are not claimed as bitwise-canonical across different math-library platforms.

## Test evidence

`crates/seacad-dxf-core/tests/lightweight_polyline_segment_geometry_tests.rs`
covers:

- ASCII/Binary parity across all supported AC1009-AC1032 physical dialects;
- exact straight endpoints;
- positive and negative semicircles with exact centers/radii/bulges and signed
  pi sweeps;
- a quarter-circle factor with center, radius, and pi/2 sweep tolerance checks;
- unavailable start, end, and bulge semantics;
- nonzero-bulge zero-chord rejection;
- overflowing endpoint subtraction/intermediate geometry rejection;
- zero-length zero-bulge straight preservation;
- source identity, lookup bounds, cancellation, and public traits.

The parity fixtures are physical decoding evidence only. They do not claim
that LWPOLYLINE is semantically applicable to every `$ACADVER` value.

## Non-claims

M9.1h does not select effective constant/variable widths; validate negative
widths; require declared/observed count agreement; interpret Plinegen; convert
OCS elevation/extrusion to WCS; merge adjacent arcs; establish tolerance-based
coincidence; canonicalize derived floating bits across platforms; edit/write;
render; snap; offset; tessellate; or infer higher topology.

## Required checkpoint gates

- `cargo deny --locked check`
- `cargo +1.97.1 fmt --all -- --check`
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`
- `cargo +1.97.1 clippy --locked --workspace --all-targets -- -D warnings`
- `cargo +1.97.1 test --locked --workspace`
- `git diff --check`

All checkpoint gates passed on 2026-07-29:

- `cargo deny --locked check`: advisories, bans, licenses, and sources passed;
- `cargo +1.97.1 fmt --all -- --check`: passed;
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`: generated
  schema matched the committed artifacts;
- `cargo +1.97.1 clippy --locked --workspace --all-targets -- -D warnings`:
  passed with warnings denied;
- `cargo +1.97.1 test --locked --workspace`: 360 passed, 0 failed, 0
  ignored, including all three focused M9.1h integration tests;
- `git diff --check`: passed.

No dependency manifest or lockfile changed. A focused production-source scan
also found none of the prohibited `unsafe`, panic, unwrap, expect, todo, or
unimplemented constructs in the new module.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/lightweight_polyline_segment_geometry.rs` | 276 | `90a393a9310f37a3b40b1860a798a3ffc323ef99af66987cb0b46b7f6c09ef17` |
| `crates/seacad-dxf-core/src/lib.rs` | 315 | `2444d71872296549c32163bd1373cda5e6463bbd1339ef71d34f774bd4e0a0ae` |
| `crates/seacad-dxf-core/tests/lightweight_polyline_segment_geometry_tests.rs` | 302 | `eda394b070c217573a8f52a870de0a61c35c4aad4e0a6cccda7485adbef69c90` |
| `docs/IMPLEMENTATION_PLAN.md` | 484 | `612a0909aa402a18cf4fe130701d1e99f6a7ec67d37dfa95b709298cd3661d82` |
| `docs/SUPPORT_MATRIX.md` | 399 | `cbe18d90565fe5a4a8dc9b9a9bb5021fd812e8905f85c9516cb827a97f08a32f` |
