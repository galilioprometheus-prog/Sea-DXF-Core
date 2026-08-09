# Antigravity Mechanical Verification Batch

Status: READY

Checkpoint: `M14.3dr-M14.4ae-hatch-boundary-circular-arc-edge-wcs-geometry`
Repository root: `D:\SeaCad\SeaCad`
Baseline: `028d726b7f04709927e0d6c8c8d419dc1eb49fcb`
Review target: annotated tag `m14.4ae-hatch-boundary-circular-arc-edge-wcs-geometry`
Report: `D:\SeaCad\AntigravityReports\seacad-m14.3dr-m14.4ae-hatch-boundary-circular-arc-edge-wcs-geometry-2026-08-10.yaml`
Prepared: `2026-08-10` (`Asia/Saigon`)

The preceding M14.3dl-M14.3dq cumulative batch was independently reviewed PASS
at exact baseline HEAD. Its external report SHA-256 is
`b27b982faee5ad86a47ca87346b4ac383a210097596ab80c9ed9c41ceec8e665`.
Do not repeat that retired batch. This READY batch intentionally accumulates
M14.3dr through M14.4ae so implementation does not pause between checkpoints.

## Authority, procedure, and writes

Mechanically verify M14.3dr-M14.4ae only. Codex/user retain architecture,
support, license, commit/tag/merge/release decisions. Use the exact root, read
`AGENTS.md` and this note completely, confirm exactly one READY, and run phases
in order. The repository is read-only; ignored target/cache activity is
tolerated. Write only the external report. No edits, Git mutation, installs,
network/vendor CAD, legacy/private corpus, license/export/archive, or fixture
creation. A root, baseline, tag, path, or receipt mismatch is BLOCKED.

## Exact preflight state

Require a clean tracked and untracked worktree. Record `Get-Location`, Git
status, HEAD, all target annotated-tag objects/messages/peeled commits, the
baseline tag target, and the baseline-to-target changed path set. Require:

```text
HEAD == m14.4ae-hatch-boundary-circular-arc-edge-wcs-geometry^{}
m14.4ae-hatch-boundary-circular-arc-edge-wcs-geometry^1 == m14.4ad-hatch-boundary-circular-arc-edge-ocs-geometry^{}
m14.4ad-hatch-boundary-circular-arc-edge-ocs-geometry^1 == m14.4ac-hatch-boundary-circular-arc-edge-semantics^{}
m14.4ac-hatch-boundary-circular-arc-edge-semantics^1 == m14.4ab-hatch-boundary-circular-arc-edge-numerics^{}
m14.4ab-hatch-boundary-circular-arc-edge-numerics^1 == m14.4aa-hatch-boundary-circular-arc-edge-cards^{}
m14.4aa-hatch-boundary-circular-arc-edge-cards^1 == dxf-struct-r1b-hatch-geometry-resolvers^{}
dxf-struct-r1b-hatch-geometry-resolvers^1 == dxf-struct-r1a-hatch-read-support^{}
dxf-struct-r1a-hatch-read-support^1 == m14.4z-hatch-boundary-line-edge-wcs-geometry^{}
m14.4z-hatch-boundary-line-edge-wcs-geometry^1 == m14.4y-hatch-boundary-line-edge-ocs-geometry^{}
m14.4y-hatch-boundary-line-edge-ocs-geometry^1 == m14.4x-hatch-boundary-line-edge-coordinates^{}
m14.4x-hatch-boundary-line-edge-coordinates^1 == m14.4w-hatch-boundary-line-edge-numerics^{}
m14.4w-hatch-boundary-line-edge-numerics^1 == m14.4v-hatch-boundary-line-edge-cards^{}
m14.4v-hatch-boundary-line-edge-cards^1 == m14.4u-hatch-boundary-edge-types^{}
m14.4u-hatch-boundary-edge-types^1 == m14.4t-hatch-boundary-edge-grouping^{}
m14.4t-hatch-boundary-edge-grouping^1 == m14.4s-hatch-polyline-wcs-geometry^{}
m14.4s-hatch-polyline-wcs-geometry^1 == m14.4r-hatch-polyline-segment-geometry^{}
m14.4r-hatch-polyline-segment-geometry^1 == m14.4q-hatch-polyline-line-geometry^{}
m14.4q-hatch-polyline-line-geometry^1 == m14.4p-hatch-polyline-segment-shapes^{}
m14.4p-hatch-polyline-segment-shapes^1 == m14.4o-hatch-polyline-segments^{}
m14.4o-hatch-polyline-segments^1 == m14.4n-hatch-polyline-coordinates^{}
m14.4n-hatch-polyline-coordinates^1 == m14.4m-hatch-polyline-bulges^{}
m14.4m-hatch-polyline-bulges^1 == m14.4l-hatch-polyline-vertex-numerics^{}
m14.4l-hatch-polyline-vertex-numerics^1 == m14.4k-hatch-polyline-vertices^{}
m14.4k-hatch-polyline-vertices^1 == m14.4j-hatch-polyline-header^{}
m14.4j-hatch-polyline-header^1 == m14.4i-hatch-path-flags^{}
m14.4i-hatch-path-flags^1 == m14.4h-hatch-boundary-paths^{}
m14.4h-hatch-boundary-paths^1 == m14.4g-hatch-elevation^{}
m14.4g-hatch-elevation^1 == m14.4f-hatch-boundary-partition^{}
m14.4f-hatch-boundary-partition^1 == m14.4e-hatch-extrusion^{}
m14.4e-hatch-extrusion^1 == m14.4d-hatch-scalar-semantics^{}
m14.4d-hatch-scalar-semantics^1 == m14.4c-hatch-scalar-cardinality^{}
m14.4c-hatch-scalar-cardinality^1 == m14.4b-hatch-scalar-evidence^{}
m14.4b-hatch-scalar-evidence^1 == m14.4a-fill-mesh-evidence^{}
m14.4a-fill-mesh-evidence^1 == m14.3ec-curve-completion-ledger^{}
m14.3ec-curve-completion-ledger^1 == m14.3eb-spline-first-derivative^{}
m14.3eb-spline-first-derivative^1 == m14.3ea-spline-point-evaluation^{}
m14.3ea-spline-point-evaluation^1 == m14.3dz-point-completion-ledger^{}
m14.3dz-point-completion-ledger^1 == m14.3dy-point-color-name-transcode^{}
m14.3dy-point-color-name-transcode^1 == m14.3dx-xdata-text-transcode^{}
m14.3dx-xdata-text-transcode^1 == m14.3dw-text-transcode^{}
m14.3dw-text-transcode^1 == m14.3dv-point-clone-legacy-adaptation^{}
m14.3dv-point-clone-legacy-adaptation^1 == m14.3du-point-clone-xdata-insert-write^{}
m14.3du-point-clone-xdata-insert-write^1 == m14.3dt-point-clone-xdata-draft^{}
m14.3dt-point-clone-xdata-draft^1 == m14.3ds-point-clone-draft-projection^{}
m14.3ds-point-clone-draft-projection^1 == m14.3dr-entity-xdata-draft-write^{}
m14.3dr-entity-xdata-draft-write^1 == 028d726b7f04709927e0d6c8c8d419dc1eb49fcb
m14.3dq-entity-xdata-draft-verification^{} == 028d726b7f04709927e0d6c8c8d419dc1eb49fcb
all forty-five review tags are annotated tag objects
```

The baseline-to-target changed path set must be exactly:

```text
README.md
README.vi.md
crates/seacad-dxf-core/src/encoding.rs
crates/seacad-dxf-core/src/entity_completion.rs
crates/seacad-dxf-core/src/entity_edit_session.rs
crates/seacad-dxf-core/src/entity_xdata_encoded_destination.rs
crates/seacad-dxf-core/src/entity_xdata_draft_write.rs
crates/seacad-dxf-core/src/fill_mesh_evidence.rs
crates/seacad-dxf-core/src/hatch_scalar_evidence.rs
crates/seacad-dxf-core/src/hatch_scalar_card.rs
crates/seacad-dxf-core/src/hatch_scalar_semantic.rs
crates/seacad-dxf-core/src/hatch_extrusion.rs
crates/seacad-dxf-core/src/hatch_polyline_bulge.rs
crates/seacad-dxf-core/src/hatch_boundary_partition.rs
crates/seacad-dxf-core/src/hatch_boundary_circular_arc_edge_card.rs
crates/seacad-dxf-core/src/hatch_boundary_circular_arc_edge_geometry.rs
crates/seacad-dxf-core/src/hatch_boundary_circular_arc_edge_wcs_geometry.rs
crates/seacad-dxf-core/src/hatch_boundary_circular_arc_edge_numeric.rs
crates/seacad-dxf-core/src/hatch_boundary_circular_arc_edge_semantic.rs
crates/seacad-dxf-core/src/hatch_boundary_edge.rs
crates/seacad-dxf-core/src/hatch_boundary_edge_type.rs
crates/seacad-dxf-core/src/hatch_boundary_line_edge_card.rs
crates/seacad-dxf-core/src/hatch_boundary_line_edge_coordinate.rs
crates/seacad-dxf-core/src/hatch_boundary_line_edge_geometry.rs
crates/seacad-dxf-core/src/hatch_boundary_line_edge_wcs_geometry.rs
crates/seacad-dxf-core/src/hatch_boundary_line_edge_numeric.rs
crates/seacad-dxf-core/src/hatch_elevation.rs
crates/seacad-dxf-core/src/hatch_boundary_path.rs
crates/seacad-dxf-core/src/hatch_boundary_path_flags.rs
crates/seacad-dxf-core/src/hatch_polyline_header.rs
crates/seacad-dxf-core/src/hatch_polyline_line_geometry.rs
crates/seacad-dxf-core/src/hatch_polyline_segment.rs
crates/seacad-dxf-core/src/hatch_polyline_segment_geometry.rs
crates/seacad-dxf-core/src/hatch_polyline_segment_shape.rs
crates/seacad-dxf-core/src/hatch_polyline_vertex.rs
crates/seacad-dxf-core/src/hatch_polyline_vertex_coordinate.rs
crates/seacad-dxf-core/src/hatch_polyline_vertex_numeric.rs
crates/seacad-dxf-core/src/hatch_polyline_wcs_geometry.rs
crates/seacad-dxf-core/src/lib.rs
crates/seacad-dxf-core/src/read_support.rs
crates/seacad-dxf-core/src/point_clone_draft_projection.rs
crates/seacad-dxf-core/src/point_clone_xdata_draft.rs
crates/seacad-dxf-core/src/point_clone_xdata_insert.rs
crates/seacad-dxf-core/src/spline_first_derivative.rs
crates/seacad-dxf-core/src/spline_point_evaluation.rs
crates/seacad-dxf-core/src/text_decoder.rs
crates/seacad-dxf-core/src/text_encoder.rs
crates/seacad-dxf-core/src/text_transcode.rs
crates/seacad-dxf-core/tests/entity_draft_record_tests.rs
crates/seacad-dxf-core/tests/entity_completion_tests.rs
crates/seacad-dxf-core/tests/entity_xdata_draft_record_tests.rs
crates/seacad-dxf-core/tests/entity_xdata_encoded_destination_tests.rs
crates/seacad-dxf-core/tests/entity_xdata_text_transcode_tests.rs
crates/seacad-dxf-core/tests/fill_mesh_evidence_tests.rs
crates/seacad-dxf-core/tests/hatch_scalar_evidence_tests.rs
crates/seacad-dxf-core/tests/hatch_scalar_card_tests.rs
crates/seacad-dxf-core/tests/hatch_scalar_semantic_tests.rs
crates/seacad-dxf-core/tests/hatch_extrusion_tests.rs
crates/seacad-dxf-core/tests/hatch_polyline_bulge_tests.rs
crates/seacad-dxf-core/tests/hatch_boundary_partition_tests.rs
crates/seacad-dxf-core/tests/hatch_boundary_circular_arc_edge_card_tests.rs
crates/seacad-dxf-core/tests/hatch_boundary_circular_arc_edge_geometry_tests.rs
crates/seacad-dxf-core/tests/hatch_boundary_circular_arc_edge_wcs_geometry_tests.rs
crates/seacad-dxf-core/tests/hatch_boundary_circular_arc_edge_numeric_tests.rs
crates/seacad-dxf-core/tests/hatch_boundary_circular_arc_edge_semantic_tests.rs
crates/seacad-dxf-core/tests/hatch_boundary_edge_tests.rs
crates/seacad-dxf-core/tests/hatch_boundary_edge_type_tests.rs
crates/seacad-dxf-core/tests/hatch_boundary_line_edge_card_tests.rs
crates/seacad-dxf-core/tests/hatch_boundary_line_edge_coordinate_tests.rs
crates/seacad-dxf-core/tests/hatch_boundary_line_edge_geometry_tests.rs
crates/seacad-dxf-core/tests/hatch_boundary_line_edge_wcs_geometry_tests.rs
crates/seacad-dxf-core/tests/hatch_boundary_line_edge_numeric_tests.rs
crates/seacad-dxf-core/tests/hatch_elevation_tests.rs
crates/seacad-dxf-core/tests/hatch_boundary_path_tests.rs
crates/seacad-dxf-core/tests/hatch_boundary_path_flag_tests.rs
crates/seacad-dxf-core/tests/hatch_polyline_header_tests.rs
crates/seacad-dxf-core/tests/hatch_polyline_line_geometry_tests.rs
crates/seacad-dxf-core/tests/hatch_polyline_segment_tests.rs
crates/seacad-dxf-core/tests/hatch_polyline_segment_geometry_tests.rs
crates/seacad-dxf-core/tests/hatch_polyline_segment_shape_tests.rs
crates/seacad-dxf-core/tests/hatch_polyline_vertex_tests.rs
crates/seacad-dxf-core/tests/hatch_polyline_vertex_coordinate_tests.rs
crates/seacad-dxf-core/tests/hatch_polyline_vertex_numeric_tests.rs
crates/seacad-dxf-core/tests/hatch_polyline_wcs_geometry_tests.rs
crates/seacad-dxf-core/tests/spline_first_derivative_tests.rs
crates/seacad-dxf-core/tests/spline_point_evaluation_tests.rs
crates/seacad-dxf-core/tests/text_transcode_tests.rs
docs/ANTIGRAVITY_MECHANICAL_BATCH.md
docs/ARCHITECTURE.md
docs/IMPLEMENTATION_PLAN.md
docs/SUPPORT_MATRIX.md
docs/audits/M14_3DR_ENTITY_XDATA_DRAFT_WRITE.md
docs/audits/M14_3DS_POINT_CLONE_DRAFT_PROJECTION.md
docs/audits/M14_3DT_POINT_CLONE_XDATA_DRAFT.md
docs/audits/M14_3DU_POINT_CLONE_XDATA_INSERT_WRITE.md
docs/audits/M14_3DV_POINT_CLONE_LEGACY_ADAPTATION.md
docs/audits/M14_3DW_TEXT_TRANSCODE.md
docs/audits/M14_3DX_XDATA_TEXT_TRANSCODE.md
docs/audits/M14_3DY_POINT_COLOR_NAME_TRANSCODE.md
docs/audits/M14_3DZ_POINT_COMPLETION_LEDGER.md
docs/audits/M14_3EA_SPLINE_POINT_EVALUATION.md
docs/audits/M14_3EB_SPLINE_FIRST_DERIVATIVE.md
docs/audits/M14_3EC_CURVE_COMPLETION_LEDGER.md
docs/audits/M14_4A_FILL_MESH_EXACT_EVIDENCE.md
docs/audits/M14_4B_HATCH_SCALAR_EVIDENCE.md
docs/audits/M14_4C_HATCH_SCALAR_CARDINALITY.md
docs/audits/M14_4D_HATCH_SCALAR_SEMANTICS.md
docs/audits/M14_4E_HATCH_EXTRUSION.md
docs/audits/M14_4F_HATCH_BOUNDARY_PARTITION.md
docs/audits/M14_4G_HATCH_ELEVATION.md
docs/audits/M14_4H_HATCH_BOUNDARY_PATHS.md
docs/audits/M14_4I_HATCH_PATH_FLAGS.md
docs/audits/M14_4J_HATCH_POLYLINE_HEADER.md
docs/audits/M14_4K_HATCH_POLYLINE_VERTICES.md
docs/audits/M14_4L_HATCH_POLYLINE_VERTEX_NUMERICS.md
docs/audits/M14_4M_HATCH_POLYLINE_BULGES.md
docs/audits/M14_4N_HATCH_POLYLINE_COORDINATES.md
docs/audits/M14_4O_HATCH_POLYLINE_SEGMENTS.md
docs/audits/M14_4P_HATCH_POLYLINE_SEGMENT_SHAPES.md
docs/audits/M14_4Q_HATCH_POLYLINE_LINE_GEOMETRY.md
docs/audits/M14_4R_HATCH_POLYLINE_SEGMENT_GEOMETRY.md
docs/audits/M14_4S_HATCH_POLYLINE_WCS_GEOMETRY.md
docs/audits/M14_4T_HATCH_BOUNDARY_EDGE_GROUPING.md
docs/audits/M14_4U_HATCH_BOUNDARY_EDGE_TYPES.md
docs/audits/M14_4V_HATCH_BOUNDARY_LINE_EDGE_CARDS.md
docs/audits/M14_4W_HATCH_BOUNDARY_LINE_EDGE_NUMERICS.md
docs/audits/M14_4X_HATCH_BOUNDARY_LINE_EDGE_COORDINATES.md
docs/audits/M14_4Y_HATCH_BOUNDARY_LINE_EDGE_OCS_GEOMETRY.md
docs/audits/M14_4Z_HATCH_BOUNDARY_LINE_EDGE_WCS_GEOMETRY.md
docs/audits/M14_4AA_HATCH_BOUNDARY_CIRCULAR_ARC_EDGE_CARDS.md
docs/audits/M14_4AB_HATCH_BOUNDARY_CIRCULAR_ARC_EDGE_NUMERICS.md
docs/audits/M14_4AC_HATCH_BOUNDARY_CIRCULAR_ARC_EDGE_SEMANTICS.md
docs/audits/M14_4AD_HATCH_BOUNDARY_CIRCULAR_ARC_EDGE_OCS_GEOMETRY.md
docs/audits/M14_4AE_HATCH_BOUNDARY_CIRCULAR_ARC_EDGE_WCS_GEOMETRY.md
docs/audits/DXF_STRUCT_R1A_HATCH_READ_SUPPORT.md
docs/audits/DXF_STRUCT_R1B_HATCH_GEOMETRY_RESOLVERS.md
docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md
docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md
```

