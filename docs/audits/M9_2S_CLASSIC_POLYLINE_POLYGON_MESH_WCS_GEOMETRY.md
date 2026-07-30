# M9.2s Classic POLYLINE Polygon-Mesh WCS Geometry

Retrieved: 2026-07-30

## Normative boundary

Autodesk [VERTEX (DXF)](https://help.autodesk.com/cloudhelp/2020/ENU/AutoCAD-DXF/files/GUID-0741E831-599E-4CBF-91E1-8ADBCFD6556D.htm)
defines groups `10/20/30` as WCS location components for 3D vertices and flag
bit `64` as a 3D polygon-mesh vertex. M9.2o already freezes Autodesk's
row-major M/N topology and exact named cell-corner evidence.

## Implementation contract

`crates/seacad-dxf-core/src/polyline_polygon_mesh_geometry.rs` assembles every
M9.2o cell's named `(m0,n0)`, `(m0,n1)`, `(m1,n1)`, and `(m1,n0)` coordinate
VERTEX components into exact WCS tuples. Topology, wrap state, and source
identity remain attached. An unusable component retains a typed cell state
naming the first failed corner and all three component semantic states.

## Test evidence

`crates/seacad-dxf-core/tests/polyline_polygon_mesh_geometry_tests.rs` covers
ASCII/Binary parity across all nine dialects, exact WCS bits and row-major
corner order, record-local lookup, corner/component failure, cancellation,
source identity, and public traits.

## Non-claims

M9.2s does not assign winding, derive edges, validate degeneracy or planarity,
calculate normals, interpret smoothing metadata, triangulate, edit, write, or
render.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, formatting,
generated-schema drift, workspace Clippy with warnings denied, 423 workspace
tests with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/polyline_polygon_mesh_geometry.rs` | 253 | `c97657583457e8decebfecf4393225c3f2ccd8e9635912a88fc0cba14fd57503` |
| `crates/seacad-dxf-core/src/lib.rs` | 419 | `f0f83acaab5e9f4a579553c9cd7e0de56068f277b078994db78632fd9ca370de` |
| `crates/seacad-dxf-core/tests/polyline_polygon_mesh_geometry_tests.rs` | 235 | `5e297900ef79114bddb3b484821e8ae614e49a65def423dca794d40abab52281` |
| `docs/IMPLEMENTATION_PLAN.md` | 703 | `7645fb9a4c64d4ca563257e290d61bba5d987bf61c1894f03af88d9ea3d3f370` |
| `docs/SUPPORT_MATRIX.md` | 594 | `3edeabe1011b9b0c23348706be39e65e419f43b359f44587dd4866b534636039` |
