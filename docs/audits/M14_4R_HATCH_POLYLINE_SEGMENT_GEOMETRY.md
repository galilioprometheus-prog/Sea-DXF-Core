# M14.4r HATCH Polyline OCS Segment Geometry

M14.4r publishes finite OCS geometry for every M14.4q segment result. Straight
segments retain exact source endpoints. Arc segments with usable endpoints
derive center, positive radius, and signed included-angle sweep while retaining
exact endpoints and authoritative bulge orientation. A nonzero-bulge zero chord
and non-finite intermediate or derived arithmetic fail with distinct typed
issues; prior shape and endpoint failures remain typed.

Autodesk documents group 42 as the Polyline vertex bulge:
`https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-DC5215D6-E73F-4DFF-8BE9-01CA9610FAEE.htm`

The focused suite passes 4/4 tests and workspace passes 1,161/1,161 tests. All
gates pass. No dependency, license, schema, release, corpus, or CI surface
changes. Production adds 338 lines: a 332-line module and six module/export
lines. Derived binary64 values are not raw evidence or guaranteed
cross-platform canonical bits. OCS-to-WCS transformation, HATCH elevation and
extrusion application, applicability, CRUD/write, and `Complete` remain open.
This audit omits its own hash.

One cumulative Antigravity review packet is prepared as `Status: READY` through
M14.4r at
`D:\SeaCad\AntigravityReports\seacad-m14.3dr-m14.4r-hatch-polyline-segment-geometry-2026-08-09.yaml`.
It supersedes the M14.4q packet; no independent Antigravity PASS is claimed at
this checkpoint.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 231 | `81d7e96ef5430da23c56829bf519551fe1592b97fa32aba09eabae6600fd4660` |
| `README.vi.md` | 230 | `59807c767fc6e9fab876d510119e6c25f45934ef35a383c4692b13b180ed8aba` |
| `crates/seacad-dxf-core/src/hatch_polyline_segment_geometry.rs` | 332 | `2b515c20348694898c1d28d8932d4b4875434be32508dabc8c3d7a4c5927fd43` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,374 | `9b6fcf6e2b112fd35debd29c6d6589d198aa876f43b8fe2c527599d5495bf95a` |
| `crates/seacad-dxf-core/tests/hatch_polyline_segment_geometry_tests.rs` | 474 | `7a5f17ca1fec95174171bf8a5d2125227aac40cbb051a836d51bec2360822985` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `2d4dcdb11dc74f855c342ad93c685c3a65fc02c2cdfe750b2956c634aaf91587` |
| `docs/SUPPORT_MATRIX.md` | 3,190 | `1a907875eb1d5c8fc6ab5c5c899b97d36913fd45cda399386c9d2ec161c3774a` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,101 | `6651890da084e8bae1cf4d4ae2819a5c76da78ab0c7d2d55db20043a0cf82fe3` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,519 | `4fad0c3aa626e71f876b0660a2a8827f387ae30f05fe2b3666d4358d9bf427a2` |