Compare paths as sets.

## Artifact receipts

| Path | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 267 | `dab50c56dfb316a6472e47b1bf9ca0c8862de7cdd7000623e8653ddcba905c72` |
| `README.vi.md` | 267 | `407eb082f1035ea39a2cc0238892792e8154273036ca6854101eac6120e9cc69` |
| `crates/seacad-dxf-core/src/encoding.rs` | 788 | `246e8f228588680d408dd7ae71f37e16094ab687c3c0fe55d8a48e4aef75a9a7` |
| `crates/seacad-dxf-core/src/entity_completion.rs` | 129 | `0be29a16b5cc3c1d08075678461acc017531f5ac0eb2a3c0f1436470f81f5aa6` |
| `crates/seacad-dxf-core/src/entity_edit_session.rs` | 2,972 | `ecbc34c75c2c6c03467ea66448f55ddddaaa5550cf087b528239bd214e890207` |
| `crates/seacad-dxf-core/src/entity_xdata_encoded_destination.rs` | 680 | `c4363b3f36799a4cf1b05b0dde28292169a92b5bbca12b0832135a432386d97b` |
| `crates/seacad-dxf-core/src/entity_xdata_draft_write.rs` | 206 | `bbcf63d4f01a6f0d6baa3eae84bd92f71cd21371e36b9bd6b69cc8e9b80691ee` |
| `crates/seacad-dxf-core/src/fill_mesh_evidence.rs` | 399 | `814364cbf0b63b7b060c38a0b08d0975dc7befc18051cc00fbb63ad2545d1fa1` |
| `crates/seacad-dxf-core/src/hatch_scalar_evidence.rs` | 395 | `4e76086db9803d9e56b497df55bddcf942554f77e9a7978227ca72bb59410aec` |
| `crates/seacad-dxf-core/src/hatch_scalar_card.rs` | 326 | `27a0d63020afa8fee257b54ea051c467c3f7dfbc7271c952d1cca3d06d0a133a` |
| `crates/seacad-dxf-core/src/hatch_scalar_semantic.rs` | 380 | `696c5c142f68cb286e1a3c9eb7cf07d1f417bf025987f71f5f17d1f28641151d` |
| `crates/seacad-dxf-core/src/hatch_extrusion.rs` | 332 | `d19c3906f909be8fe9b57c6d62457e1a7ac0984e6525a07770812165be7e6916` |
| `crates/seacad-dxf-core/src/hatch_polyline_bulge.rs` | 234 | `e2e2b4a06328caa65e0cb2f5062b33c2ea76ab8b082d3e3580b3767dd8faad57` |
| `crates/seacad-dxf-core/src/hatch_boundary_partition.rs` | 350 | `4cc56cbefc4d1eb73dfa5baaf9a3f0867774df56c1e8a347e4ec71ce07dc98d8` |
| `crates/seacad-dxf-core/src/hatch_elevation.rs` | 443 | `2543b5459525495f3af6a16b47fe0fcf78d64abdb09084d14de4edf46caaaf7d` |
| `crates/seacad-dxf-core/src/hatch_boundary_path.rs` | 476 | `7d5557d42e3b208badd81ef9678c3c959c7fdc3595388b1696c2e72e47034d87` |
| `crates/seacad-dxf-core/src/hatch_boundary_path_flags.rs` | 309 | `16d27be5eeae57b70814975594526b2b187e084bac6d0d4cadb12a8992647284` |
| `crates/seacad-dxf-core/src/hatch_polyline_header.rs` | 348 | `db67ae6332c2d5f19d7e1dd36e3205c322ee96411926b65b7baf060e002102fa` |
| `crates/seacad-dxf-core/src/hatch_polyline_segment.rs` | 413 | `5d54ec4f661282fb0c0f3fc6c1a73beb2e18ccd4df0397fc7e2acbd4d841cc22` |
| `crates/seacad-dxf-core/src/hatch_polyline_segment_shape.rs` | 200 | `b3b99e6cbc1358ad7c7465060201420f97859b665faef6b9fb7a56fdce41cbcd` |
| `crates/seacad-dxf-core/src/hatch_polyline_vertex.rs` | 405 | `1b2f32f1508b3429dd8fb6fc7c44d3f1b296268a61c9e661a8a8220767ac39a2` |
| `crates/seacad-dxf-core/src/hatch_polyline_vertex_coordinate.rs` | 293 | `964a9c9d2c063ad544185fb0a0eb0e21859940e5eef4f2d376aa38e360610b03` |
| `crates/seacad-dxf-core/src/hatch_polyline_vertex_numeric.rs` | 305 | `f53e39bfc880bb5151dab3fce6ab6955eae257ee4db426ac95ceb1ebc9abed4a` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,459 | `40e9d5bd3b5b041e21b42f5ec49cf45e21106ad6fbbc4e1121ced2acb5875871` |
| `crates/seacad-dxf-core/src/read_support.rs` | 41 | `8bc57927b02ec282b32ede57d0a62cf3e9b7083ffa1ae9fd30fa680930480ed5` |
| `crates/seacad-dxf-core/src/point_clone_draft_projection.rs` | 511 | `9809d74f63e8b5510689e650b95dc45baaea5073ba4d93247271fc41be74a822` |
| `crates/seacad-dxf-core/src/point_clone_xdata_draft.rs` | 250 | `0a34de42328039c76f8ac45226da6e6a7ae027831bec4b3088ba3d8c3295e77e` |
| `crates/seacad-dxf-core/src/point_clone_xdata_insert.rs` | 317 | `59c087ff6cc447604379588eba154d013e67712d9dcc881a0b6708dadf2bae6d` |
| `crates/seacad-dxf-core/src/spline_first_derivative.rs` | 250 | `fe7dbe2cd6b8dc4e5b8505688eb564b7a5c8f8c9d0b4c3f4fcdad36ce65377c2` |
| `crates/seacad-dxf-core/src/spline_point_evaluation.rs` | 416 | `d2e5e9d7bb70fe59b976793e8d5fe48ce1d121fc0984afd3c481cb4675ee81b2` |
| `crates/seacad-dxf-core/src/text_decoder.rs` | 468 | `d38aa65593bd980581de7e6da81ffcd93ce780f7dfc98f49c61e49e691ffd9bb` |
| `crates/seacad-dxf-core/src/text_encoder.rs` | 156 | `914fac5c7237a60ccc418cecfec8104fc759725f31a9fa6874a90665ece89d88` |
| `crates/seacad-dxf-core/src/text_transcode.rs` | 330 | `0bc7bfedcadabd98bc882baf4cb234c5775495622d7f183a3690a51961d24a5d` |
| `crates/seacad-dxf-core/tests/entity_draft_record_tests.rs` | 1,924 | `2d053ec4c981df8ad0b13aade304e4ca2c41d59f7d977ab3a5280cab5ed10537` |
| `crates/seacad-dxf-core/tests/entity_completion_tests.rs` | 152 | `1eb2a2fd6bf3eee59fbffd0fb6d60979cbc49e6445fff5792f98499ecc65cbf1` |
| `crates/seacad-dxf-core/tests/entity_xdata_draft_record_tests.rs` | 1,630 | `84ad2245a954fbf221439abf26d9863c73b33b6937b555e31e161a612867eb08` |
| `crates/seacad-dxf-core/tests/entity_xdata_encoded_destination_tests.rs` | 1,234 | `e6450275eb747cf8fc2d4caf9546738fdd1c711b93f15399644409fdc7f5e619` |
| `crates/seacad-dxf-core/tests/entity_xdata_text_transcode_tests.rs` | 627 | `70e5d77b74566aab2cc70e4ef5b40d3fe3d522f2a880b1b160a0563575180485` |
| `crates/seacad-dxf-core/tests/fill_mesh_evidence_tests.rs` | 358 | `4ba42f81dac85ca940de29bd6e97f89b0cda49d0846e831eaa3217586b400189` |
| `crates/seacad-dxf-core/tests/hatch_scalar_evidence_tests.rs` | 457 | `95c76aabaaffe0e2c7cfd1be1ba0d659853705e621f67571cb5bc59baca62af3` |
| `crates/seacad-dxf-core/tests/hatch_scalar_card_tests.rs` | 424 | `88947fdc8c9827ee3efdfa556912062e3f1833835d7890c6db97b3db3269a439` |
| `crates/seacad-dxf-core/tests/hatch_scalar_semantic_tests.rs` | 497 | `fb2fc1d5b98409c0cedaadacc39ad05c45b324352e91b9a77d04e1ab2b82320b` |
| `crates/seacad-dxf-core/tests/hatch_extrusion_tests.rs` | 351 | `83ae95388810e2a1b50a0e4dcf28b073366a4d2f4fd4c5b247548240535d0cbf` |
| `crates/seacad-dxf-core/tests/hatch_polyline_bulge_tests.rs` | 293 | `cefe72c94239252ec6d6fe5b3c704ac1c24f56f6870027d8ceb70eef8709a9a1` |
| `crates/seacad-dxf-core/tests/hatch_boundary_partition_tests.rs` | 397 | `e6cbc4677ebbb30347e88434c54526629f7552d5030023397d6beef03162ae3b` |
| `crates/seacad-dxf-core/tests/hatch_elevation_tests.rs` | 404 | `cc9216d83d712a4028828f8492ad804b24c7d9e156b962dfac95b5f78430c052` |
| `crates/seacad-dxf-core/tests/hatch_boundary_path_tests.rs` | 411 | `71adb9b4690fc2cd2f0ac81e74158bcfb987410d69dced21e11567b94d148118` |
| `crates/seacad-dxf-core/tests/hatch_boundary_path_flag_tests.rs` | 256 | `408c222b00e3e270007a1286a42a6ab54d8901ac49708eba3050a7253d6566d3` |
| `crates/seacad-dxf-core/tests/hatch_polyline_header_tests.rs` | 160 | `be0dd29445c827e7a5aa30ab2260e69e536e24c1f81d6897c18df89c749ea01a` |
| `crates/seacad-dxf-core/tests/hatch_polyline_segment_tests.rs` | 311 | `e64d78976c0dd1707370711431e4e62c54e479ca9e1488bea4a1189f742aa253` |
| `crates/seacad-dxf-core/tests/hatch_polyline_segment_shape_tests.rs` | 343 | `d1afcf5105e1f59a0cd5199ace5706e9e532a343b7cff3c9bc57bea808f35e96` |
| `crates/seacad-dxf-core/tests/hatch_polyline_vertex_tests.rs` | 213 | `c7ec214df32076c80a2a3335cc4ecc841413e1b8e3d6ecd44fd6065fb32f1b85` |
| `crates/seacad-dxf-core/tests/hatch_polyline_vertex_coordinate_tests.rs` | 293 | `e2aa71909a5c9222143a84ae2655ee6ec45ad94e1b87eb61d6edfaaff4eb8e15` |
| `crates/seacad-dxf-core/tests/hatch_polyline_vertex_numeric_tests.rs` | 272 | `980097c2f7080a249398a9dca52b49e3e3bc94596b8abe52b41e7e5e36407043` |
| `crates/seacad-dxf-core/tests/spline_first_derivative_tests.rs` | 367 | `4b6306081db5d1ca6cba239f0fecc09dbcf3506e87e3d04920a9c0a09a9c3dbb` |
| `crates/seacad-dxf-core/tests/spline_point_evaluation_tests.rs` | 348 | `bc72c88c95a4fc983ff0e64a409d539f8d8d6b460d8e9c5d37e6cb0972e2fdc3` |
| `crates/seacad-dxf-core/tests/text_transcode_tests.rs` | 476 | `9bda1ee19362b058ab9a174705cc0d5893f73d13bdb8e008b6e4a7d2d2340a40` |
| `docs/audits/M14_3DR_ENTITY_XDATA_DRAFT_WRITE.md` | 55 | `0475c6d1c6f73507f1c63e9da586c7078e39c25c91f0fd1b2a13848af264d540` |
| `docs/audits/M14_3DS_POINT_CLONE_DRAFT_PROJECTION.md` | 50 | `0ab9ac7a5274f38ad9cb8309a53c3b9f04f1d79512c7159f8434c7b6d4f911cc` |
| `docs/audits/M14_3DT_POINT_CLONE_XDATA_DRAFT.md` | 46 | `c9ff7377bfe2c674a0eddcb029e32f34a4c6e4b488adf76bef638e0788e04905` |
| `docs/audits/M14_3DU_POINT_CLONE_XDATA_INSERT_WRITE.md` | 44 | `0275c2672e26db5d5e5bd0031c615dea733df10005e1a12755cfc53638e31c08` |
| `docs/audits/M14_3DV_POINT_CLONE_LEGACY_ADAPTATION.md` | 41 | `d1489d15a457ee5c518c8c8d4a6e83a9b6e12a091d889c4b543d0400cc858be4` |
| `docs/audits/M14_3DW_TEXT_TRANSCODE.md` | 42 | `dcfa1d5099f1d1fe620936e930008db7d6d38d3b6f8627e4be06014542820726` |
| `docs/audits/M14_3DX_XDATA_TEXT_TRANSCODE.md` | 48 | `0c357517d7337856a41ea408e61c98645e610bade3352b67c3fc0e4b29e56f65` |
| `docs/audits/M14_3DY_POINT_COLOR_NAME_TRANSCODE.md` | 47 | `83c9cf0bc099af593c14f6899c8a9211990f3c98db2c364ee373b42d77e9db42` |
| `docs/audits/M14_3DZ_POINT_COMPLETION_LEDGER.md` | 54 | `a5e56f3813ceb079aa699e1537a8d72eb5364a3bdff5f5abaa443947c9f71308` |
| `docs/audits/M14_3EA_SPLINE_POINT_EVALUATION.md` | 66 | `a5aea63b5fe3565533f74046e72af1d0ad12c4845702ab2823777a936e580768` |
| `docs/audits/M14_3EB_SPLINE_FIRST_DERIVATIVE.md` | 76 | `72bd840ad7da890989d45de3aa47e19e66baa6a5ab5db736bf97773c7f34ce28` |
| `docs/audits/M14_3EC_CURVE_COMPLETION_LEDGER.md` | 60 | `9ee25e9da9433116db22df1c8ac0f9a37681d0b18162b033fd07c80b264c0706` |
| `docs/audits/M14_4A_FILL_MESH_EXACT_EVIDENCE.md` | 87 | `db4bc08acc877e919d9a432146ce803bbba2bdd231359831247f8c3b5e3dd185` |
| `docs/audits/M14_4B_HATCH_SCALAR_EVIDENCE.md` | 84 | `35ed6a7cdaf14439a4a4239015dabe4667b802c95874232e1a9ac9288b27567a` |
| `docs/audits/M14_4C_HATCH_SCALAR_CARDINALITY.md` | 71 | `fc8954c82c54efbdbd58360ae5bd3a18dcfc06bf601dc38bfe66238868d987a7` |
| `docs/audits/M14_4D_HATCH_SCALAR_SEMANTICS.md` | 72 | `4f8c59031ec6353764ee00eae34d067125da0eb0f2dc6ef35efb1a9c9185c2b3` |
| `docs/audits/M14_4E_HATCH_EXTRUSION.md` | 64 | `8d9eb014e0121ed19cc48bb95a6d53149a6e1a5b6cd70cef7c226b58ff1f6be5` |
| `docs/audits/M14_4F_HATCH_BOUNDARY_PARTITION.md` | 65 | `11b5ed8ba61f51647c31ae78916438d932d581806821a17ba52d1e9729dc5d8f` |
| `docs/audits/M14_4G_HATCH_ELEVATION.md` | 64 | `a188c6962a2a79500117015556b0cad12b446f2640f421b368d95d5b1a0ba35f` |
| `docs/audits/M14_4H_HATCH_BOUNDARY_PATHS.md` | 64 | `8f57df94fc17c5327a047a6e031c29c290c4be9cf01e2a219cf39d85a0900284` |
| `docs/audits/M14_4I_HATCH_PATH_FLAGS.md` | 52 | `e32ca0680547a9488c4387540bb94f89b76c3e98dca59872f1043aae72283037` |
| `docs/audits/M14_4J_HATCH_POLYLINE_HEADER.md` | 30 | `4fc65258d9e22afbdf998e01c6ba7db9499156264c93ff16af7dc74af97b5702` |
| `docs/audits/M14_4K_HATCH_POLYLINE_VERTICES.md` | 28 | `5c9ac5617698dfceaecff1d5fcb0c71f607e586e70a4d80d1da0b1b0c48ac7ce` |
| `docs/audits/M14_4L_HATCH_POLYLINE_VERTEX_NUMERICS.md` | 28 | `ccdc67d5db63bdf5299e80fec312f6e24ab82ab29e374ecddaf6782e85c83b98` |
| `docs/audits/M14_4M_HATCH_POLYLINE_BULGES.md` | 28 | `bc47d75bf7fcf61f0e76f5e1b3e8ec5819a996b4823826c9eb8ef4128d10c11d` |
| `docs/audits/M14_4N_HATCH_POLYLINE_COORDINATES.md` | 29 | `2d911f6cebb2e41b7e5370773dacadb8a089e9c074cf379e2be3bf66b1c31056` |
| `docs/audits/M14_4O_HATCH_POLYLINE_SEGMENTS.md` | 28 | `e31ab1ca17a486fe43fac5b3d61f90fb19a1d4a4005e0d81ff6ec0d825fdbc32` |
| `docs/audits/M14_4P_HATCH_POLYLINE_SEGMENT_SHAPES.md` | 28 | `2ba2ca33d3eaab5008aeef78026b73de40b7b76ada8451b97ddc5d2506e51b3f` |
| `crates/seacad-dxf-core/src/hatch_polyline_line_geometry.rs` | 231 | `ce3448a1a1c766de669862eaf85282fd5340540523354ebf1e7f6d0b45d61e24` |
| `crates/seacad-dxf-core/tests/hatch_polyline_line_geometry_tests.rs` | 394 | `50a1a78ace72330aaacf82d1bad1cfb4f2cea1f20cb63117e938b7a23a2b2359` |
| `docs/audits/M14_4Q_HATCH_POLYLINE_LINE_GEOMETRY.md` | 36 | `39e2578b4183ddbc0be3b9fce9a5040e00bfc02c1fcb51bfa6ee68c9ea0bd7c7` |
| `crates/seacad-dxf-core/src/hatch_polyline_segment_geometry.rs` | 343 | `07fe2d014d3bcbab1cd2eb94a1e8b4467552b701809b58ef6183f63078dce76a` |
| `crates/seacad-dxf-core/tests/hatch_polyline_segment_geometry_tests.rs` | 474 | `7a5f17ca1fec95174171bf8a5d2125227aac40cbb051a836d51bec2360822985` |
| `docs/audits/M14_4R_HATCH_POLYLINE_SEGMENT_GEOMETRY.md` | 37 | `c955110561944b10e5520d5a6cc07078a4712b9dcf4288e1c094fdbb09ca8129` |
| `crates/seacad-dxf-core/src/hatch_polyline_wcs_geometry.rs` | 450 | `d0f0f4ca229cde1dcfc0fd159fe728805461069865e7df00a31fd32a827ab4a2` |
| `crates/seacad-dxf-core/tests/hatch_polyline_wcs_geometry_tests.rs` | 514 | `5aba8bc64afb133db67e7c57aebc0ab58ad25a645b30a2fa3860ccf58dc50701` |
| `docs/audits/M14_4S_HATCH_POLYLINE_WCS_GEOMETRY.md` | 38 | `209289cb1cb408dfb2b15de4100905fd5575078e6b72a42098481beca7c5ae79` |
| `crates/seacad-dxf-core/src/hatch_boundary_edge.rs` | 468 | `b1f3c61d79bb4dbebc70c1e65e2ab03ebd3290ce9eee0bb643d6672342f5543b` |
| `crates/seacad-dxf-core/tests/hatch_boundary_edge_tests.rs` | 382 | `37bb0dcdc6d22822abf8bc6f6f1daa02546781f8dc824b002ddaa98e034920d6` |
| `docs/audits/M14_4T_HATCH_BOUNDARY_EDGE_GROUPING.md` | 38 | `b81f9ebb7a4d88e55c34858501e0a16ee92525c5e72a5b50a857b5a3b3ef1794` |
| `crates/seacad-dxf-core/src/hatch_boundary_edge_type.rs` | 174 | `6e4819d65757e92fd37aa9fc35f1dfa3186f4f31d796b9daccd1d7f0f31aa927` |
| `crates/seacad-dxf-core/tests/hatch_boundary_edge_type_tests.rs` | 281 | `840a3bda352e35af6977aaaa1a12e48178a255070ce2095102b1b7ce68358dd9` |
| `docs/audits/M14_4U_HATCH_BOUNDARY_EDGE_TYPES.md` | 40 | `3d64de7ac6638718ef397c4e6a76246e841f43cce43e8ccf52b8471d07b56a7e` |
| `crates/seacad-dxf-core/src/hatch_boundary_line_edge_card.rs` | 292 | `b012a334c8fce0727afdcf5e37b3b6b577600c4e0dfe21adc285c6666cfd3ab6` |
| `crates/seacad-dxf-core/tests/hatch_boundary_line_edge_card_tests.rs` | 301 | `bbcb46b022fc568b0adf4d63c5d5dd18c3f627e9ac51ad718da70ca5b3edf59c` |
| `docs/audits/M14_4V_HATCH_BOUNDARY_LINE_EDGE_CARDS.md` | 39 | `2aefb8d93fb7867f02a873dddeceb93cf690c2de2a260d1354d4767243815d56` |
| `crates/seacad-dxf-core/src/hatch_boundary_line_edge_numeric.rs` | 307 | `4d6f849a4477d109c76d34f27b75a7190570543ab997edad827d1d892fd2e1ec` |
| `crates/seacad-dxf-core/tests/hatch_boundary_line_edge_numeric_tests.rs` | 316 | `3388531ea007f147c620805e09256feec8ab68a4ad6ee5c67126a4805c61c215` |
| `docs/audits/M14_4W_HATCH_BOUNDARY_LINE_EDGE_NUMERICS.md` | 41 | `603f3b6467680d6a091bb29e4f69b3c0a1862ffcb62ec15be505c3b3427d40ba` |
| `crates/seacad-dxf-core/src/hatch_boundary_line_edge_coordinate.rs` | 336 | `3ad0595a8345cffd3a1bef3857e1966fc2ae193a3c6960508a80aa38063ccdd0` |
| `crates/seacad-dxf-core/tests/hatch_boundary_line_edge_coordinate_tests.rs` | 360 | `a2ddadd431735f9257f22238282a507342a9ed8528480297d22839fc60bf2362` |
| `docs/audits/M14_4X_HATCH_BOUNDARY_LINE_EDGE_COORDINATES.md` | 40 | `f0b197ef981abde4e418c75a9dd0a4f857c129620477527e0326c10a57f00c4b` |
| `crates/seacad-dxf-core/src/hatch_boundary_line_edge_geometry.rs` | 202 | `e8f4a0a495d02a44a0097a101d459fa277c4ae18ab8c4b363b593b3057a65793` |
| `crates/seacad-dxf-core/tests/hatch_boundary_line_edge_geometry_tests.rs` | 335 | `4ab1f4f7bf6e053a516035fbc7f2d9d233a06419561162ca09b2754c2a0be6eb` |
| `docs/audits/M14_4Y_HATCH_BOUNDARY_LINE_EDGE_OCS_GEOMETRY.md` | 39 | `b103dcc4692aa4f786e3dde8b6e0d894348bae7e179275dc8ffe51a58eb21227` |
| `crates/seacad-dxf-core/src/hatch_boundary_line_edge_wcs_geometry.rs` | 267 | `305bc6eaa1d7bc2e478d1003bedb3fc85ffffd5d8dfd7769b7394707db489400` |
| `crates/seacad-dxf-core/tests/hatch_boundary_line_edge_wcs_geometry_tests.rs` | 438 | `3b404ba79a2f83aff6f43cffea41f727fc492124fe8ec8b6bf2dc7e556cecba9` |
| `docs/audits/M14_4Z_HATCH_BOUNDARY_LINE_EDGE_WCS_GEOMETRY.md` | 46 | `779ca62f1286cc397bb60dc9e5cdf9abaaf5d3268f432ac20292304e0024e4d1` |
| `docs/ARCHITECTURE.md` | 204 | `4fdcda1179cb34a1dc9b3f188a553899d0ccd114e558dbce2ad330e3db5ec570` |
| `docs/audits/DXF_STRUCT_R1A_HATCH_READ_SUPPORT.md` | 41 | `284db1a0087232e23cd12e2ce587994055ea6ef6191fbe6a44f79f07feca155d` |
| `docs/audits/DXF_STRUCT_R1B_HATCH_GEOMETRY_RESOLVERS.md` | 35 | `6b131455084f7cd407e643e883c43f02b2bb66a04af94232b783701d0a8e110a` |
| `crates/seacad-dxf-core/src/hatch_boundary_circular_arc_edge_card.rs` | 304 | `2a6468d37d0001b85567a30b3ddf5a78bff9c822146fd36e255c710369fadb07` |
| `crates/seacad-dxf-core/tests/hatch_boundary_circular_arc_edge_card_tests.rs` | 320 | `7efb5656911a85fd46343792d147fea6deab7a56f75cc7d45ff87db936234bda` |
| `docs/audits/M14_4AA_HATCH_BOUNDARY_CIRCULAR_ARC_EDGE_CARDS.md` | 42 | `c3bea76c1b86b3c184a4c190e112914922459e4dbdda3d7c41e255c0b845d6ca` |
| `crates/seacad-dxf-core/src/hatch_boundary_circular_arc_edge_numeric.rs` | 390 | `c6a4a220d203816b04b538ba571423a4e2b338f9ddae00854bd17e4a1d51bbfa` |
| `crates/seacad-dxf-core/tests/hatch_boundary_circular_arc_edge_numeric_tests.rs` | 377 | `c93905c601cf84e30b7393b6358fc37647afd451cfcbe2624a48bbf3ee7b972c` |
| `docs/audits/M14_4AB_HATCH_BOUNDARY_CIRCULAR_ARC_EDGE_NUMERICS.md` | 44 | `0c9f989eaae33143cd21606db23272cccfbf1ebe70a5309a2a40c8e533927193` |
| `crates/seacad-dxf-core/src/hatch_boundary_circular_arc_edge_semantic.rs` | 341 | `470ad786f740459d2a211cde5b4f24ec769c034ff4775c7599f95117acf2964f` |
| `crates/seacad-dxf-core/tests/hatch_boundary_circular_arc_edge_semantic_tests.rs` | 522 | `7d8d9513972a599ab68c4e5ebf005cf9b32f91b3436da6246419320d3493c0a8` |
| `docs/audits/M14_4AC_HATCH_BOUNDARY_CIRCULAR_ARC_EDGE_SEMANTICS.md` | 50 | `26b284485b2092292927a3e37706a5844b26246bbdf247fa108ef9d009fa5cfa` |
| `crates/seacad-dxf-core/src/hatch_boundary_circular_arc_edge_geometry.rs` | 322 | `916a0518c88c2c09e1fa824d253b701ebd77b52a34ac79772ec67206ca957b13` |
| `crates/seacad-dxf-core/tests/hatch_boundary_circular_arc_edge_geometry_tests.rs` | 390 | `eeeafc4f3db6397400ae604d52f0beb389e83db0d77673096ddf1ec95ea66970` |
| `docs/audits/M14_4AD_HATCH_BOUNDARY_CIRCULAR_ARC_EDGE_OCS_GEOMETRY.md` | 47 | `06566b9d45d6e11e620ef3c8dbbc7ac2efec12bde7974874f912211596137d50` |
| `crates/seacad-dxf-core/src/hatch_boundary_circular_arc_edge_wcs_geometry.rs` | 315 | `9b9dcacdf5987719f05a500d73918c7dc72ba56dc2144ab6ac938135289572be` |
| `crates/seacad-dxf-core/tests/hatch_boundary_circular_arc_edge_wcs_geometry_tests.rs` | 483 | `9bdfd04070ebd2002cc80476e100077bb6d67e78b9d0d1692205db64a3c044cf` |
| `docs/audits/M14_4AE_HATCH_BOUNDARY_CIRCULAR_ARC_EDGE_WCS_GEOMETRY.md` | 55 | `863b4157f8079f74b2f6cee83901625d416956062b1d93c5e47441a38550b26e` |
| `docs/IMPLEMENTATION_PLAN.md` | 424 | `480759a6c908073aac76b31fbed8ea8fc0b749f6e48e3eee79bac85926c8b684` |
| `docs/SUPPORT_MATRIX.md` | 3,346 | `46b9be5d74f4fbef0e53cedbb98e3bf7411b4573abef86891d16427ab3616199` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,264 | `d7d636e09afc5736d21a28598a1d20390d64137a980a72d50db3819dbb0b2e12` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,689 | `6959a05ed3bf14e0c7e2464a363afa1fd052df7f24507dcd9baa0be8b4b06614` |

