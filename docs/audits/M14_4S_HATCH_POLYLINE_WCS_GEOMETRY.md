# M14.4s HATCH Polyline WCS Segment Geometry

M14.4s joins every M14.4r segment to its exact HATCH subclass elevation and
explicit/defaulted extrusion, normalizes the nonzero extrusion, and applies the
documented arbitrary-axis basis with its exact `1/64` branch. Straight and Arc
endpoints plus Arc centers become finite WCS triples with normalized WCS
normals. Orthonormal projection preserves radius, signed sweep, and exact
bulge. Failure precedence is source geometry, elevation, extrusion, then
derived transform.

Autodesk documents the basis construction in the
[Arbitrary Axis Algorithm](https://help.autodesk.com/cloudhelp/2015/ENU/AutoCAD-DXF/files/GUID-E19E5B42-0CC7-4EBA-B29F-5E1D595149EE.htm).

The focused suite passes 4/4 tests and workspace passes 1,165/1,165 tests. All
gates pass. No dependency, license, schema, release, corpus, or CI surface
changes. Production adds 451 lines: a 445-line module and six module/export
lines. Derived binary64 WCS values are not raw evidence or guaranteed
cross-platform canonical bits. Applicability, boundary Edges geometry,
CRUD/write, rendering, and `Complete` remain open. This audit omits its own
hash.

One cumulative Antigravity review packet is prepared as `Status: READY` through
M14.4s at
`D:\SeaCad\AntigravityReports\seacad-m14.3dr-m14.4s-hatch-polyline-wcs-geometry-2026-08-09.yaml`.
It supersedes the M14.4r packet; no independent Antigravity PASS is claimed at
this checkpoint.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 233 | `f671f343c05b6835c6d93d5beefde666950d7c0c1c264ec64a1954762d8df42f` |
| `README.vi.md` | 232 | `405ae89c6ebad13845d9981c8ea994fe23a831f916a770fab695dabe49615b51` |
| `crates/seacad-dxf-core/src/hatch_polyline_wcs_geometry.rs` | 445 | `29eaf96f4cfb2ce59ecb8a4f4d2891706862997538476ec52602a40c7d6bb957` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,380 | `1e63c450d3a034ec9ecdb62151d862e8d44a23d94013e222819d45dbafe288c6` |
| `crates/seacad-dxf-core/tests/hatch_polyline_wcs_geometry_tests.rs` | 514 | `5aba8bc64afb133db67e7c57aebc0ab58ad25a645b30a2fa3860ccf58dc50701` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `8bf613cf92aa30340ea56dc6a2265e149b20b03be8e140d0fbf277f4230ca63a` |
| `docs/SUPPORT_MATRIX.md` | 3,202 | `0e9efb6bedcb24c1a002acee658f44d17b0c57bf70fe725d99a93e3f015fd8b0` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,112 | `0dbdf553ce1ff534f42803e54cf7106e96cfe28843dbecf7533d046782a99d25` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,530 | `d44a6e85376464762df4be5a209f2e7f84d625b8db8c4c36ad89a71645b6bf38` |
