# M14.4x HATCH Boundary Line Edge Coordinates

M14.4x promotes all four M14.4w Line-edge numeric components to required
source-anchored coordinate semantics. Four usable finite values assemble exact
OCS start and end points with bit-exact signed zero. An absent component becomes
`MissingRequiredValue` without raw provenance; duplicate, malformed ASCII, and
non-finite Binary components retain their original typed numeric issue and any
available raw provenance. Endpoint failure reports an exact StartX, StartY,
EndX, and EndY unavailable mask while retaining the complete numeric/card/type/
grouping chain. Declared/observed edge-count mismatch does not erase otherwise
usable coordinates. Non-Line, invalid, and Polyline paths publish no Line
coordinate entries.

Autodesk documents Line-edge groups 10/20 as the OCS start point and groups
11/21 as the OCS endpoint in the
[Boundary Path Data table](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-DC5215D6-E73F-4DFF-8BE9-01CA9610FAEE.htm).

The focused suite passes 4/4 tests and workspace passes 1,185/1,185 tests. All
gates pass. No dependency, license, schema, release, corpus, or CI surface
changes. Production adds 375 lines: a 367-line module and eight module/export
lines. OCS/WCS Line geometry, circular/elliptic/spline edge payloads, HATCH
applicability, CRUD/write, rendering, and `Complete` remain open.

The repository batch note and one cumulative external `Status: READY` packet
are synchronized through M14.4x at
`D:\SeaCad\AntigravityReports\seacad-m14.3dr-m14.4x-hatch-boundary-line-edge-coordinates-2026-08-09.yaml`.
No independent Antigravity PASS is claimed. This audit omits its own hash and
the self-referential active batch-note hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 247 | `30c45aa9c07e88dfbc1eef10384541acb2a5360fa20e63269728c5086f36a123` |
| `README.vi.md` | 246 | `2713c71c3e8407581d6b7cb94895e432748dcb207feb2c87717f107d40c680f1` |
| `crates/seacad-dxf-core/src/hatch_boundary_line_edge_coordinate.rs` | 367 | `e50647301efe607b6dc24ec7f0d4d4270f3b3c979822c87cf64623229851d724` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,412 | `e5cab2bc0da15cc5eb2883973b20784e9bb16cb3611f29eb5bd78f7fcad29d3d` |
| `crates/seacad-dxf-core/tests/hatch_boundary_line_edge_coordinate_tests.rs` | 360 | `a2ddadd431735f9257f22238282a507342a9ed8528480297d22839fc60bf2362` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `bb3a36e4a2256d6715433682d8fb165ce2ec733aa0e42377365788f842b6c91e` |
| `docs/SUPPORT_MATRIX.md` | 3,261 | `52bbb195b0d01708ea56bae90cabee74cf5784379020d81e2dabaf20e40ecead` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,167 | `39bd27b6191e23cd2e3d679480d1e1d038c623f9d54359eebb7cc1ed0260a91b` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,586 | `9b8f07b5cf429d8f58337dacabb2caa8a13ab01cf46b3b3143dd82cede60fa13` |
