# M14.4z HATCH Boundary Line Edge WCS Geometry

M14.4z joins every M14.4y OCS Line segment to its exact HATCH subclass
elevation and explicit/defaulted extrusion, then projects both endpoints through
the reviewed M14.4s arbitrary-axis basis. Each usable result exposes finite WCS
start/end triples and the normalized WCS normal. The shared basis remains
crate-private, so this checkpoint removes duplicate projection policy without
expanding the public API.

Failure precedence is source geometry, elevation, extrusion, then non-finite
derived geometry. Every result retains the complete lower-layer geometry,
elevation, and extrusion directories, plus per-entry elevation/extrusion
receipts. Declared/observed edge-count mismatch does not erase otherwise usable
geometry. Non-Line, invalid, and Polyline paths publish no Line entries.

Autodesk documents HATCH elevation groups 10/20/30, extrusion groups
210/220/230, and Line-edge OCS endpoints 10/20 and 11/21 in the
[HATCH entity data](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-C6C71CED-CE0F-4184-82A5-07AD6241F15B.htm)
and
[Boundary Path Data table](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-DC5215D6-E73F-4DFF-8BE9-01CA9610FAEE.htm).

The focused suite passes 4/4 tests, the existing M14.4s WCS suite passes 4/4,
and workspace passes 1,193/1,193 tests. All gates pass. No dependency, license,
schema, release, corpus, or CI surface changes. Production adds 316 net lines:
a 309-line WCS module, five module/export lines, and two net crate-private basis
visibility lines. Circular/elliptic/spline edge payloads and geometry, HATCH
applicability, CRUD/write, rendering, and `Complete` remain open.

The repository batch note and one cumulative external `Status: READY` packet
are synchronized through M14.4z at
`D:\SeaCad\AntigravityReports\seacad-m14.3dr-m14.4z-hatch-boundary-line-edge-wcs-geometry-2026-08-09.yaml`.
No independent Antigravity PASS is claimed. This audit omits its own hash and
the self-referential active batch-note hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 253 | `0adba91ca9959e5101414385367ff6e1692e4e3f04c8da81fa172234231dd86c` |
| `README.vi.md` | 252 | `488f64fed5422266815fa8a5648a2b7d443b2c05eb0ca94aac027242f72c8752` |
| `crates/seacad-dxf-core/src/hatch_boundary_line_edge_wcs_geometry.rs` | 309 | `b2201edc6c546c7266cc161644b521b148bb237aa7a78b5bd44dc9a1817d577b` |
| `crates/seacad-dxf-core/src/hatch_polyline_wcs_geometry.rs` | 447 | `1cd8fcfb9836ab973aae8839a527c324c842a574e1ca490bfc9d1a6375b2ffbe` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,422 | `b00b6edf68c3a1a91d6d7494c2d6e32a65eed9ef1a52ab19567e742d2c5ab90b` |
| `crates/seacad-dxf-core/tests/hatch_boundary_line_edge_wcs_geometry_tests.rs` | 438 | `3b404ba79a2f83aff6f43cffea41f727fc492124fe8ec8b6bf2dc7e556cecba9` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `22fb21e32ae2110616e2832cc743c388fdbcbbcfbdc414f32c20e2ed806847e0` |
| `docs/SUPPORT_MATRIX.md` | 3,283 | `05acd7e3f7d6a98284462e4c39c8e3d7db7722c14d3b9021431fb305aa924b3e` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,188 | `c0ebee489a65aa3834a2ad7951e189e620235f78b974372d7e3fae7330fa1f04` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,608 | `8863f0d5e72f1ac193ee1879eb93f12483e272635b6103a8582075239573be84` |