Compute every line count and lowercase SHA-256 and require exact matches. The
active batch note intentionally omits its self-referential receipt.

## Focused and contract checks

Run separately:

```powershell
cargo +1.97.1 test -p seacad-dxf-core --test entity_completion_tests
cargo +1.97.1 test -p seacad-dxf-core --test fill_mesh_evidence_tests
cargo +1.97.1 test -p seacad-dxf-core --test hatch_scalar_evidence_tests
cargo +1.97.1 test -p seacad-dxf-core --test hatch_scalar_card_tests
cargo +1.97.1 test -p seacad-dxf-core --test hatch_scalar_semantic_tests
cargo +1.97.1 test -p seacad-dxf-core --test hatch_extrusion_tests
cargo +1.97.1 test -p seacad-dxf-core --test hatch_polyline_bulge_tests
cargo +1.97.1 test -p seacad-dxf-core --test hatch_boundary_partition_tests
cargo +1.97.1 test -p seacad-dxf-core --test hatch_elevation_tests
cargo +1.97.1 test -p seacad-dxf-core --test hatch_boundary_path_tests
cargo +1.97.1 test -p seacad-dxf-core --test hatch_boundary_path_flag_tests
cargo +1.97.1 test -p seacad-dxf-core --test hatch_polyline_header_tests
cargo +1.97.1 test -p seacad-dxf-core --test hatch_polyline_segment_tests
cargo +1.97.1 test -p seacad-dxf-core --test hatch_polyline_segment_shape_tests
cargo +1.97.1 test -p seacad-dxf-core --test hatch_polyline_line_geometry_tests
cargo +1.97.1 test -p seacad-dxf-core --test hatch_polyline_segment_geometry_tests
cargo +1.97.1 test -p seacad-dxf-core --test hatch_polyline_wcs_geometry_tests
cargo +1.97.1 test -p seacad-dxf-core --test hatch_boundary_edge_tests
cargo +1.97.1 test -p seacad-dxf-core --test hatch_boundary_edge_type_tests
cargo +1.97.1 test -p seacad-dxf-core --test hatch_boundary_circular_arc_edge_card_tests
cargo +1.97.1 test -p seacad-dxf-core --test hatch_boundary_circular_arc_edge_numeric_tests
cargo +1.97.1 test -p seacad-dxf-core --test hatch_boundary_circular_arc_edge_semantic_tests
cargo +1.97.1 test -p seacad-dxf-core --test hatch_boundary_circular_arc_edge_geometry_tests
cargo +1.97.1 test -p seacad-dxf-core --test hatch_boundary_circular_arc_edge_wcs_geometry_tests
cargo +1.97.1 test -p seacad-dxf-core --test hatch_boundary_line_edge_card_tests
cargo +1.97.1 test -p seacad-dxf-core --test hatch_boundary_line_edge_numeric_tests
cargo +1.97.1 test -p seacad-dxf-core --test hatch_boundary_line_edge_coordinate_tests
cargo +1.97.1 test -p seacad-dxf-core --test hatch_boundary_line_edge_geometry_tests
cargo +1.97.1 test -p seacad-dxf-core --test hatch_boundary_line_edge_wcs_geometry_tests
cargo +1.97.1 test -p seacad-dxf-core --test hatch_polyline_vertex_tests
cargo +1.97.1 test -p seacad-dxf-core --test hatch_polyline_vertex_coordinate_tests
cargo +1.97.1 test -p seacad-dxf-core --test hatch_polyline_vertex_numeric_tests
cargo +1.97.1 test -p seacad-dxf-core --test spline_point_evaluation_tests
cargo +1.97.1 test -p seacad-dxf-core --test spline_first_derivative_tests
cargo +1.97.1 test -p seacad-dxf-core --test entity_xdata_draft_record_tests
cargo +1.97.1 test -p seacad-dxf-core --test entity_draft_record_tests
cargo +1.97.1 test -p seacad-dxf-core --test entity_insert_session_tests
cargo +1.97.1 test -p seacad-dxf-core --test text_transcode_tests
cargo +1.97.1 test -p seacad-dxf-core --lib text_encoder::tests
cargo +1.97.1 test -p seacad-dxf-core --test entity_xdata_text_transcode_tests
cargo +1.97.1 test -p seacad-dxf-core --test entity_xdata_encoded_destination_tests
rg -n 'DxfEntityXDataDraftWrite|write_reparse_verify_and_journal_to_new_file|existing|cancelled|tampered' crates/seacad-dxf-core/src/entity_xdata_draft_write.rs crates/seacad-dxf-core/tests/entity_xdata_draft_record_tests.rs crates/seacad-dxf-core/src/lib.rs
rg -n 'DxfPointCloneDestinationBindings|DxfPointCloneDraftProjectionIssue|DxfPointCloneDraftProjectionPlan|project_point_clone_draft_from|prepare_point_clone_snapshot|borrowed_for_destination' crates/seacad-dxf-core/src/point_clone_draft_projection.rs crates/seacad-dxf-core/src/entity_edit_session.rs crates/seacad-dxf-core/tests/entity_draft_record_tests.rs crates/seacad-dxf-core/src/lib.rs
rg -n 'DxfPointCloneXDataSource|DxfPointCloneXDataDraftIssue|DxfPointCloneXDataDraftPlan|project_point_clone_xdata_draft_from|with_xdata_composition|EncodedPayloadUnavailable|standalone' crates/seacad-dxf-core/src/point_clone_xdata_draft.rs crates/seacad-dxf-core/src/point_clone_draft_projection.rs crates/seacad-dxf-core/tests/entity_xdata_draft_record_tests.rs crates/seacad-dxf-core/src/lib.rs
rg -n 'DxfPointCloneSourceEvidence|DxfPointCloneXDataInsertPlan|DxfPointCloneXDataVerificationJournal|DxfPointCloneXDataWriteJournal|plan_point_clone_xdata_insert|write_reparse_verify_and_journal_to_new_file|existing|tampered' crates/seacad-dxf-core/src/point_clone_xdata_insert.rs crates/seacad-dxf-core/tests/entity_xdata_draft_record_tests.rs crates/seacad-dxf-core/src/lib.rs
rg -n 'DxfPointCloneDialectAdaptations|legacy_placement_owns_layout|omitted_by_layer_lineweight|DestinationFieldNotRepresentable|LINEWEIGHT|Ac1032|Ac1009' crates/seacad-dxf-core/src/point_clone_draft_projection.rs crates/seacad-dxf-core/src/point_clone_xdata_draft.rs crates/seacad-dxf-core/src/point_clone_xdata_insert.rs crates/seacad-dxf-core/tests/entity_draft_record_tests.rs crates/seacad-dxf-core/tests/entity_xdata_draft_record_tests.rs crates/seacad-dxf-core/src/lib.rs
rg -n 'DxfTextEncoder|DxfTextEncodeStatus|DxfTextTranscodeIssue|DxfTextTranscodePlan|transcode_text_span_to|round_trip|Unmappable|Unavailable|ValueBytes' crates/seacad-dxf-core/src/text_encoder.rs crates/seacad-dxf-core/src/text_transcode.rs crates/seacad-dxf-core/src/encoding.rs crates/seacad-dxf-core/tests/text_transcode_tests.rs crates/seacad-dxf-core/src/lib.rs
rg -n 'DxfTextTranscodeReceipt|text_transcode_for_entry|TextTranscode|TranscodedTextTooLong|DXF_XDATA_STRING_MAX_BYTES|point_clone_writes_transcoded_xdata' crates/seacad-dxf-core/src/text_transcode.rs crates/seacad-dxf-core/src/entity_xdata_encoded_destination.rs crates/seacad-dxf-core/tests/entity_xdata_text_transcode_tests.rs crates/seacad-dxf-core/tests/entity_xdata_encoded_destination_tests.rs crates/seacad-dxf-core/tests/entity_xdata_draft_record_tests.rs crates/seacad-dxf-core/src/lib.rs
rg -n 'PointCloneText|color_name_transcode|TextTranscode|COLOR_NAME|transcode_color_name|point_clone_color_name|point_clone_carries_color_name' crates/seacad-dxf-core/src/entity_edit_session.rs crates/seacad-dxf-core/src/point_clone_draft_projection.rs crates/seacad-dxf-core/src/point_clone_xdata_draft.rs crates/seacad-dxf-core/src/point_clone_xdata_insert.rs crates/seacad-dxf-core/src/text_transcode.rs crates/seacad-dxf-core/tests/entity_draft_record_tests.rs crates/seacad-dxf-core/tests/entity_xdata_draft_record_tests.rs crates/seacad-dxf-core/tests/text_transcode_tests.rs
rg -n 'DxfEntityCompletion(Level|Blocker|Assessment)|DXF_ENTITY_COMPLETION_ASSESSMENTS|dxf_entity_completion_assessment|TypedSemantics|Geometry|VerifiedMutation|ReleaseQualified|PublicGeometryQualification|PrivateCorpusQualification|CurrentCheckpointSixNativeCi' crates/seacad-dxf-core/src/entity_completion.rs crates/seacad-dxf-core/tests/entity_completion_tests.rs crates/seacad-dxf-core/src/lib.rs docs/audits/M14_3DZ_POINT_COMPLETION_LEDGER.md docs/audits/M14_3EC_CURVE_COMPLETION_LEDGER.md
rg -n 'DxfFillMesh(EvidenceDirectory|Family|Field|Range|RecordEntry|SubclassEntry)|fill_mesh_evidence_directory|AcDbHatch|AcDbSubDMesh' crates/seacad-dxf-core/src/fill_mesh_evidence.rs crates/seacad-dxf-core/tests/fill_mesh_evidence_tests.rs crates/seacad-dxf-core/src/lib.rs docs/audits/M14_4A_FILL_MESH_EXACT_EVIDENCE.md
rg -n 'DXF_HATCH_SCALAR_ROLES|DxfHatchScalar(Directory|Entry|Issue|Occurrence|Role|Value)|hatch_scalar_directory|PatternLineCount|Gradient' crates/seacad-dxf-core/src/hatch_scalar_evidence.rs crates/seacad-dxf-core/tests/hatch_scalar_evidence_tests.rs crates/seacad-dxf-core/src/lib.rs docs/audits/M14_4B_HATCH_SCALAR_EVIDENCE.md
rg -n 'DxfHatchScalarCard(State|MemberRange|Member|Directory)|hatch_scalar_card_directory|cards_for_subclass|cards_for_raw_record|card_for_role|occurrence_for_member' crates/seacad-dxf-core/src/hatch_scalar_card.rs crates/seacad-dxf-core/tests/hatch_scalar_card_tests.rs crates/seacad-dxf-core/src/lib.rs docs/audits/M14_4C_HATCH_SCALAR_CARDINALITY.md
rg -n 'DxfHatchScalarSemantic(Issue|Value|Entry|Directory)|hatch_scalar_semantic_directory|MultipleValues|NonFiniteDouble|ValueOutOfDomain|entity\.hatch|default_for|value_in_domain' crates/seacad-dxf-core/src/hatch_scalar_semantic.rs crates/seacad-dxf-core/tests/hatch_scalar_semantic_tests.rs crates/seacad-dxf-core/src/lib.rs docs/audits/M14_4D_HATCH_SCALAR_SEMANTICS.md
rg -n 'DxfHatchExtrusion(Component|Directory|Entry|InputKind|Issue|UnavailableComponents)|hatch_extrusion_directory|ComponentsUnavailable|ZeroVector|entry_for_subclass' crates/seacad-dxf-core/src/hatch_extrusion.rs crates/seacad-dxf-core/tests/hatch_extrusion_tests.rs crates/seacad-dxf-core/src/lib.rs docs/audits/M14_4E_HATCH_EXTRUSION.md
rg -n 'DxfHatchBoundaryPartition(Directory|Entry|Issue)|hatch_boundary_partition_directory|BoundaryPathCountAbsent|HatchStyleAbsent|AnchorOrderInvalid|header_fields_for_subclass|boundary_fields_for_subclass|trailing_fields_for_subclass' crates/seacad-dxf-core/src/hatch_boundary_partition.rs crates/seacad-dxf-core/tests/hatch_boundary_partition_tests.rs crates/seacad-dxf-core/src/lib.rs docs/audits/M14_4F_HATCH_BOUNDARY_PARTITION.md
rg -n 'DxfHatchElevation(Component|Components|Directory|Entry|EntryState|Issue|UnavailableComponents)|hatch_elevation_directory|PartitionUnavailable|PlanarComponentNonZero|NonFiniteDouble|entries_for_raw_record' crates/seacad-dxf-core/src/hatch_elevation.rs crates/seacad-dxf-core/tests/hatch_elevation_tests.rs crates/seacad-dxf-core/src/lib.rs docs/audits/M14_4G_HATCH_ELEVATION.md
rg -n 'DxfHatchBoundaryPath(CountIssue|CountRelation|Directory|Entry|Range|Topology|TopologyEntry|TopologyIssue)|hatch_boundary_path_directory|orphan_fields_for_subclass|payload_fields_for_path|Matched|Mismatched|DeclaredUnavailable' crates/seacad-dxf-core/src/hatch_boundary_path.rs crates/seacad-dxf-core/tests/hatch_boundary_path_tests.rs crates/seacad-dxf-core/src/lib.rs docs/audits/M14_4H_HATCH_BOUNDARY_PATHS.md
rg -n 'DxfHatchBoundaryPath(FlagDirectory|FlagEntry|FlagIssue|FlagState|FlagValue|Flags|Kind)|hatch_boundary_path_flag_directory|UnsupportedBits|is_polyline|is_external|is_outermost' crates/seacad-dxf-core/src/hatch_boundary_path_flags.rs crates/seacad-dxf-core/tests/hatch_boundary_path_flag_tests.rs crates/seacad-dxf-core/src/lib.rs docs/audits/M14_4I_HATCH_PATH_FLAGS.md
rg -n 'DxfHatchPolyline(Boolean|Header|HeaderDirectory|HeaderEntry|HeaderIssue|HeaderState|VertexCount)|hatch_polyline_header_directory|NotPolyline|ValueOutOfDomain' crates/seacad-dxf-core/src/hatch_polyline_header.rs crates/seacad-dxf-core/tests/hatch_polyline_header_tests.rs crates/seacad-dxf-core/src/lib.rs docs/audits/M14_4J_HATCH_POLYLINE_HEADER.md
rg -n 'DxfHatchPolylineVertex(CardState|CountRelation|Directory|Entry|Grouping|GroupingState|Member|PathEntry|Role)|hatch_polyline_vertex_directory|vertex_for_ordinal|orphan_for_ordinal|Matched|Mismatched' crates/seacad-dxf-core/src/hatch_polyline_vertex.rs crates/seacad-dxf-core/tests/hatch_polyline_vertex_tests.rs crates/seacad-dxf-core/src/lib.rs docs/audits/M14_4K_HATCH_POLYLINE_VERTICES.md
rg -n 'DxfHatchPolylineVertexNumeric(Components|Directory|Entry|Issue|Value)|hatch_polyline_vertex_numeric_directory|MultipleValues|InvalidAsciiNumber|NonFiniteDouble|polyline_vertex_(x|y|bulge)' crates/seacad-dxf-core/src/hatch_polyline_vertex_numeric.rs crates/seacad-dxf-core/tests/hatch_polyline_vertex_numeric_tests.rs crates/seacad-dxf-core/src/lib.rs docs/audits/M14_4L_HATCH_POLYLINE_VERTEX_NUMERICS.md
rg -n 'DxfHatchPolylineBulge(Directory|Entry|Issue|Value)|hatch_polyline_bulge_directory|PresentWhenHeaderDisallows|DEFAULT_BULGE|Defaulted' crates/seacad-dxf-core/src/hatch_polyline_bulge.rs crates/seacad-dxf-core/tests/hatch_polyline_bulge_tests.rs crates/seacad-dxf-core/src/lib.rs docs/audits/M14_4M_HATCH_POLYLINE_BULGES.md
rg -n 'DxfHatchPolylineVertex(CoordinateDirectory|CoordinateEntry|CoordinateIssue|CoordinateValue|Coordinates|OcsPosition|PositionIssue|UnavailableCoordinates)|hatch_polyline_vertex_coordinate_directory|MissingRequiredValue|CoordinatesUnavailable|ocs_position' crates/seacad-dxf-core/src/hatch_polyline_vertex_coordinate.rs crates/seacad-dxf-core/tests/hatch_polyline_vertex_coordinate_tests.rs crates/seacad-dxf-core/src/lib.rs docs/audits/M14_4N_HATCH_POLYLINE_COORDINATES.md
rg -n 'DxfHatchPolylineSegment(Directory|Endpoints|Entry|PathEntry|PathState|Range|Topology)|hatch_polyline_segment_directory|VertexCountMismatched|Consecutive|Closing|endpoints_for_segment' crates/seacad-dxf-core/src/hatch_polyline_segment.rs crates/seacad-dxf-core/tests/hatch_polyline_segment_tests.rs crates/seacad-dxf-core/src/lib.rs docs/audits/M14_4O_HATCH_POLYLINE_SEGMENTS.md
rg -n 'DxfHatchPolylineSegmentShape(Directory|Entry)?|hatch_polyline_segment_shape_directory|Straight|Arc|Indeterminate|start_bulge' crates/seacad-dxf-core/src/hatch_polyline_segment_shape.rs crates/seacad-dxf-core/tests/hatch_polyline_segment_shape_tests.rs crates/seacad-dxf-core/src/lib.rs docs/audits/M14_4P_HATCH_POLYLINE_SEGMENT_SHAPES.md
rg -n 'DxfHatchPolyline(OcsLineSegment|LineGeometryDirectory|LineGeometryEntry|LineGeometryIssue)|hatch_polyline_line_geometry_directory' crates/seacad-dxf-core/src/hatch_polyline_line_geometry.rs crates/seacad-dxf-core/tests/hatch_polyline_line_geometry_tests.rs crates/seacad-dxf-core/src/lib.rs docs/audits/M14_4Q_HATCH_POLYLINE_LINE_GEOMETRY.md
rg -n 'DxfHatchPolyline(OcsArcSegment|OcsSegmentGeometry|SegmentGeometryDirectory|SegmentGeometryIssue)|hatch_polyline_segment_geometry_directory' crates/seacad-dxf-core/src/hatch_polyline_segment_geometry.rs crates/seacad-dxf-core/tests/hatch_polyline_segment_geometry_tests.rs crates/seacad-dxf-core/src/lib.rs docs/audits/M14_4R_HATCH_POLYLINE_SEGMENT_GEOMETRY.md
rg -n 'DxfHatchPolyline(WcsLineSegment|WcsArcSegment|WcsSegmentGeometry|WcsGeometryDirectory|WcsGeometryIssue)|hatch_polyline_wcs_geometry_directory' crates/seacad-dxf-core/src/hatch_polyline_wcs_geometry.rs crates/seacad-dxf-core/tests/hatch_polyline_wcs_geometry_tests.rs crates/seacad-dxf-core/src/lib.rs docs/audits/M14_4S_HATCH_POLYLINE_WCS_GEOMETRY.md
rg -n 'DxfHatchBoundaryEdge(Count|CountRelation|Directory|Entry|Path|PathIssue|PathState|Range)|hatch_boundary_edge_directory' crates/seacad-dxf-core/src/hatch_boundary_edge.rs crates/seacad-dxf-core/tests/hatch_boundary_edge_tests.rs crates/seacad-dxf-core/src/lib.rs docs/audits/M14_4T_HATCH_BOUNDARY_EDGE_GROUPING.md
rg -n 'DxfHatchBoundaryEdgeType(Directory|Entry|Issue)?|hatch_boundary_edge_type_directory|CircularArc|EllipticArc' crates/seacad-dxf-core/src/hatch_boundary_edge_type.rs crates/seacad-dxf-core/tests/hatch_boundary_edge_type_tests.rs crates/seacad-dxf-core/src/lib.rs docs/audits/M14_4U_HATCH_BOUNDARY_EDGE_TYPES.md
rg -n 'DXF_HATCH_BOUNDARY_CIRCULAR_ARC_EDGE_ROLES|DxfHatchBoundaryCircularArcEdge(Card|CardDirectory|CardState|Member|MemberRange|Role)|hatch_boundary_circular_arc_edge_card_directory|CenterX|Counterclockwise' crates/seacad-dxf-core/src/hatch_boundary_circular_arc_edge_card.rs crates/seacad-dxf-core/tests/hatch_boundary_circular_arc_edge_card_tests.rs crates/seacad-dxf-core/src/lib.rs docs/audits/M14_4AA_HATCH_BOUNDARY_CIRCULAR_ARC_EDGE_CARDS.md
rg -n 'DxfHatchBoundaryCircularArcEdgeNumeric(Components|Directory|DoubleValue|Entry|IntegerValue|Issue)|hatch_boundary_circular_arc_edge_numeric_directory|MultipleValues|InvalidAsciiNumber|NonFiniteDouble|boundary_circular_arc_(center_x|center_y|radius|start_angle|end_angle|counterclockwise)' crates/seacad-dxf-core/src/hatch_boundary_circular_arc_edge_numeric.rs crates/seacad-dxf-core/tests/hatch_boundary_circular_arc_edge_numeric_tests.rs crates/seacad-dxf-core/src/lib.rs docs/audits/M14_4AB_HATCH_BOUNDARY_CIRCULAR_ARC_EDGE_NUMERICS.md
rg -n 'DxfHatchBoundaryCircularArcEdge(Direction|SemanticDirectory|SemanticDirectionValue|SemanticDoubleValue|SemanticEntry|SemanticIssue|Semantics)|hatch_boundary_circular_arc_edge_semantic_directory|MissingRequiredValue|NonPositiveRadius|DirectionFlagOutOfDomain|Clockwise|Counterclockwise' crates/seacad-dxf-core/src/hatch_boundary_circular_arc_edge_semantic.rs crates/seacad-dxf-core/tests/hatch_boundary_circular_arc_edge_semantic_tests.rs crates/seacad-dxf-core/src/lib.rs docs/audits/M14_4AC_HATCH_BOUNDARY_CIRCULAR_ARC_EDGE_SEMANTICS.md
rg -n 'DxfHatchBoundaryCircularArcEdge(GeometryDirectory|GeometryEntry|GeometryIssue|OcsSegment|UnavailableValues)|hatch_boundary_circular_arc_edge_geometry_directory|ValuesUnavailable|start_angle_degrees|end_angle_degrees' crates/seacad-dxf-core/src/hatch_boundary_circular_arc_edge_geometry.rs crates/seacad-dxf-core/tests/hatch_boundary_circular_arc_edge_geometry_tests.rs crates/seacad-dxf-core/src/lib.rs docs/audits/M14_4AD_HATCH_BOUNDARY_CIRCULAR_ARC_EDGE_OCS_GEOMETRY.md
rg -n 'DxfHatchBoundaryCircularArcEdge(WcsGeometryDirectory|WcsGeometryEntry|WcsGeometryIssue|WcsSegment)|hatch_boundary_circular_arc_edge_wcs_geometry_directory|ElevationUnavailable|ExtrusionUnavailable|NonFiniteDerivedGeometry|x_axis|y_axis|OcsBasis' crates/seacad-dxf-core/src/hatch_boundary_circular_arc_edge_geometry.rs crates/seacad-dxf-core/src/hatch_boundary_circular_arc_edge_wcs_geometry.rs crates/seacad-dxf-core/src/hatch_polyline_wcs_geometry.rs crates/seacad-dxf-core/tests/hatch_boundary_circular_arc_edge_wcs_geometry_tests.rs crates/seacad-dxf-core/src/lib.rs docs/audits/M14_4AE_HATCH_BOUNDARY_CIRCULAR_ARC_EDGE_WCS_GEOMETRY.md
rg -n 'DXF_HATCH_BOUNDARY_LINE_EDGE_ROLES|DxfHatchBoundaryLineEdge(Card|CardDirectory|CardState|Member|MemberRange|Role)|hatch_boundary_line_edge_card_directory' crates/seacad-dxf-core/src/hatch_boundary_line_edge_card.rs crates/seacad-dxf-core/tests/hatch_boundary_line_edge_card_tests.rs crates/seacad-dxf-core/src/lib.rs docs/audits/M14_4V_HATCH_BOUNDARY_LINE_EDGE_CARDS.md
rg -n 'DxfHatchBoundaryLineEdgeNumeric(Components|Directory|Entry|Issue|Value)|hatch_boundary_line_edge_numeric_directory|MultipleValues|InvalidAsciiNumber|NonFiniteDouble|boundary_line_(start|end)_(x|y)' crates/seacad-dxf-core/src/hatch_boundary_line_edge_numeric.rs crates/seacad-dxf-core/tests/hatch_boundary_line_edge_numeric_tests.rs crates/seacad-dxf-core/src/lib.rs docs/audits/M14_4W_HATCH_BOUNDARY_LINE_EDGE_NUMERICS.md
rg -n 'DxfHatchBoundaryLineEdge(CoordinateDirectory|CoordinateEntry|CoordinateIssue|CoordinateValue|Coordinates|EndpointIssue|OcsEndpoints|OcsPoint|UnavailableCoordinates)|hatch_boundary_line_edge_coordinate_directory|MissingRequiredValue|CoordinatesUnavailable|ocs_endpoints' crates/seacad-dxf-core/src/hatch_boundary_line_edge_coordinate.rs crates/seacad-dxf-core/tests/hatch_boundary_line_edge_coordinate_tests.rs crates/seacad-dxf-core/src/lib.rs docs/audits/M14_4X_HATCH_BOUNDARY_LINE_EDGE_COORDINATES.md
rg -n 'DxfHatchBoundaryLineEdge(GeometryDirectory|GeometryEntry|GeometryIssue|OcsSegment)|hatch_boundary_line_edge_geometry_directory|EndpointsUnavailable|ocs_endpoints' crates/seacad-dxf-core/src/hatch_boundary_line_edge_geometry.rs crates/seacad-dxf-core/tests/hatch_boundary_line_edge_geometry_tests.rs crates/seacad-dxf-core/src/lib.rs docs/audits/M14_4Y_HATCH_BOUNDARY_LINE_EDGE_OCS_GEOMETRY.md
rg -n 'DxfHatchBoundaryLineEdge(WcsGeometryDirectory|WcsGeometryEntry|WcsGeometryIssue|WcsSegment)|hatch_boundary_line_edge_wcs_geometry_directory|ElevationUnavailable|ExtrusionUnavailable|NonFiniteDerivedGeometry|OcsBasis' crates/seacad-dxf-core/src/hatch_boundary_line_edge_wcs_geometry.rs crates/seacad-dxf-core/src/hatch_polyline_wcs_geometry.rs crates/seacad-dxf-core/tests/hatch_boundary_line_edge_wcs_geometry_tests.rs crates/seacad-dxf-core/src/lib.rs docs/audits/M14_4Z_HATCH_BOUNDARY_LINE_EDGE_WCS_GEOMETRY.md
rg -n 'read_support|compact_(len|u64)|ensure_(not_cancelled|source)|invalid_internal_data|out_of_memory' crates/seacad-dxf-core/src/read_support.rs crates/seacad-dxf-core/src/hatch_boundary_edge.rs crates/seacad-dxf-core/src/hatch_boundary_edge_type.rs crates/seacad-dxf-core/src/hatch_boundary_line_edge_card.rs crates/seacad-dxf-core/src/hatch_boundary_line_edge_numeric.rs crates/seacad-dxf-core/src/hatch_boundary_line_edge_coordinate.rs crates/seacad-dxf-core/src/hatch_boundary_line_edge_geometry.rs crates/seacad-dxf-core/src/hatch_boundary_line_edge_wcs_geometry.rs crates/seacad-dxf-core/src/lib.rs docs/ARCHITECTURE.md docs/audits/DXF_STRUCT_R1A_HATCH_READ_SUPPORT.md
rg -n 'subclass_ordinal_for_entry|entry_ordinal|retained exact path evidence' crates/seacad-dxf-core/src/hatch_boundary_line_edge_geometry.rs crates/seacad-dxf-core/src/hatch_boundary_line_edge_wcs_geometry.rs crates/seacad-dxf-core/src/hatch_polyline_segment_geometry.rs crates/seacad-dxf-core/src/hatch_polyline_wcs_geometry.rs docs/ARCHITECTURE.md docs/audits/DXF_STRUCT_R1B_HATCH_GEOMETRY_RESOLVERS.md
$wcsReachThrough = @(rg -n 'coordinate_directory\(|numeric_directory\(|card_directory\(|edge_type_directory\(|line_geometry_directory\(|shape_directory\(|segment_directory\(' crates/seacad-dxf-core/src/hatch_boundary_line_edge_wcs_geometry.rs crates/seacad-dxf-core/src/hatch_polyline_wcs_geometry.rs)
if ($wcsReachThrough.Count -ne 0) { $wcsReachThrough; throw 'WCS ownership reach-through remains.' }
rg -n 'DXF_SPLINE_EVALUATION_MAX_DEGREE|evaluate_point_for_raw_record|DxfSpline(PointEvaluation|EvaluatedPoint|EvaluationInputKind)|DegreeLimitExceeded|NonFiniteParameter|ParameterOutOfDomain|DegenerateKnotInterval|ArithmeticOverflow|NonPositiveHomogeneousWeight' crates/seacad-dxf-core/src/spline_point_evaluation.rs crates/seacad-dxf-core/tests/spline_point_evaluation_tests.rs crates/seacad-dxf-core/src/lib.rs docs/audits/M14_3EA_SPLINE_POINT_EVALUATION.md
rg -n 'evaluate_first_derivative_for_raw_record|DxfSpline(EvaluatedDifferential|EvaluatedVector|FirstDerivative)|prepare_evaluation|evaluate_homogeneous|homogeneous_to_point|DegenerateKnotInterval|ArithmeticOverflow' crates/seacad-dxf-core/src/spline_first_derivative.rs crates/seacad-dxf-core/src/spline_point_evaluation.rs crates/seacad-dxf-core/tests/spline_first_derivative_tests.rs crates/seacad-dxf-core/src/lib.rs docs/audits/M14_3EB_SPLINE_FIRST_DERIVATIVE.md
```

