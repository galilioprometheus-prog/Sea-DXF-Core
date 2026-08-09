# M14.4k HATCH Polyline Vertex Grouping

M14.4k groups each M14.4j Polyline payload by exact group-10 vertex anchors.
Following group 20 and optional group 42 occurrences remain source-ordered
members of that vertex; recognized pre-anchor fields remain explicit orphans.
X is unique by construction, Y/bulge cardinality is fixed, and observed vertex
count compares with group 93. Edges and invalid headers publish no vertices.

Autodesk documents group 10/20 vertex coordinates and optional group 42 bulge:
`https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-DC5215D6-E73F-4DFF-8BE9-01CA9610FAEE.htm`

The focused suite passes 4/4 tests and workspace passes 1,133/1,133 tests. All
gates pass. No dependency/license/schema/release/corpus/CI change. Production
adds 412 lines: 405 module and seven exports. Numeric selection, defaults,
geometry, applicability, CRUD/write, and `Complete` remain open. This audit
omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 216 | `ec0c7ef12dd58c669c526f85b66a007457ffcd9fdeb1513a6ade44ee60763112` |
| `README.vi.md` | 214 | `b1443a61c118f276e7efd52a1aec94e8b648cd4fdcafc4eae41c123c1ce8da9e` |
| `crates/seacad-dxf-core/src/hatch_polyline_vertex.rs` | 405 | `1b2f32f1508b3429dd8fb6fc7c44d3f1b296268a61c9e661a8a8220767ac39a2` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,333 | `91bc559b2a37785ab062546fb19934281b188a26ef6b762be93b69f983ab8062` |
| `crates/seacad-dxf-core/tests/hatch_polyline_vertex_tests.rs` | 213 | `c7ec214df32076c80a2a3335cc4ecc841413e1b8e3d6ecd44fd6065fb32f1b85` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `7a91dcee2bf23ba59f54904cd99fbd0e2f0e2a6bc4d73a487377ea3fc042ee0e` |
| `docs/SUPPORT_MATRIX.md` | 3,121 | `bbb1506541656131581c044fcc9c04bec97145cd13e1bf8e09f1a7f22b217ae4` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,034 | `d5f2f44fde1645ad13281cf82ac2155da4dbcec89a019dd950b18d7158727819` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,453 | `b08a34bf5c63723f04ec4d6d89fa24b566961a8984766f3e422c6977fc4057d8` |
