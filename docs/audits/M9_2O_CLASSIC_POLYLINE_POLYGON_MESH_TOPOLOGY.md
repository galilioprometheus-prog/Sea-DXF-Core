# M9.2o Classic POLYLINE Polygon-Mesh Topology

Retrieved: 2026-07-30

## Normative boundary

Autodesk [POLYLINE (DXF)](https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-ABF6B778-BE20-4B49-9B58-A94E64CEFFF3.htm)
defines polygon-mesh family bit `16`, M/N counts `71/72`, M closure bit `1`,
and N closure bit `32`. Autodesk [3DMESH](https://help.autodesk.com/view/ACD/2024/ENU/?guid=GUID-BA35CABA-6FDF-419C-AE83-9E28690A4B15)
states that M times N is the vertex count and that all vertices in row M are
supplied before row M+1.

## Implementation contract

`crates/seacad-dxf-core/src/polyline_polygon_mesh.rs` builds row-major
quadrilateral cells only for complete classic polygon-mesh sequences with
matched vertex families, positive M/N counts, and exact count agreement. Each
cell retains named grid-corner VERTEX evidence and independent M/N wrap state.
All incomplete, unsupported, indeterminate, inconsistent, unusable-count, and
count-mismatch cases retain typed zero-cell record state.

## Test evidence

`crates/seacad-dxf-core/tests/polyline_polygon_mesh_tests.rs` covers
ASCII/Binary parity across all nine dialects, a closed 2x3 grid, exact
row-major corners, M/N wrap cells, record-local ranges, every typed failure,
lookup bounds, cancellation, source identity, and public traits.

## Non-claims

M9.2o does not assign face winding or normals, project vertex coordinates,
interpret smoothing metadata, diagnose conformance, edit, write, or render.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, formatting,
generated-schema drift, workspace Clippy with warnings denied, 410 workspace
tests with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/polyline_polygon_mesh.rs` | 447 | `0955bbd3306e160d38ada8cf27f9017b75196d110c6c769d5b1d5c5f7c35202d` |
| `crates/seacad-dxf-core/src/lib.rs` | 396 | `438b492b98d92adf250fd3b111648ec86f15ee5fbfd49cf76ac74ebfe4defa54` |
| `crates/seacad-dxf-core/tests/polyline_polygon_mesh_tests.rs` | 245 | `79a65f5666b0045ea5ecc9af82a07205e9872bd6095d1809b244f0fedf11645c` |
| `docs/IMPLEMENTATION_PLAN.md` | 661 | `368d8a188e729320861e87e1afc196a4fd6f9f30402fd73e311baf2cbfe99bf2` |
| `docs/SUPPORT_MATRIX.md` | 558 | `5c800a050543f7c90491ce434533b0694780351bea2209af488941a17e39e3eb` |