Expected respectively: 8/8; 4/4 for every focused HATCH/FILL test through
`hatch_boundary_line_edge_wcs_geometry_tests`, including
`hatch_boundary_circular_arc_edge_card_tests` and
`hatch_boundary_circular_arc_edge_numeric_tests`, and
`hatch_boundary_circular_arc_edge_semantic_tests`, and
`hatch_boundary_circular_arc_edge_geometry_tests`, and
`hatch_boundary_circular_arc_edge_wcs_geometry_tests`; 4/4 and 4/4 for the two SPLINE tests; 11/11,
13/13, 18/18, 4/4, 3/3, 5/5, and 10/10 for the remaining tests. For M14.3dr prove create-new-only behavior,
strict reparse/verification, cleanup, exact inverse, all formats/dialects, zero
and non-empty XDATA, identity, bounds, cancellation, tamper rejection, and
redaction. For M14.3ds prove all four format pairs and nine Core dialects,
AC1009/AC1032 boundary behavior, immutable source evidence, reviewed common and
POINT geometry preservation, explicit local bindings, missing/ambiguous layer,
unexpected binding, same-document rejection, cancellation, traits, and
redaction. For M14.3dt prove exact XDATA-entry ownership, safe internal XDATA
admission, standalone rejection, family-plus-XDATA bytes, all format/dialect
pairs plus AC1009-to-AC1032, downstream insertion/strict verification/inverse,
unavailable payload, foreign identity, cancellation, bounds, traits, and
redaction. For M14.3du prove provenance retention through atomic insertion,
strict verification, create-new writing/reparse, receipts, inverse restoration,
existing-file preservation, pre-cancellation, final tamper cleanup, all format/
dialect pairs plus AC1009-to-AC1032, foreign destination rejection, bounds,
traits, and redaction. For M14.3dv prove both AC1009/AC1032 directions over all
four format pairs, exact legacy layout/BY_LAYER adaptation evidence through
write journals, exact output/inverse, empty forward adaptation, and typed
non-representable non-default lineweight rejection. For M14.3dw prove all four
ASCII/Binary format pairs, UTF-8/Windows-1252 both directions, same/cross-
legacy and empty values, replacement-free failure for unmappable and malformed
text, Johab encoder unavailability, unsupported/indeterminate resolutions,
round-trip verification, cancellation, value-byte bounds, identities/counts,
traits, and debug redaction. For M14.3dx prove only group-1000 XDATA strings
transcode while APPID/LAYER remain destination-bound and controls source-exact;
prove UTF-8/Windows-1252 both ways over all four format pairs, compact dual-
source receipts through application/entity grouping, exact Binary AC1018-to-
ASCII AC1021 POINT create-new writing/reparse/XDATA verification/inverse,
ASCII portability, 255-byte destination bounds, unmappable/malformed/
unsupported/indeterminate/Johab failures, identity, traits, metadata bounds,
and redaction. For M14.3dy prove group-430 POINT color-book names transcode
UTF-8/Windows-1252 both ways over all four format pairs while layer/layout/
linetype remain explicit destination bindings; prove exact field-span receipts
through family/XDATA draft, insertion, verification and create-new journals,
Binary AC1018-to-ASCII AC1021 write/reparse/color-plus-XDATA verification/
inverse, matching-decoder exact bytes, typed unmappable failure, AC1009 non-
representability, bounded legacy-decoder terminal slack, cancellation, traits,
identity, and redaction. For M14.3dz prove POINT is exactly level 5, satisfies
levels 1-5 but not level 6, retains exactly the two release-evidence blockers,
is not complete, and has compact stable metadata. Confirm the audit maps every
satisfied level to existing evidence and does not treat rendering, POINT display behavior,
application-specific XDATA interpretation, or automatic symbol creation as an
entity-completion blocker. For M14.3ea prove all 18 ASCII/Binary dialect
variants evaluate identical quadratic endpoint/midpoint bits, rational weights
use homogeneous division, the closed parameter domain and terminal span are
exact, degree 64 bounds CPU/scratch work, and unavailable analytic data,
non-finite/out-of-domain parameters, non-finite inputs, degenerate knots,
overflow, nonpositive resulting weight, cancellation, missing lookup, traits,
and metadata size remain explicit. Confirm no sampling, derivatives,
tessellation, rendering, HELIX evaluation, CRUD/write, or `Complete` claim is
added. For M14.3eb prove the shared preflight/De Boor refactor preserves the
M14.3ea 4/4 results; all 18 dialect/encoding variants retain bit-identical
quadratic points and first derivatives at endpoints/midpoint; nonconstant
rational weights satisfy the homogeneous quotient rule; degree 64 bounds both
scratch sets and the combined 4,096 recurrences; typed failures, cancellation,
lookup, traits, and metadata bounds remain explicit; zero derivative stays
valid; and no normalization, frame, curvature, sampling, tessellation, HELIX,
CRUD/write, or `Complete` claim is added. For M14.3ec prove POINT remains
exactly level 5 with its two release blockers;
SPLINE is exactly level 4 with mutation plus two release blockers; HELIX is
exactly level 3 with public-geometry, mutation, and two release blockers; all
three remain incomplete; the public list is unique and sorted by canonical
topic ordinal; binary lookup is deterministic; 42 topics remain unaudited; and
the HELIX Autodesk warning is not converted into a geometry claim. For M14.4a
prove exact canonical record and subclass matching, separate duplicate
subclass scopes, raw field/source-order/payload provenance, nested group-code
collisions without semantic roles, complete BLOCKS/ENTITIES filtering, all
nine ASCII/Binary dialect pairs, application group/XDATA exclusion from family
evidence, source identity, cancellation, bounds, traits, and compact metadata.
Confirm HATCH applicability remains unreviewed, MESH remains AC1024-and-later,
and no cardinality, semantic, topology, geometry, subdivision, CRUD/write, or
completion claim is added. For M14.4b prove all 25 fixed roles and exact
Text/Double/Int16/Int32 wire domains, especially Int16 group 78 and Int32
groups 90-99/450-453; exact text spans, duplicate/invalid ordered evidence,
independent duplicate subclasses, nested boundary/pattern/seed decoy exclusion,
MESH and near/missing marker exclusion, all nine ASCII/Binary dialect pairs,
AC1009 high-code omission parity, source identity, cancellation, bounds,
traits, and compact metadata. Confirm applicability stays unreviewed and no
cardinality, defaults/domains, tuples, nested state, geometry, CRUD/write, or
completion claim is added. For M14.4c prove exactly 25 ordered cards per HATCH
subclass; absent/unique/multiple states and exact occurrence counts; compact
member-to-original-occurrence resolution; invalid-value/cardinality
independence; duplicate subclass isolation plus contiguous raw-record lookup;
all nine ASCII/Binary card/member pairs with nine AC1009 high-code absences;
source identity, cancellation, bounds, traits, and compact metadata. Confirm no
selection, defaults/domains, relations, tuples, nested state, applicability,
geometry, CRUD/write, or completion claim is added. For M14.4d prove 25 ordered
four-state semantics per subclass; exact `entity.hatch` field provenance;
extrusion-only `(0,0,1)` defaults; all other absences including gradient fields;
duplicate no-selection behavior; invalid ASCII and non-finite Binary raw
provenance; every reviewed flag/style/type/count/reserved/mode/count/shift/tint
domain; duplicate subclass isolation; all nine ASCII/Binary pairs with nine
AC1009 high-code absences; cancellation, bounds, traits, and redaction. Confirm
no conditional gradient defaults, tuple/zero-vector/requiredness/cross-field
relations, nested state, applicability, geometry, CRUD/write, or completion
claim is added. For M14.4e prove exact non-normalized extrusion assembly;
explicit/defaulted provenance for complete, absent, and partial tuples;
unavailable-component masks for duplicate, malformed, and non-finite evidence;
exact signed-zero vector rejection; duplicate-subclass isolation; all nine
ASCII/Binary dialect pairs; cancellation, lookup bounds, traits, source
identity, and compact metadata. Confirm elevation remains deferred because
groups 10/20 require stateful HATCH partitioning, and no transform, geometry,
applicability, CRUD/write, or completion claim is added. For M14.4f prove one
entry per exact HATCH subclass; unique ordered group-91/group-75 fences; exact
header, opaque-boundary, and trailing ranges covering every retained field;
nested 10/20 collisions and unknown payload preservation; missing, duplicate,
and reversed-anchor typed issues without slice publication; duplicate-subclass
isolation; all nine ASCII/Binary dialect pairs; cancellation, source identity,
lookup bounds, traits, and compact metadata. Confirm no path-count relation,
path/edge decoding, elevation selection, later-state partition, geometry,
applicability, CRUD/write, or completion claim is added. For M14.4g prove one
elevation entry per exact HATCH subclass; header-only unique group 10/20/30
selection with exact binary64/raw provenance; positive and negative zero X/Y;
finite Z; exclusion of boundary, seed, trailing, MESH, and duplicate-subclass
decoys; typed partition, absent, multiple, malformed ASCII, non-finite Binary,
and nonzero-planar failures plus exact masks; all nine ASCII/Binary dialect
pairs; cancellation, source identity, lookup bounds, traits, and compact
metadata. Confirm no defaults, OCS/WCS transform, nested topology/state,
geometry, applicability, CRUD/write, or completion claim is added. For M14.4h
prove one topology entry per exact HATCH subclass; group-92 anchor
grouping with exact raw markers and payload ranges; explicit pre-anchor orphan
fields; empty paths/payloads/orphans; group-91 signed Int32 matched, mismatched,
malformed ASCII, and negative relations; partition failure without published
slices; duplicate-subclass isolation; all nine ASCII/Binary dialect pairs;
cancellation, source identity, lookup bounds, traits, and compact metadata.
Confirm no flag semantics, polyline/edge branching, payload cardinality,
handles, geometry, applicability, CRUD/write, or completion claim is added. For
M14.4i prove exact signed Int32 group-92 decoding and the reviewed Polyline,
External, and Outermost bits; absent, malformed, negative, and unsupported-bit
states; duplicate-subclass isolation; all nine ASCII/Binary dialect pairs;
cancellation, source identity, lookup bounds, traits, and compact metadata.
Confirm no payload grammar/cardinality, vertex/edge decoding, handles, geometry,
applicability, CRUD/write, or completion claim is added. For M14.4j prove only
valid Polyline paths select unique groups 72/73/93; both flags accept exactly
0/1 and declared vertex count is nonnegative; every selected value retains its
raw group; Edges are NotPolyline; invalid flags, absence, duplicates, malformed
ASCII, and domain failures remain typed; and all nine dialects retain parity.
Confirm vertex grouping/count reconciliation, bulge decoding, geometry,
applicability, CRUD/write, and completion remain open. For M14.4k prove every
group 10 opens one vertex, following groups 20/42 attach until the next anchor,
pre-anchor 20/42 remain ordered orphans, X is unique by anchor construction,
Y/bulge expose absent/unique/multiple cardinality, and observed count compares
with group 93. Prove Edges and invalid headers publish no vertices, unknown
fields/source handles remain raw evidence, all nine dialects retain parity, and
cancellation/source identity/lookup bounds/traits/metadata remain bounded.
Confirm numeric decoding/defaults, geometry, applicability, CRUD/write, and
completion remain open. For M14.4l prove one numeric entry per grouped M14.4k
vertex; exact finite binary64 decoding for unique X/Y/bulge; signed-zero bit
preservation; `entity.hatch` field provenance; absent Y/bulge without defaults;
duplicate invalidity without member selection; malformed ASCII and non-finite
Binary failures with raw provenance; Edges and invalid headers publishing no
entries; all nine ASCII/Binary dialect pairs; cancellation, source identity,
lookup bounds, traits, and debug redaction. Confirm required-Y and has-bulge
relations, bulge defaults, OCS/WCS geometry, applicability, CRUD/write, and
completion remain open. For M14.4m prove absent group 42 defaults to exact
`+0.0` without raw provenance under both has-bulge states; enabled headers pass
through explicit and typed-invalid M14.4l semantics; disabled headers reject
unique and duplicate group 42 with exact card state while retaining the full
numeric directory; header group/provenance remains available; Edges and invalid
headers publish no entries; all nine ASCII/Binary dialect pairs; cancellation,
source identity, lookup bounds, traits, and debug redaction. Confirm required-Y
semantics, closed topology, OCS/WCS geometry, applicability, CRUD/write, and
completion remain open. For M14.4n prove one coordinate entry per M14.4m bulge
entry; required source-anchored X/Y semantics; exact signed-zero OCS tuples when
both components are usable; missing Y as a required-value issue without raw
provenance; duplicate, malformed ASCII, and non-finite Binary failures retaining
their numeric issue and available provenance; exact X/Y unavailable masks; the
complete bulge/header directory retained; Edges and invalid headers publishing
no entries; all nine ASCII/Binary dialect pairs; cancellation, source identity,
lookup bounds, traits, and debug redaction. Confirm OCS/WCS transformation,
closed topology, segment geometry, applicability, CRUD/write, and completion
remain open. For M14.4o prove one path entry per M14.4n path; topology only on
matched group-93 count; open `n-1` adjacency and closed last-to-first addition;
one-vertex self-closing and zero-vertex empty behavior; mismatch, Edges, and
invalid headers publishing typed path states with no segments; coordinate
failure retaining topology; compact path/segment/endpoint ordinals resolving
exact M14.4n entries; all nine ASCII/Binary dialect pairs; cancellation, source
identity, lookup bounds, traits, and compact metadata. Confirm segment shape,
bulge arc construction, OCS/WCS transformation, applicability, CRUD/write, and
completion remain open. For M14.4p prove one shape entry per M14.4o segment;
defaulted and explicit positive/negative zero as Straight; every finite nonzero
bulge as Arc with exact bits; numeric and group-72 relation failures as
Indeterminate with the original typed issue; coordinate failure independent of
shape; exact start-bulge state/raw provenance resolvable; unavailable paths
publishing no entries; all nine ASCII/Binary dialect pairs; cancellation,
source identity, lookup bounds, traits, redaction, and compact metadata. Confirm
arc center/radius/angle construction, line geometry, OCS/WCS transformation,
applicability, CRUD/write, and completion remain open.
For M14.4q prove exact OCS endpoints only for Straight segments; Arc and
Indeterminate shapes plus unavailable endpoint coordinates remain distinct
typed outcomes; every M14.4o segment retains one aligned entry; all nine
ASCII/Binary dialect pairs, cancellation, source identity, bounds, traits, and
redaction hold. Confirm no arc construction, WCS transformation,
applicability, CRUD/write, or completion claim. For M14.4r prove finite OCS
center/radius/signed sweep for every usable nonzero bulge arc; exact line
pass-through; degenerate chord and non-finite derived arithmetic remain typed;
all nine dialect pairs and safety properties hold. Confirm no WCS transform,
tessellation, applicability, CRUD/write, or completion claim. For M14.4s prove
finite WCS line/arc projection from exact elevation plus normalized explicit or
defaulted extrusion using the documented arbitrary-axis basis and `1/64`
branch; preserve radius, signed sweep, bulge, and normalized normal; enforce
source/elevation/extrusion/derived failure precedence; all nine dialect pairs
include non-axis and negative normals. Confirm derived WCS bits are not raw or
cross-platform canonical and applicability/CRUD/write/completion remain open.
For M14.4t prove only Edges paths select one nonnegative group-93 count and
group every later group-72 marker into exact conservative payload slices;
Matched/Mismatched counts and zero edges remain explicit; invalid flags/counts
and marker-before-count publish no edges; all nine dialect pairs and safety
properties hold. Confirm edge types and payload semantics remain open. For
M14.4u prove exactly one signed Int16 type result per grouped edge with 1 Line,
2 CircularArc, 3 EllipticArc, and 4 Spline; malformed ASCII and every other
signed value remain exact typed issues; count mismatch retains types, empty
grouped paths expose empty slices, and Polyline/unavailable paths expose no
entries; all nine dialect pairs and safety properties hold. Confirm payload
fields and geometry remain open. For M14.4v prove four ordered cards for every
Line edge only: StartX 10, StartY 20, EndX 11, EndY 21; exact source fields and
independent Absent/Unique/Multiple states; mismatch does not erase cards;
non-Line and invalid type markers expose empty card slices while the complete
M14.4u directory remains retained; all nine dialect pairs, cancellation,
identity, bounds, traits, compact metadata, and redaction hold. Confirm numeric
selection, required-coordinate semantics, OCS/WCS geometry, other edge payload
families, applicability, CRUD/write, and completion remain open.
For M14.4w prove one numeric entry per Line edge with four source-stable
components; finite Explicit values and bit-exact signed zero; exact
`entity.hatch` field/raw provenance; Absent fields without requiredness or
defaults; duplicate counts without winner selection; malformed ASCII and
non-finite Binary typed issues with unique raw provenance; mismatch retention;
non-Line, invalid, and Polyline paths publishing no numeric entries; all nine
dialect pairs, cancellation, identity, bounds, traits, and redaction. Confirm
required OCS tuples, OCS/WCS geometry, other edge payload families,
applicability, CRUD/write, and completion remain open.
For M14.4x prove one coordinate entry per M14.4w Line numeric entry; all four
components are required and retain exact field/raw provenance; usable values
assemble bit-exact OCS start/end points; absent values become
MissingRequiredValue without raw provenance; duplicate, malformed ASCII, and
non-finite Binary values retain their numeric issue and available provenance;
failure exposes an exact StartX/StartY/EndX/EndY mask; mismatch retains usable
coordinates; non-Line, invalid, and Polyline paths publish no entries; all nine
dialect pairs, cancellation, identity, bounds, traits, and redaction hold.
Confirm OCS/WCS geometry, other edge payload families, applicability,
CRUD/write, rendering, and completion remain open.
For M14.4y prove one geometry entry per M14.4x coordinate entry; usable OCS
endpoints pass through without arithmetic into an exact Line segment; signed
zero and zero-length Lines remain valid; unavailable endpoints retain the exact
four-component mask and lower-layer issue/provenance; mismatch keeps usable
geometry; non-Line, invalid, and Polyline paths publish no entries; all nine
dialect pairs, cancellation, identity, bounds, traits, and redaction hold.
Confirm no OCS-to-WCS projection, other edge payload geometry, applicability,
CRUD/write, rendering, or completion claim.
For M14.4z prove each usable M14.4y Line joins the exact subclass elevation and
explicit/defaulted extrusion, then projects finite WCS endpoints and a
normalized normal through the shared M14.4s arbitrary-axis basis and exact
`1/64` branch. Prove source/elevation/extrusion/derived failure precedence,
non-axis-aligned and negative normals, mismatch retention, non-Line exclusion,
all nine ASCII/Binary dialect pairs, cancellation, identity, bounds, compact
metadata, traits, receipts, and redaction. Confirm derived WCS bits are not raw
or cross-platform canonical evidence and other edge payload geometry,
applicability, CRUD/write, rendering, and completion remain open.
For DXF-STRUCT-R1a prove the seven M14.4t-M14.4z domain modules import the
crate-private shared helpers and define no local copy of `compact_len`,
`compact_u64`, `ensure_source`, `ensure_not_cancelled`,
`invalid_internal_data`, `out_of_memory`, or their `Read` error mapper. Prove
all seven focused suites remain 4/4, exact error/cancellation/source-identity/
bounds/redaction behavior is unchanged, no public export or support claim is
added, and production adds 75 lines, deletes 264, for a net 189-line removal.
For DXF-STRUCT-R1b prove both WCS constructors call only the immediate source-
geometry resolver for subclass ownership and contain zero lower-directory
accessor calls. Prove each resolver selects a local entry ordinal and derives
the same subclass from retained exact path evidence; both focused suites remain
4/4; public API, struct fields/layout, raw bytes, provenance, typed failures,
support, dependencies, and the 1,193-test workspace count remain unchanged.
Confirm production adds 28 lines, deletes 17, and removes the two upper-layer
dependency paths without copying owner state.
For M14.4aa prove six ordered cards per CircularArc edge with exact groups
10/20/40/50/51/73; independent Absent/Unique/Multiple states; duplicate member
source order; mismatch retention; no cards for Line, EllipticArc, Spline,
invalid, empty, or Polyline states; all nine ASCII/Binary dialect pairs;
cancellation, identity, bounds, compact traits, and redaction. Confirm no value
selection, requiredness, defaults/domains, radius/angle/direction semantics,
OCS/WCS geometry, applicability, CRUD/write, rendering, or completion claim.
For M14.4ab prove one numeric entry per CircularArc edge with five finite
binary64 components plus one exact signed Int16 direction value; bit-exact
signed zero; exact `entity.hatch` field/raw provenance; independent Absent and
Multiple states without winner selection; malformed ASCII and non-finite
Binary typed issues; count-mismatch retention; no entries for Line,
EllipticArc, Spline, invalid, empty, or Polyline states; all nine ASCII/Binary
dialect pairs; cancellation, identity, bounds, compact traits, and redaction.
Confirm no requiredness, positive-radius, angle normalization, direction 0/1
domain, OCS/WCS geometry, applicability, CRUD/write, rendering, or completion
claim.
For M14.4ac prove all six CircularArc values are required; absent values become
MissingRequiredValue without invented raw provenance; duplicate, malformed
ASCII, and non-finite Binary values retain their exact M14.4ab issue and
available raw provenance; radius accepts only finite values strictly greater
than zero while preserving positive-zero, negative-zero, and negative bits in
NonPositiveRadius failures; direction 0 maps Clockwise, 1 maps
Counterclockwise, and every other signed Int16 remains
DirectionFlagOutOfDomain with exact raw provenance. Prove negative and large
angles remain exact DXF degrees without normalization, wrapping, or derived
sweep; mismatch retention; no entries for Line, EllipticArc, Spline, invalid,
empty, or Polyline states; all nine ASCII/Binary dialect pairs; cancellation,
identity, bounds, compact traits, and redaction. Confirm no OCS/WCS CircularArc
geometry, other edge payload semantics, applicability, CRUD/write, rendering,
or completion claim.
For M14.4ad prove one OCS CircularArc geometry entry per M14.4ac semantic entry;
usable center, strictly positive radius, exact start/end DXF degrees, and
Clockwise/Counterclockwise direction pass through without arithmetic;
unavailable geometry carries the exact six-component mask while retaining the
complete semantic/numeric issue and provenance chain; negative, large, equal,
and signed-zero values remain bit-exact; equal angles do not imply zero sweep
or a full turn; mismatch retention; no entries for Line, EllipticArc, Spline,
invalid, empty, or Polyline states; all nine ASCII/Binary dialect pairs;
cancellation, identity, bounds, compact traits, and redaction. Confirm no angle
wrapping, signed sweep, endpoint derivation, OCS-to-WCS projection, other edge
payload semantics, applicability, CRUD/write, rendering, or completion claim.
For M14.4ae prove every usable M14.4ad OCS CircularArc joins its exact subclass
elevation and explicit/defaulted extrusion, then projects a finite WCS center,
finite WCS X/Y axes, and normalized normal through the shared M14.4s
arbitrary-axis basis and exact `1/64` branch. Prove exact source radius,
start/end DXF degrees, and Clockwise/Counterclockwise direction pass through
unchanged; source/elevation/extrusion/derived failure precedence;
non-axis-aligned and negative normals; mismatch retention; non-CircularArc
exclusion; all nine ASCII/Binary dialect pairs; cancellation, identity, bounds,
compact traits, direct evidence receipts, and redaction. Confirm no endpoint,
signed-sweep, wrapping, or trigonometric derivation; derived WCS bits are not
raw or cross-platform canonical evidence; other edge payload geometry,
applicability, CRUD/write, rendering, and completion remain open.
Require zero production matches:

