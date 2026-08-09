# Antigravity Mechanical Verification Batch

Status: READY

Checkpoint: `M14.3dr-M14.4h-hatch-boundary-paths`
Repository root: `D:\SeaCad\SeaCad`
Baseline: `028d726b7f04709927e0d6c8c8d419dc1eb49fcb`
Review target: annotated tag `m14.4h-hatch-boundary-paths`
Report: `D:\SeaCad\AntigravityReports\seacad-m14.3dr-m14.4h-hatch-boundary-paths-2026-08-09.yaml`
Prepared: `2026-08-09` (`Asia/Saigon`)

The preceding M14.3dl-M14.3dq cumulative batch was independently reviewed PASS
at exact baseline HEAD. Its external report SHA-256 is
`b27b982faee5ad86a47ca87346b4ac383a210097596ab80c9ed9c41ceec8e665`.
Do not repeat that retired batch. This READY batch intentionally accumulates
M14.3dr through M14.4h so implementation does not pause between checkpoints.

## Authority, procedure, and writes

Mechanically verify M14.3dr-M14.4h only. Codex/user retain architecture,
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
HEAD == m14.4h-hatch-boundary-paths^{}
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
all twenty review tags are annotated tag objects
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
crates/seacad-dxf-core/src/hatch_boundary_partition.rs
crates/seacad-dxf-core/src/hatch_elevation.rs
crates/seacad-dxf-core/src/hatch_boundary_path.rs
crates/seacad-dxf-core/src/lib.rs
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
crates/seacad-dxf-core/tests/hatch_boundary_partition_tests.rs
crates/seacad-dxf-core/tests/hatch_elevation_tests.rs
crates/seacad-dxf-core/tests/hatch_boundary_path_tests.rs
crates/seacad-dxf-core/tests/spline_first_derivative_tests.rs
crates/seacad-dxf-core/tests/spline_point_evaluation_tests.rs
crates/seacad-dxf-core/tests/text_transcode_tests.rs
docs/ANTIGRAVITY_MECHANICAL_BATCH.md
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
docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md
docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md
```

Compare paths as sets.

## Artifact receipts

| Path | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 210 | `19aa21184a9998906ea0fd286a6b4a01df3ffd34140d68cce17e02f458bfe6d6` |
| `README.vi.md` | 208 | `2727800588ae493a208c0f14b5c75ff4835eee8dbf3c9aab2c7e5e0e474a1eeb` |
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
| `crates/seacad-dxf-core/src/hatch_boundary_partition.rs` | 350 | `4cc56cbefc4d1eb73dfa5baaf9a3f0867774df56c1e8a347e4ec71ce07dc98d8` |
| `crates/seacad-dxf-core/src/hatch_elevation.rs` | 443 | `2543b5459525495f3af6a16b47fe0fcf78d64abdb09084d14de4edf46caaaf7d` |
| `crates/seacad-dxf-core/src/hatch_boundary_path.rs` | 476 | `7d5557d42e3b208badd81ef9678c3c959c7fdc3595388b1696c2e72e47034d87` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,314 | `04414c977b49dfaded65df8063c57b03f3930e867f32fbf2c3913440b38bd280` |
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
| `crates/seacad-dxf-core/tests/hatch_boundary_partition_tests.rs` | 397 | `e6cbc4677ebbb30347e88434c54526629f7552d5030023397d6beef03162ae3b` |
| `crates/seacad-dxf-core/tests/hatch_elevation_tests.rs` | 404 | `cc9216d83d712a4028828f8492ad804b24c7d9e156b962dfac95b5f78430c052` |
| `crates/seacad-dxf-core/tests/hatch_boundary_path_tests.rs` | 411 | `71adb9b4690fc2cd2f0ac81e74158bcfb987410d69dced21e11567b94d148118` |
| `crates/seacad-dxf-core/tests/spline_first_derivative_tests.rs` | 367 | `4b6306081db5d1ca6cba239f0fecc09dbcf3506e87e3d04920a9c0a09a9c3dbb` |
| `crates/seacad-dxf-core/tests/spline_point_evaluation_tests.rs` | 348 | `bc72c88c95a4fc983ff0e64a409d539f8d8d6b460d8e9c5d37e6cb0972e2fdc3` |
| `crates/seacad-dxf-core/tests/text_transcode_tests.rs` | 476 | `9bda1ee19362b058ab9a174705cc0d5893f73d13bdb8e008b6e4a7d2d2340a40` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `8d6dac3fa90a386ead0bd9ca9f33114abb0bd3db14d5e069890daddcaed126f9` |
| `docs/SUPPORT_MATRIX.md` | 3,093 | `62e046722a44805de24e615753594773adfe23d7b66ebb240dbb41502fe64ca4` |
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
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,011 | `9116d3ac753d66e3169ad54e378aff762227ce2c77c4ea9678320c3e21c429ed` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,432 | `c2d6097cde468c7b58b3f11471112b841707325eee9e3091a4d18614c688483d` |

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
cargo +1.97.1 test -p seacad-dxf-core --test hatch_boundary_partition_tests
cargo +1.97.1 test -p seacad-dxf-core --test hatch_elevation_tests
cargo +1.97.1 test -p seacad-dxf-core --test hatch_boundary_path_tests
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
rg -n 'DXF_SPLINE_EVALUATION_MAX_DEGREE|evaluate_point_for_raw_record|DxfSpline(PointEvaluation|EvaluatedPoint|EvaluationInputKind)|DegreeLimitExceeded|NonFiniteParameter|ParameterOutOfDomain|DegenerateKnotInterval|ArithmeticOverflow|NonPositiveHomogeneousWeight' crates/seacad-dxf-core/src/spline_point_evaluation.rs crates/seacad-dxf-core/tests/spline_point_evaluation_tests.rs crates/seacad-dxf-core/src/lib.rs docs/audits/M14_3EA_SPLINE_POINT_EVALUATION.md
rg -n 'evaluate_first_derivative_for_raw_record|DxfSpline(EvaluatedDifferential|EvaluatedVector|FirstDerivative)|prepare_evaluation|evaluate_homogeneous|homogeneous_to_point|DegenerateKnotInterval|ArithmeticOverflow' crates/seacad-dxf-core/src/spline_first_derivative.rs crates/seacad-dxf-core/src/spline_point_evaluation.rs crates/seacad-dxf-core/tests/spline_first_derivative_tests.rs crates/seacad-dxf-core/src/lib.rs docs/audits/M14_3EB_SPLINE_FIRST_DERIVATIVE.md
```

