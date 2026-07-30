# M9.2t Classic POLYLINE Polygon-Mesh Smoothing Metadata

Retrieved: 2026-07-30

## Normative boundary

Autodesk [POLYLINE (DXF)](https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-ABF6B778-BE20-4B49-9B58-A94E64CEFFF3.htm)
defines groups `73/74` as smooth-surface M/N density with zero defaults and
defines group `75` as a non-bit-coded type: `0` none, `5` quadratic B-spline,
`6` cubic B-spline, or `8` Bezier. The reference does not publish a density
range, so M9.2t preserves the existing exact signed-i16 semantics.

## Implementation contract

`crates/seacad-dxf-core/src/polyline_polygon_mesh_smoothing.rs` layers typed
smoothing metadata over every M9.2o record. Valid metadata retains exact signed
densities and one documented surface type. Invalid fields, unknown type codes,
and unavailable topology retain separate typed states without changing cells or
coordinates.

## Test evidence

`crates/seacad-dxf-core/tests/polyline_polygon_mesh_smoothing_tests.rs` covers
ASCII/Binary parity across all nine dialects, signed density retention,
documented type classification, topology failure, invalid/defaulted metadata,
unknown type codes, lookup bounds, cancellation, source identity, and public
traits.

## Non-claims

M9.2t does not impose an undocumented density range, generate fitted vertices
or cells, alter M9.2o topology or M9.2s WCS corners, validate continuity,
tessellate, edit, write, or render.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, formatting,
generated-schema drift, workspace Clippy with warnings denied, 426 workspace
tests with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/polyline_polygon_mesh_smoothing.rs` | 232 | `961f06f72d3f3b0f39ec40b7435684db0283d93bf38e1182f092b0628bc9d9d8` |
| `crates/seacad-dxf-core/src/lib.rs` | 424 | `3bfee94ff86b84eb44dd5b6b1801fe7e8b34ca23b2eb0eebfd72be5e838e85c0` |
| `crates/seacad-dxf-core/tests/polyline_polygon_mesh_smoothing_tests.rs` | 193 | `1864c6b8f73a1ac966446e08418f17346dc455cd8835f0ef4fa33a48987fa6a5` |
| `docs/IMPLEMENTATION_PLAN.md` | 710 | `c7c1802bd99169bf5ccb90d908dbbba14ca4353e3cac2a3320130a272e84718a` |
| `docs/SUPPORT_MATRIX.md` | 602 | `7ebeab2cc858f946e030c2109366643e727ad43e0675086f7db2adaad11bbdac` |
