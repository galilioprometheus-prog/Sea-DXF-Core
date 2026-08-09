# M14.4m HATCH Polyline Bulges

M14.4m resolves each M14.4l bulge against its unique group-72 has-bulge flag.
Absent group 42 receives the documented exact `+0.0` default without invented
raw provenance. Enabled headers preserve explicit and invalid numeric states;
disabled headers reject any present group 42 while retaining the complete
underlying numeric directory. No geometry is inferred.

Autodesk documents group 72 has-bulge and optional group 42 default zero:
`https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-DC5215D6-E73F-4DFF-8BE9-01CA9610FAEE.htm`

The focused suite passes 4/4 tests and workspace passes 1,141/1,141 tests. All
gates pass. No dependency/license/schema/release/corpus/CI change. Production
adds 239 lines: 234 module and five exports. Required-Y semantics, closed-path
topology, OCS/WCS geometry, applicability, CRUD/write, and `Complete` remain
open. This audit omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 221 | `23c2ab8f48622c63e946b8820cf6183464eacbff6d396f6e3f9096896406f03e` |
| `README.vi.md` | 219 | `3fdc4bf6e6539f373e9eae9db28a581dace5c19927ae4ec27a416bac71488882` |
| `crates/seacad-dxf-core/src/hatch_polyline_bulge.rs` | 234 | `e2e2b4a06328caa65e0cb2f5062b33c2ea76ab8b082d3e3580b3767dd8faad57` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,344 | `1dcb6ad6d7db480b31ec842fafbe785e0b09a9164c71f4c36fdca9971b2badc4` |
| `crates/seacad-dxf-core/tests/hatch_polyline_bulge_tests.rs` | 293 | `cefe72c94239252ec6d6fe5b3c704ac1c24f56f6870027d8ceb70eef8709a9a1` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `50d4b865cbd460de565ffbb72052c669497fadce0009adf617c6fd7a8ac648e1` |
| `docs/SUPPORT_MATRIX.md` | 3,139 | `9387302f7e8fe36d3385b78012f5b988482d36440746a95840ba01239f4eb150` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,052 | `d452fec9301b60ebb68a56686eba8dc5a0ba3aa1e634b8141b9b1cbfa85f93a2` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,470 | `28ef3b5afd6b1fe9455f4b43d47c29ded31378d7888fb486511d8d8bf188e117` |