```powershell
rg -n 'panic!|unwrap\(|expect\(|todo!|unimplemented!|unsafe' crates/seacad-dxf-core/src/read_support.rs crates/seacad-dxf-core/src/entity_completion.rs crates/seacad-dxf-core/src/entity_xdata_draft_write.rs crates/seacad-dxf-core/src/entity_xdata_encoded_destination.rs crates/seacad-dxf-core/src/fill_mesh_evidence.rs crates/seacad-dxf-core/src/hatch_boundary_partition.rs crates/seacad-dxf-core/src/hatch_boundary_path.rs crates/seacad-dxf-core/src/hatch_boundary_path_flags.rs crates/seacad-dxf-core/src/hatch_boundary_edge.rs crates/seacad-dxf-core/src/hatch_boundary_edge_type.rs crates/seacad-dxf-core/src/hatch_boundary_circular_arc_edge_card.rs crates/seacad-dxf-core/src/hatch_boundary_circular_arc_edge_geometry.rs crates/seacad-dxf-core/src/hatch_boundary_circular_arc_edge_wcs_geometry.rs crates/seacad-dxf-core/src/hatch_boundary_circular_arc_edge_numeric.rs crates/seacad-dxf-core/src/hatch_boundary_circular_arc_edge_semantic.rs crates/seacad-dxf-core/src/hatch_boundary_line_edge_card.rs crates/seacad-dxf-core/src/hatch_boundary_line_edge_coordinate.rs crates/seacad-dxf-core/src/hatch_boundary_line_edge_geometry.rs crates/seacad-dxf-core/src/hatch_boundary_line_edge_wcs_geometry.rs crates/seacad-dxf-core/src/hatch_boundary_line_edge_numeric.rs crates/seacad-dxf-core/src/hatch_elevation.rs crates/seacad-dxf-core/src/hatch_extrusion.rs crates/seacad-dxf-core/src/hatch_polyline_bulge.rs crates/seacad-dxf-core/src/hatch_polyline_header.rs crates/seacad-dxf-core/src/hatch_polyline_line_geometry.rs crates/seacad-dxf-core/src/hatch_polyline_segment.rs crates/seacad-dxf-core/src/hatch_polyline_segment_geometry.rs crates/seacad-dxf-core/src/hatch_polyline_segment_shape.rs crates/seacad-dxf-core/src/hatch_polyline_vertex.rs crates/seacad-dxf-core/src/hatch_polyline_vertex_coordinate.rs crates/seacad-dxf-core/src/hatch_polyline_vertex_numeric.rs crates/seacad-dxf-core/src/hatch_polyline_wcs_geometry.rs crates/seacad-dxf-core/src/hatch_scalar_card.rs crates/seacad-dxf-core/src/hatch_scalar_evidence.rs crates/seacad-dxf-core/src/hatch_scalar_semantic.rs crates/seacad-dxf-core/src/point_clone_draft_projection.rs crates/seacad-dxf-core/src/point_clone_xdata_draft.rs crates/seacad-dxf-core/src/point_clone_xdata_insert.rs crates/seacad-dxf-core/src/entity_edit_session.rs crates/seacad-dxf-core/src/encoding.rs crates/seacad-dxf-core/src/spline_first_derivative.rs crates/seacad-dxf-core/src/spline_point_evaluation.rs crates/seacad-dxf-core/src/text_decoder.rs crates/seacad-dxf-core/src/text_encoder.rs crates/seacad-dxf-core/src/text_transcode.rs
```

