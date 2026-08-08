# Antigravity Mechanical Verification Batch

Status: READY

Checkpoint: `M14.3dr-M14.3dz-point-completion-ledger`
Repository root: `D:\SeaCad\SeaCad`
Baseline: `028d726b7f04709927e0d6c8c8d419dc1eb49fcb`
Review target: annotated tag `m14.3dz-point-completion-ledger`
Report: `D:\SeaCad\AntigravityReports\seacad-m14.3dr-m14.3dz-point-completion-ledger-2026-08-09.yaml`
Prepared: `2026-08-09` (`Asia/Saigon`)

The preceding M14.3dl-M14.3dq cumulative batch was independently reviewed PASS
at exact baseline HEAD. Its external report SHA-256 is
`b27b982faee5ad86a47ca87346b4ac383a210097596ab80c9ed9c41ceec8e665`.
Do not repeat that retired batch. This READY batch intentionally accumulates
M14.3dr through M14.3dz so implementation does not pause between checkpoints.

## Authority, procedure, and writes

Mechanically verify M14.3dr-M14.3dz only. Codex/user retain architecture,
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
HEAD == m14.3dz-point-completion-ledger^{}
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
all nine review tags are annotated tag objects
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
crates/seacad-dxf-core/src/lib.rs
crates/seacad-dxf-core/src/point_clone_draft_projection.rs
crates/seacad-dxf-core/src/point_clone_xdata_draft.rs
crates/seacad-dxf-core/src/point_clone_xdata_insert.rs
crates/seacad-dxf-core/src/text_decoder.rs
crates/seacad-dxf-core/src/text_encoder.rs
crates/seacad-dxf-core/src/text_transcode.rs
crates/seacad-dxf-core/tests/entity_draft_record_tests.rs
crates/seacad-dxf-core/tests/entity_completion_tests.rs
crates/seacad-dxf-core/tests/entity_xdata_draft_record_tests.rs
crates/seacad-dxf-core/tests/entity_xdata_encoded_destination_tests.rs
crates/seacad-dxf-core/tests/entity_xdata_text_transcode_tests.rs
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
docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md
docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md
```

Compare paths as sets.

## Artifact receipts

| Path | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 181 | `62791807c53d2b5eb2f7de38f3214a45972fa7954690cafa551e6cf71d2d9309` |
| `README.vi.md` | 179 | `4ce248708347132d7f8ac10ca423071ff01b89da517eda490ec866d64ea22e4f` |
| `crates/seacad-dxf-core/src/encoding.rs` | 788 | `246e8f228588680d408dd7ae71f37e16094ab687c3c0fe55d8a48e4aef75a9a7` |
| `crates/seacad-dxf-core/src/entity_completion.rs` | 100 | `9611cc56db530b71c662479e19dcc3234fe3fafecc500165c462156a80bee5d8` |
| `crates/seacad-dxf-core/src/entity_edit_session.rs` | 2,972 | `ecbc34c75c2c6c03467ea66448f55ddddaaa5550cf087b528239bd214e890207` |
| `crates/seacad-dxf-core/src/entity_xdata_encoded_destination.rs` | 680 | `c4363b3f36799a4cf1b05b0dde28292169a92b5bbca12b0832135a432386d97b` |
| `crates/seacad-dxf-core/src/entity_xdata_draft_write.rs` | 206 | `bbcf63d4f01a6f0d6baa3eae84bd92f71cd21371e36b9bd6b69cc8e9b80691ee` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,259 | `8ec3456b36fa2b8a2f2f71932b21d7baa07e2c5d72572dc9e179963933346b00` |
| `crates/seacad-dxf-core/src/point_clone_draft_projection.rs` | 511 | `9809d74f63e8b5510689e650b95dc45baaea5073ba4d93247271fc41be74a822` |
| `crates/seacad-dxf-core/src/point_clone_xdata_draft.rs` | 250 | `0a34de42328039c76f8ac45226da6e6a7ae027831bec4b3088ba3d8c3295e77e` |
| `crates/seacad-dxf-core/src/point_clone_xdata_insert.rs` | 317 | `59c087ff6cc447604379588eba154d013e67712d9dcc881a0b6708dadf2bae6d` |
| `crates/seacad-dxf-core/src/text_decoder.rs` | 468 | `d38aa65593bd980581de7e6da81ffcd93ce780f7dfc98f49c61e49e691ffd9bb` |
| `crates/seacad-dxf-core/src/text_encoder.rs` | 156 | `914fac5c7237a60ccc418cecfec8104fc759725f31a9fa6874a90665ece89d88` |
| `crates/seacad-dxf-core/src/text_transcode.rs` | 330 | `0bc7bfedcadabd98bc882baf4cb234c5775495622d7f183a3690a51961d24a5d` |
| `crates/seacad-dxf-core/tests/entity_draft_record_tests.rs` | 1,924 | `2d053ec4c981df8ad0b13aade304e4ca2c41d59f7d977ab3a5280cab5ed10537` |
| `crates/seacad-dxf-core/tests/entity_completion_tests.rs` | 101 | `2dafe50d9587a85f8a4055522930e849ce8167a640acde4bba69ce815d40a3bb` |
| `crates/seacad-dxf-core/tests/entity_xdata_draft_record_tests.rs` | 1,630 | `84ad2245a954fbf221439abf26d9863c73b33b6937b555e31e161a612867eb08` |
| `crates/seacad-dxf-core/tests/entity_xdata_encoded_destination_tests.rs` | 1,234 | `e6450275eb747cf8fc2d4caf9546738fdd1c711b93f15399644409fdc7f5e619` |
| `crates/seacad-dxf-core/tests/entity_xdata_text_transcode_tests.rs` | 627 | `70e5d77b74566aab2cc70e4ef5b40d3fe3d522f2a880b1b160a0563575180485` |
| `crates/seacad-dxf-core/tests/text_transcode_tests.rs` | 476 | `9bda1ee19362b058ab9a174705cc0d5893f73d13bdb8e008b6e4a7d2d2340a40` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `89379b582ac385f8feb3a813ecc0440c14cd21a71fe94495a0a4c66a9931f0a9` |
| `docs/SUPPORT_MATRIX.md` | 2,939 | `1bff10c12e4470c967105f0e23b66a787f06d29cb7af56e2f9b7d5a694a4f483` |
| `docs/audits/M14_3DR_ENTITY_XDATA_DRAFT_WRITE.md` | 55 | `0475c6d1c6f73507f1c63e9da586c7078e39c25c91f0fd1b2a13848af264d540` |
| `docs/audits/M14_3DS_POINT_CLONE_DRAFT_PROJECTION.md` | 50 | `0ab9ac7a5274f38ad9cb8309a53c3b9f04f1d79512c7159f8434c7b6d4f911cc` |
| `docs/audits/M14_3DT_POINT_CLONE_XDATA_DRAFT.md` | 46 | `c9ff7377bfe2c674a0eddcb029e32f34a4c6e4b488adf76bef638e0788e04905` |
| `docs/audits/M14_3DU_POINT_CLONE_XDATA_INSERT_WRITE.md` | 44 | `0275c2672e26db5d5e5bd0031c615dea733df10005e1a12755cfc53638e31c08` |
| `docs/audits/M14_3DV_POINT_CLONE_LEGACY_ADAPTATION.md` | 41 | `d1489d15a457ee5c518c8c8d4a6e83a9b6e12a091d889c4b543d0400cc858be4` |
| `docs/audits/M14_3DW_TEXT_TRANSCODE.md` | 42 | `dcfa1d5099f1d1fe620936e930008db7d6d38d3b6f8627e4be06014542820726` |
| `docs/audits/M14_3DX_XDATA_TEXT_TRANSCODE.md` | 48 | `0c357517d7337856a41ea408e61c98645e610bade3352b67c3fc0e4b29e56f65` |
| `docs/audits/M14_3DY_POINT_COLOR_NAME_TRANSCODE.md` | 47 | `83c9cf0bc099af593c14f6899c8a9211990f3c98db2c364ee373b42d77e9db42` |
| `docs/audits/M14_3DZ_POINT_COMPLETION_LEDGER.md` | 54 | `a5e56f3813ceb079aa699e1537a8d72eb5364a3bdff5f5abaa443947c9f71308` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 1,866 | `c6c372d76bd202e3023437273edfa71ebf19a8ee42f0d44c9b8c8fad85e74e3e` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,297 | `fba52cb27dd49bda66e80d72b152f52cbe9aa236351bdc1987457da9b1353d8c` |

Compute every line count and lowercase SHA-256 and require exact matches. The
active batch note intentionally omits its self-referential receipt.

## Focused and contract checks

Run separately:

```powershell
cargo +1.97.1 test -p seacad-dxf-core --test entity_completion_tests
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
rg -n 'DxfEntityCompletion(Level|Blocker|Assessment)|DXF_ENTITY_COMPLETION_ASSESSMENTS|dxf_entity_completion_assessment|VerifiedMutation|ReleaseQualified|PrivateCorpusQualification|CurrentCheckpointSixNativeCi' crates/seacad-dxf-core/src/entity_completion.rs crates/seacad-dxf-core/tests/entity_completion_tests.rs crates/seacad-dxf-core/src/lib.rs docs/audits/M14_3DZ_POINT_COMPLETION_LEDGER.md
```

Expected 6/6, 11/11, 13/13, 18/18, 4/4, 3/3, 5/5, and 10/10. For M14.3dr prove create-new-only behavior,
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
is not complete, has unique deterministic compact metadata, and leaves the
other 44 public topics unaudited. Confirm the audit maps every satisfied level
to existing evidence and does not treat rendering, POINT display behavior,
application-specific XDATA interpretation, or automatic symbol creation as an
entity-completion blocker. Require zero production matches:

```powershell
rg -n 'panic!|unwrap\(|expect\(|todo!|unimplemented!|unsafe' crates/seacad-dxf-core/src/entity_completion.rs crates/seacad-dxf-core/src/entity_xdata_draft_write.rs crates/seacad-dxf-core/src/entity_xdata_encoded_destination.rs crates/seacad-dxf-core/src/point_clone_draft_projection.rs crates/seacad-dxf-core/src/point_clone_xdata_draft.rs crates/seacad-dxf-core/src/point_clone_xdata_insert.rs crates/seacad-dxf-core/src/entity_edit_session.rs crates/seacad-dxf-core/src/encoding.rs crates/seacad-dxf-core/src/text_decoder.rs crates/seacad-dxf-core/src/text_encoder.rs crates/seacad-dxf-core/src/text_transcode.rs
```

Validate every local Markdown link in all changed overview/plan/audit files.
Require empty protected diff and clean whitespace:

```powershell
git diff 028d726b7f04709927e0d6c8c8d419dc1eb49fcb..m14.3dz-point-completion-ledger -- Cargo.toml Cargo.lock LICENSE NOTICE THIRD_PARTY_NOTICES.md schema corpus release .github .agents
git diff --check 028d726b7f04709927e0d6c8c8d419dc1eb49fcb..m14.3dz-point-completion-ledger
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
if ($listed.Count -ne 1079) { throw 'Workspace test count drift.' }
git diff --check 028d726b7f04709927e0d6c8c8d419dc1eb49fcb..m14.3dz-point-completion-ledger
```

Repeat preflight, receipts, protected diff, and whitespace. HEAD/tag/path sets/
hashes must match and worktree remain clean. PASS requires exact state,
receipts, 6/6, 11/11, 13/13, 18/18, 4/4, 3/3, 5/5, 10/10, all contract/safety/
link/protected checks, all gates, exactly 1,079 tests, no mutation, and the
external report.

Write UTF-8 YAML-shaped evidence with batch id
`seacad-m14.3dr-m14.3dz-point-completion-ledger-2026-08-09`, status/root/
report/timestamps, HEAD/tags/Git/path sets before/after, every command and hash
receipt, focused tests, workspace total, links, forbidden scan, protected
surfaces, mutations, deviations, failures, blocker, and final assessment. After
writing, print only the report path and status.
