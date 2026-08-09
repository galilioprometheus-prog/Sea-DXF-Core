# M14.4l HATCH Polyline Vertex Numerics

M14.4l selects every M14.4k X/Y/bulge component into a source-anchored
four-state numeric semantic. Unique values decode as exact finite binary64;
malformed ASCII and non-finite Binary values remain invalid with raw
provenance. Absent Y/bulge stays absent, while duplicates remain invalid
without choosing a member. No bulge default or geometry is inferred.

Autodesk documents group 10/20 vertex coordinates and optional group 42 bulge:
`https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-DC5215D6-E73F-4DFF-8BE9-01CA9610FAEE.htm`

The focused suite passes 4/4 tests and workspace passes 1,137/1,137 tests. All
gates pass. No dependency/license/schema/release/corpus/CI change. Production
adds 311 lines: 305 module and six exports. Required-Y and has-bulge relations,
bulge defaults, OCS/WCS geometry, applicability, CRUD/write, and `Complete`
remain open. This audit omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 219 | `fbc6df77e2350d1a77983a8f25c0f63606af4003793da311aec86235918dc0c7` |
| `README.vi.md` | 217 | `9551fb6049f7977124047b12b4362165891507de7df2c417c4cbcda791adc129` |
| `crates/seacad-dxf-core/src/hatch_polyline_vertex_numeric.rs` | 305 | `f53e39bfc880bb5151dab3fce6ab6955eae257ee4db426ac95ceb1ebc9abed4a` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,339 | `0c9561c41be410bfd157adf3bd5018195a0da2f2b9196b3ce71b738ee9397c46` |
| `crates/seacad-dxf-core/tests/hatch_polyline_vertex_numeric_tests.rs` | 272 | `980097c2f7080a249398a9dca52b49e3e3bc94596b8abe52b41e7e5e36407043` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `1804e4eee6da686bf54ceb14ba81645d1e5e4b5d4c459d61fb9730cef0257dd0` |
| `docs/SUPPORT_MATRIX.md` | 3,130 | `c1c2552d0a0ea0904af63352ac807293ba480dda7b711a913c359adf1a6c39cb` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,043 | `96b10b0928851db2f4116aa81dce0a0fe98411cdb791dd035f297120f9ef62d1` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,461 | `27c6005679f4285c7dabdaecb53b92d0ec3e485f1077b08e4ba3fa7bc3c455ea` |