Validate every local Markdown link in all changed overview/plan/audit files.
Require empty protected diff and clean whitespace:

```powershell
git diff 028d726b7f04709927e0d6c8c8d419dc1eb49fcb..m14.4ae-hatch-boundary-circular-arc-edge-wcs-geometry -- Cargo.toml Cargo.lock LICENSE NOTICE THIRD_PARTY_NOTICES.md schema corpus release .github .agents
git diff --check 028d726b7f04709927e0d6c8c8d419dc1eb49fcb..m14.4ae-hatch-boundary-circular-arc-edge-wcs-geometry
```

## Required gates and postflight

Run separately, all exit zero:

```powershell
cargo deny --locked check
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 run --locked -p seacad-schema-gen -- --check
cargo +1.97.1 run --locked -p seacad-schema-gen --bin seacad-release-evidence -- --check
cargo +1.97.1 clippy --workspace --all-targets -- -D warnings
cargo +1.97.1 test --workspace
$listed = @(cargo +1.97.1 test --workspace -- --list 2>$null | Select-String ': test$')
if ($listed.Count -ne 1213) { throw 'Workspace test count drift.' }
git diff --check 028d726b7f04709927e0d6c8c8d419dc1eb49fcb..m14.4ae-hatch-boundary-circular-arc-edge-wcs-geometry
```

Repeat preflight, receipts, protected diff, and whitespace. HEAD/tag/path sets/
hashes must match and worktree remain clean. PASS requires exact state,
receipts, all focused expected counts, all contract/safety/link/protected checks,
all gates, exactly 1,213 tests, no mutation, and the
external report.

Write UTF-8 YAML-shaped evidence with batch id
`seacad-m14.3dr-m14.4ae-hatch-boundary-circular-arc-edge-wcs-geometry-2026-08-10`, status/root/
report/timestamps, HEAD/tags/Git/path sets before/after, every command and hash
receipt, focused tests, workspace total, links, forbidden scan, protected
surfaces, mutations, deviations, failures, blocker, and final assessment. After
writing, print only the report path and status.
