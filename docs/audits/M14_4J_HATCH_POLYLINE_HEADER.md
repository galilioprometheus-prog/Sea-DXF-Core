# M14.4j HATCH Polyline Boundary Header

M14.4j selects required group 72/73/93 header semantics only for valid M14.4i
Polyline paths. Autodesk documents group 72 as has-bulge, group 73 as closed,
and group 93 as vertex count:

`https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-DC5215D6-E73F-4DFF-8BE9-01CA9610FAEE.htm`

Both flags require 0/1 and count is nonnegative. Every selected value retains
its raw group. Edges are NotPolyline; flag failure, absence, duplicates,
malformed ASCII, and domain errors remain typed. This checkpoint does not group
vertices, reconcile counts, decode bulges, derive geometry, add CRUD/write, or
claim `Complete` support.

The focused suite passes 4/4 tests and the workspace passes 1,129/1,129 tests.
All required gates pass. No dependency, license, schema, release, corpus, or CI
change. Production adds 354 lines: 348 in the module and six exports. This audit
omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 214 | `54a9104ed62e9a1c9c4b839ef3fe902dfe8a2279de69aae864baba1a04a650b1` |
| `README.vi.md` | 212 | `fb86ca01503bf907eb110f612561bb3ab8b9ece79b0e855c45428cd28be1dd48` |
| `crates/seacad-dxf-core/src/hatch_polyline_header.rs` | 348 | `db67ae6332c2d5f19d7e1dd36e3205c322ee96411926b65b7baf060e002102fa` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,326 | `fd0e5c80cfbc1d15c70f60bc7fc66dc47ef482c7f1c5cb29e7cb55443509ea57` |
| `crates/seacad-dxf-core/tests/hatch_polyline_header_tests.rs` | 160 | `be0dd29445c827e7a5aa30ab2260e69e536e24c1f81d6897c18df89c749ea01a` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `766034e42d4b704c75c9447850e0116e735a1fbea4b846b44c54cd944f4a8617` |
| `docs/SUPPORT_MATRIX.md` | 3,112 | `6902a14d9b8d0d533b0a9786396c50f966c982f8f76c052e08551db4c30a9eb4` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,027 | `99befd042caad5cffe380d3e0399b5e04e439e2c121c2a8b5993de6ddfee8599` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,447 | `fdc8d1c7742e9ba0ccb0c9bd7bbc18f58e275259530f47c52ec8b9d684541b0a` |
