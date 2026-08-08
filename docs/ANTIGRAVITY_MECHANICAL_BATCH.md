# Antigravity Mechanical Verification Batch

Status: READY

Checkpoint: `M14.3dr-M14.3dv-point-clone-legacy-adaptation`
Repository root: `D:\SeaCad\SeaCad`
Baseline: `028d726b7f04709927e0d6c8c8d419dc1eb49fcb`
Review target: annotated tag `m14.3dv-point-clone-legacy-adaptation`
Report: `D:\SeaCad\AntigravityReports\seacad-m14.3dr-m14.3dv-point-clone-legacy-adaptation-2026-08-08.yaml`
Prepared: `2026-08-08` (`Asia/Saigon`)

The preceding M14.3dl-M14.3dq cumulative batch was independently reviewed PASS
at exact baseline HEAD. Its external report SHA-256 is
`b27b982faee5ad86a47ca87346b4ac383a210097596ab80c9ed9c41ceec8e665`.
Do not repeat that retired batch. This READY batch intentionally accumulates
M14.3dr through M14.3dv so implementation does not pause between checkpoints.

## Authority, procedure, and writes

Mechanically verify M14.3dr-M14.3dv only. Codex/user retain architecture,
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
HEAD == m14.3dv-point-clone-legacy-adaptation^{}
m14.3dv-point-clone-legacy-adaptation^1 == m14.3du-point-clone-xdata-insert-write^{}
m14.3du-point-clone-xdata-insert-write^1 == m14.3dt-point-clone-xdata-draft^{}
m14.3dt-point-clone-xdata-draft^1 == m14.3ds-point-clone-draft-projection^{}
m14.3ds-point-clone-draft-projection^1 == m14.3dr-entity-xdata-draft-write^{}
m14.3dr-entity-xdata-draft-write^1 == 028d726b7f04709927e0d6c8c8d419dc1eb49fcb
m14.3dq-entity-xdata-draft-verification^{} == 028d726b7f04709927e0d6c8c8d419dc1eb49fcb
all five review tags are annotated tag objects
```

The baseline-to-target changed path set must be exactly:

```text
README.md
README.vi.md
crates/seacad-dxf-core/src/entity_edit_session.rs
crates/seacad-dxf-core/src/entity_xdata_draft_write.rs
crates/seacad-dxf-core/src/lib.rs
crates/seacad-dxf-core/src/point_clone_draft_projection.rs
crates/seacad-dxf-core/src/point_clone_xdata_draft.rs
crates/seacad-dxf-core/src/point_clone_xdata_insert.rs
crates/seacad-dxf-core/tests/entity_draft_record_tests.rs
crates/seacad-dxf-core/tests/entity_xdata_draft_record_tests.rs
docs/ANTIGRAVITY_MECHANICAL_BATCH.md
docs/IMPLEMENTATION_PLAN.md
docs/SUPPORT_MATRIX.md
docs/audits/M14_3DR_ENTITY_XDATA_DRAFT_WRITE.md
docs/audits/M14_3DS_POINT_CLONE_DRAFT_PROJECTION.md
docs/audits/M14_3DT_POINT_CLONE_XDATA_DRAFT.md
docs/audits/M14_3DU_POINT_CLONE_XDATA_INSERT_WRITE.md
docs/audits/M14_3DV_POINT_CLONE_LEGACY_ADAPTATION.md
docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md
docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md
```

Compare paths as sets.

## Artifact receipts

| Path | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 169 | `877cd0c630335c84ddbc8588751de268b560303e719c031d11c16fcaa24f7b1b` |
| `README.vi.md` | 167 | `01ce85deeec56b0c9ee60fa939a4fad2800efea7217fdc650a08f085c06da074` |
| `crates/seacad-dxf-core/src/entity_edit_session.rs` | 2,950 | `c5be6e2295a71efb638b53d3770d0f232794503846be4c4d5c13d51947f724b2` |
| `crates/seacad-dxf-core/src/entity_xdata_draft_write.rs` | 206 | `bbcf63d4f01a6f0d6baa3eae84bd92f71cd21371e36b9bd6b69cc8e9b80691ee` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,250 | `bdf7d4ef07be39641647d5c20b28fb3b1a6bda4ae756bdb668eb27b0d7ef7ec3` |
| `crates/seacad-dxf-core/src/point_clone_draft_projection.rs` | 410 | `b43d0e74ab79a642e9b64f18e9366427a1f2fbfa2b3bbd8aa8dd192fc061e7c3` |
| `crates/seacad-dxf-core/src/point_clone_xdata_draft.rs` | 237 | `b4732a028915471b7a019e0fedd2e9b36213b6939a6336dcc8be62247ae7c815` |
| `crates/seacad-dxf-core/src/point_clone_xdata_insert.rs` | 287 | `250dcab2c1765f173a325e113d4d497f5587ffe5c8d719d13ecf7f6d3585ee43` |
| `crates/seacad-dxf-core/tests/entity_draft_record_tests.rs` | 1,693 | `d9f200daecb7d26988ad6b72bb2e6c33640def3725c2838251ac07db3cc5a091` |
| `crates/seacad-dxf-core/tests/entity_xdata_draft_record_tests.rs` | 1,430 | `9fce78097f52b069bc9aa58fd0b38a089981bbe5d722c79af9c34a627c3e66f4` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `10807cc994b93d7512806a4354416bfd9969217001cd90dd338c2c816a25d0cd` |
| `docs/SUPPORT_MATRIX.md` | 2,872 | `30be0807ed5d8e6507f84b52da3c7bf3a7ac08639efe520c1a9a8d6ea232c5ce` |
| `docs/audits/M14_3DR_ENTITY_XDATA_DRAFT_WRITE.md` | 55 | `0475c6d1c6f73507f1c63e9da586c7078e39c25c91f0fd1b2a13848af264d540` |
| `docs/audits/M14_3DS_POINT_CLONE_DRAFT_PROJECTION.md` | 50 | `0ab9ac7a5274f38ad9cb8309a53c3b9f04f1d79512c7159f8434c7b6d4f911cc` |
| `docs/audits/M14_3DT_POINT_CLONE_XDATA_DRAFT.md` | 46 | `c9ff7377bfe2c674a0eddcb029e32f34a4c6e4b488adf76bef638e0788e04905` |
| `docs/audits/M14_3DU_POINT_CLONE_XDATA_INSERT_WRITE.md` | 44 | `0275c2672e26db5d5e5bd0031c615dea733df10005e1a12755cfc53638e31c08` |
| `docs/audits/M14_3DV_POINT_CLONE_LEGACY_ADAPTATION.md` | 41 | `d1489d15a457ee5c518c8c8d4a6e83a9b6e12a091d889c4b543d0400cc858be4` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 1,806 | `a6027393403e5bbd602880e24eb77e8fb9357037124838eb406e4cf3e971b238` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,236 | `f7a11b9338501d14008f73ea63373d5f9913c3c54d44a9a06ee311faca853e13` |

Compute every line count and lowercase SHA-256 and require exact matches. The
active batch note intentionally omits its self-referential receipt.

## Focused and contract checks

Run separately:

```powershell
cargo +1.97.1 test -p seacad-dxf-core --test entity_xdata_draft_record_tests
cargo +1.97.1 test -p seacad-dxf-core --test entity_draft_record_tests
cargo +1.97.1 test -p seacad-dxf-core --test entity_insert_session_tests
rg -n 'DxfEntityXDataDraftWrite|write_reparse_verify_and_journal_to_new_file|existing|cancelled|tampered' crates/seacad-dxf-core/src/entity_xdata_draft_write.rs crates/seacad-dxf-core/tests/entity_xdata_draft_record_tests.rs crates/seacad-dxf-core/src/lib.rs
rg -n 'DxfPointCloneDestinationBindings|DxfPointCloneDraftProjectionIssue|DxfPointCloneDraftProjectionPlan|project_point_clone_draft_from|prepare_point_clone_snapshot|borrowed_for_destination' crates/seacad-dxf-core/src/point_clone_draft_projection.rs crates/seacad-dxf-core/src/entity_edit_session.rs crates/seacad-dxf-core/tests/entity_draft_record_tests.rs crates/seacad-dxf-core/src/lib.rs
rg -n 'DxfPointCloneXDataSource|DxfPointCloneXDataDraftIssue|DxfPointCloneXDataDraftPlan|project_point_clone_xdata_draft_from|with_xdata_composition|EncodedPayloadUnavailable|standalone' crates/seacad-dxf-core/src/point_clone_xdata_draft.rs crates/seacad-dxf-core/src/point_clone_draft_projection.rs crates/seacad-dxf-core/tests/entity_xdata_draft_record_tests.rs crates/seacad-dxf-core/src/lib.rs
rg -n 'DxfPointCloneSourceEvidence|DxfPointCloneXDataInsertPlan|DxfPointCloneXDataVerificationJournal|DxfPointCloneXDataWriteJournal|plan_point_clone_xdata_insert|write_reparse_verify_and_journal_to_new_file|existing|tampered' crates/seacad-dxf-core/src/point_clone_xdata_insert.rs crates/seacad-dxf-core/tests/entity_xdata_draft_record_tests.rs crates/seacad-dxf-core/src/lib.rs
rg -n 'DxfPointCloneDialectAdaptations|legacy_placement_owns_layout|omitted_by_layer_lineweight|DestinationFieldNotRepresentable|LINEWEIGHT|Ac1032|Ac1009' crates/seacad-dxf-core/src/point_clone_draft_projection.rs crates/seacad-dxf-core/src/point_clone_xdata_draft.rs crates/seacad-dxf-core/src/point_clone_xdata_insert.rs crates/seacad-dxf-core/tests/entity_draft_record_tests.rs crates/seacad-dxf-core/tests/entity_xdata_draft_record_tests.rs crates/seacad-dxf-core/src/lib.rs
```

Expected 9/9, 11/11, and 18/18. For M14.3dr prove create-new-only behavior,
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
non-representable non-default lineweight rejection. Require zero production
matches:

```powershell
rg -n 'panic!|unwrap\(|expect\(|todo!|unimplemented!|unsafe' crates/seacad-dxf-core/src/entity_xdata_draft_write.rs crates/seacad-dxf-core/src/point_clone_draft_projection.rs crates/seacad-dxf-core/src/point_clone_xdata_draft.rs crates/seacad-dxf-core/src/point_clone_xdata_insert.rs crates/seacad-dxf-core/src/entity_edit_session.rs
```

Validate every local Markdown link in all changed overview/plan/audit files.
Require empty protected diff and clean whitespace:

```powershell
git diff 028d726b7f04709927e0d6c8c8d419dc1eb49fcb..m14.3dv-point-clone-legacy-adaptation -- Cargo.toml Cargo.lock LICENSE NOTICE THIRD_PARTY_NOTICES.md schema corpus release .github .agents
git diff --check 028d726b7f04709927e0d6c8c8d419dc1eb49fcb..m14.3dv-point-clone-legacy-adaptation
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
if ($listed.Count -ne 1057) { throw 'Workspace test count drift.' }
git diff --check 028d726b7f04709927e0d6c8c8d419dc1eb49fcb..m14.3dv-point-clone-legacy-adaptation
```

Repeat preflight, receipts, protected diff, and whitespace. HEAD/tag/path sets/
hashes must match and worktree remain clean. PASS requires exact state,
receipts, 9/9, 11/11, 18/18, all contract/safety/link/protected checks, all
gates, exactly 1,057 tests, no mutation, and the external report.

Write UTF-8 YAML-shaped evidence with batch id
`seacad-m14.3dr-m14.3dv-point-clone-legacy-adaptation-2026-08-08`, status/root/
report/timestamps, HEAD/tags/Git/path sets before/after, every command and hash
receipt, focused tests, workspace total, links, forbidden scan, protected
surfaces, mutations, deviations, failures, blocker, and final assessment. After
writing, print only the report path and status.