Expected 8/8, 4/4, 4/4, 4/4, 4/4, 4/4, 4/4, 4/4, 4/4, 4/4, 4/4, 11/11, 13/13, 18/18, 4/4, 3/3, 5/5, and 10/10. For M14.3dr prove create-new-only behavior,
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
handles, geometry, applicability, CRUD/write, or completion claim is added.
Require zero production matches:

```powershell
rg -n 'panic!|unwrap\(|expect\(|todo!|unimplemented!|unsafe' crates/seacad-dxf-core/src/entity_completion.rs crates/seacad-dxf-core/src/entity_xdata_draft_write.rs crates/seacad-dxf-core/src/entity_xdata_encoded_destination.rs crates/seacad-dxf-core/src/fill_mesh_evidence.rs crates/seacad-dxf-core/src/hatch_boundary_partition.rs crates/seacad-dxf-core/src/hatch_boundary_path.rs crates/seacad-dxf-core/src/hatch_elevation.rs crates/seacad-dxf-core/src/hatch_extrusion.rs crates/seacad-dxf-core/src/hatch_scalar_card.rs crates/seacad-dxf-core/src/hatch_scalar_evidence.rs crates/seacad-dxf-core/src/hatch_scalar_semantic.rs crates/seacad-dxf-core/src/point_clone_draft_projection.rs crates/seacad-dxf-core/src/point_clone_xdata_draft.rs crates/seacad-dxf-core/src/point_clone_xdata_insert.rs crates/seacad-dxf-core/src/entity_edit_session.rs crates/seacad-dxf-core/src/encoding.rs crates/seacad-dxf-core/src/spline_first_derivative.rs crates/seacad-dxf-core/src/spline_point_evaluation.rs crates/seacad-dxf-core/src/text_decoder.rs crates/seacad-dxf-core/src/text_encoder.rs crates/seacad-dxf-core/src/text_transcode.rs
```

Validate every local Markdown link in all changed overview/plan/audit files.
Require empty protected diff and clean whitespace:

```powershell
git diff 028d726b7f04709927e0d6c8c8d419dc1eb49fcb..m14.4h-hatch-boundary-paths -- Cargo.toml Cargo.lock LICENSE NOTICE THIRD_PARTY_NOTICES.md schema corpus release .github .agents
git diff --check 028d726b7f04709927e0d6c8c8d419dc1eb49fcb..m14.4h-hatch-boundary-paths
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
if ($listed.Count -ne 1121) { throw 'Workspace test count drift.' }
git diff --check 028d726b7f04709927e0d6c8c8d419dc1eb49fcb..m14.4h-hatch-boundary-paths
```

Repeat preflight, receipts, protected diff, and whitespace. HEAD/tag/path sets/
hashes must match and worktree remain clean. PASS requires exact state,
receipts, 8/8, 4/4, 4/4, 4/4, 4/4, 4/4, 4/4, 4/4, 4/4, 4/4, 4/4, 11/11, 13/13, 18/18, 4/4, 3/3, 5/5, 10/10, all
contract/safety/link/protected checks, all gates, exactly 1,121 tests, no mutation, and the
external report.

Write UTF-8 YAML-shaped evidence with batch id
`seacad-m14.3dr-m14.4h-hatch-boundary-paths-2026-08-09`, status/root/
report/timestamps, HEAD/tags/Git/path sets before/after, every command and hash
receipt, focused tests, workspace total, links, forbidden scan, protected
surfaces, mutations, deviations, failures, blocker, and final assessment. After
writing, print only the report path and status.
