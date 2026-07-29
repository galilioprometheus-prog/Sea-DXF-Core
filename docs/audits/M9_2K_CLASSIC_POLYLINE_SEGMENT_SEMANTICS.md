# M9.2k Classic POLYLINE Segment Semantics

Retrieved: 2026-07-30

## Normative boundary

Autodesk [POLYLINE (DXF)](https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-ABF6B778-BE20-4B49-9B58-A94E64CEFFF3.htm)
and [VERTEX (DXF)](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-0741E831-599E-4CBF-91E1-8ADBCFD6556D.htm)
place 2D locations in OCS, 3D locations in WCS, and publish parent default
widths separately from VERTEX-local widths, bulge, and tangent direction.

## Implementation contract

`crates/seacad-dxf-core/src/polyline_segment_semantic.rs` lazily binds every
M9.2j segment to its parent record and exact start/end VERTEX semantics. It
exposes the OCS/WCS boundary, endpoint tuples, start-vertex local widths,
bulge/tangent, and parent default widths while preserving independent invalid
states and choosing no effective-width precedence.

## Test evidence

`crates/seacad-dxf-core/tests/polyline_segment_semantic_tests.rs` covers
ASCII/Binary parity across all supported dialects, exact double bits, OCS/WCS
selection, endpoint and field binding, independent invalidity, no width
precedence, lookup bounds, cancellation, source identity, and public traits.

## Non-claims

M9.2k does not choose effective widths, derive straight/arc geometry, require
usable coordinates, apply elevation/extrusion transforms, tessellate, edit,
write, render, or diagnose conformance.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, formatting,
generated-schema drift, workspace Clippy with warnings denied, 398 workspace
tests with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/polyline_segment_semantic.rs` | 201 | `9dfb4435f9d9be00237984aa3bd64de2af27c82f2e8932096f087dd0fbe42164` |
| `crates/seacad-dxf-core/src/lib.rs` | 368 | `db9b6a64a2a7f57456efe5fbfa4016c3ee2af24c4ca70b2a1c19193bc2537621` |
| `crates/seacad-dxf-core/tests/polyline_segment_semantic_tests.rs` | 250 | `a814a154b4c20c8e18dbf62f82af4cb60f4cdd44e72367911e30e32502abcdde` |
| `docs/IMPLEMENTATION_PLAN.md` | 612 | `25666899b587ef7be0ebd50f49b5d02c954b30b70c1a8e20f638fc14795277bf` |
| `docs/SUPPORT_MATRIX.md` | 515 | `220f494af83f3c1f7282f2c98d1d3aa2bfe840ee9bc5b26dad0f22e866b427c4` |
