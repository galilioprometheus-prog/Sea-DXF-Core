# M14.4n HATCH Polyline Coordinates

M14.4n promotes each Polyline vertex X/Y pair to required source-anchored
semantics. Both usable values assemble an exact two-component OCS position;
missing Y, duplicates, malformed ASCII, and non-finite Binary inputs remain
typed, and tuple failure returns an exact X/Y unavailable mask. Signed zero and
the complete M14.4m bulge/header chain remain intact. No transform or segment
geometry is inferred.

Autodesk documents group 10/20 as Polyline vertex coordinates in OCS:
`https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-DC5215D6-E73F-4DFF-8BE9-01CA9610FAEE.htm`

The focused suite passes 4/4 tests and workspace passes 1,145/1,145 tests. All
gates pass. No dependency/license/schema/release/corpus/CI change. Production
adds 300 lines: 293 module and seven exports. OCS/WCS transformation,
closed-path topology, segment geometry, applicability, CRUD/write, and
`Complete` remain open. This audit omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 223 | `e31f0ccae82d7221cfbba1071f861b4026bf5ae51c6e9b929f670791a8578337` |
| `README.vi.md` | 221 | `44a5ea7be0924ff02db055430a324fb0dad6e6944cbe0cf120f1f634fc5ffbb0` |
| `crates/seacad-dxf-core/src/hatch_polyline_vertex_coordinate.rs` | 293 | `964a9c9d2c063ad544185fb0a0eb0e21859940e5eef4f2d376aa38e360610b03` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,351 | `f884d21741ada864315ec9349851fe3835112fa54207e9c4ba9086dc7eb9c8c7` |
| `crates/seacad-dxf-core/tests/hatch_polyline_vertex_coordinate_tests.rs` | 293 | `e2aa71909a5c9222143a84ae2655ee6ec45ad94e1b87eb61d6edfaaff4eb8e15` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `d0b57ea59afb6e3d2869d1be69fc2722f3d6380c94d8491ca60c077b4aa2ab35` |
| `docs/SUPPORT_MATRIX.md` | 3,149 | `8848bca1ec8d5a3888cc964fd3437c12281642e279a6a319827c5d2631e2076d` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,062 | `39413cc81ba1de3db09191d6a73be46b038cba46bd576a62edbce6c297f5f415` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,480 | `6a5fde3fdd9f6602477313a31a815dab91e2c1d089f9fe059140cb55e115de39` |
