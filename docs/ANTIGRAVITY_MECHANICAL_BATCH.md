# Antigravity Mechanical Verification Batch

Status: READY

Checkpoint: `M14.3dr-M14.3dt-point-clone-xdata-draft`
Repository root: `D:\SeaCad\SeaCad`
Baseline: `028d726b7f04709927e0d6c8c8d419dc1eb49fcb`
Review target: annotated tag `m14.3dt-point-clone-xdata-draft`
Report: `D:\SeaCad\AntigravityReports\seacad-m14.3dr-m14.3dt-point-clone-xdata-draft-2026-08-08.yaml`
Prepared: `2026-08-08` (`Asia/Saigon`)

The preceding M14.3dl-M14.3dq cumulative batch was independently reviewed PASS
at exact baseline HEAD. Its external report SHA-256 is
`b27b982faee5ad86a47ca87346b4ac383a210097596ab80c9ed9c41ceec8e665`.
Do not repeat that retired batch. This READY batch intentionally accumulates
M14.3dr through M14.3dt so implementation does not pause between checkpoints.

## Authority, procedure, and writes

Mechanically verify M14.3dr-M14.3dt only. Codex/user retain architecture,
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
HEAD == m14.3dt-point-clone-xdata-draft^{}
m14.3dt-point-clone-xdata-draft^1 == m14.3ds-point-clone-draft-projection^{}
m14.3ds-point-clone-draft-projection^1 == m14.3dr-entity-xdata-draft-write^{}
m14.3dr-entity-xdata-draft-write^1 == 028d726b7f04709927e0d6c8c8d419dc1eb49fcb
m14.3dq-entity-xdata-draft-verification^{} == 028d726b7f04709927e0d6c8c8d419dc1eb49fcb
all three review tags are annotated tag objects
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
crates/seacad-dxf-core/tests/entity_draft_record_tests.rs
crates/seacad-dxf-core/tests/entity_xdata_draft_record_tests.rs
docs/ANTIGRAVITY_MECHANICAL_BATCH.md
docs/IMPLEMENTATION_PLAN.md
docs/SUPPORT_MATRIX.md
docs/audits/M14_3DR_ENTITY_XDATA_DRAFT_WRITE.md
docs/audits/M14_3DS_POINT_CLONE_DRAFT_PROJECTION.md
docs/audits/M14_3DT_POINT_CLONE_XDATA_DRAFT.md
docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md
docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md
```

Compare paths as sets.

## Artifact receipts

| Path | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 165 | `dd159604e95e9484e74ddb73baeb73e9ce893b5e6548d7cb961eddefe4d31b4e` |
| `README.vi.md` | 163 | `7419bf5c572aefd60c5b7a3ee3074be1f625bb8b35bff3704b7aed81812817af` |
| `crates/seacad-dxf-core/src/entity_edit_session.rs` | 2,945 | `6ce7cb8723d7f783b6e8f6320b5b1e0351bf649998670c4c54245b2fcb7fb839` |
| `crates/seacad-dxf-core/src/entity_xdata_draft_write.rs` | 206 | `bbcf63d4f01a6f0d6baa3eae84bd92f71cd21371e36b9bd6b69cc8e9b80691ee` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,244 | `bac12fd07d99302499bcff2912cb09855bdcb6e0d534c68989ddcd6d4d50847a` |
| `crates/seacad-dxf-core/src/point_clone_draft_projection.rs` | 313 | `709b344efa9a6928dbb1ac272531ae61f5e523ca7ce5894da9bcaf0ac5415adb` |
| `crates/seacad-dxf-core/src/point_clone_xdata_draft.rs` | 228 | `f315128586accacb1b3e5675c8ccec1e0b93da2552a9b12bc0a7e1fd73920f61` |
| `crates/seacad-dxf-core/tests/entity_draft_record_tests.rs` | 1,651 | `ae5223ff018ff1daf55822b83b812d5470527bd41aea18a4c3b7272eda7ee1da` |
| `crates/seacad-dxf-core/tests/entity_xdata_draft_record_tests.rs` | 1,263 | `15a98ae3cf71e90eefc980a0ee151bd728efaae9064137cca8e992bddfc220f3` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `5b0705ca103ddf8302701399f00e00ff92b1992bfa82999e20417d97a9563a90` |
| `docs/SUPPORT_MATRIX.md` | 2,847 | `53ce9dd80cdc856a52463ae602672efd853c179207b51c7782787eea8b9c47ab` |
| `docs/audits/M14_3DR_ENTITY_XDATA_DRAFT_WRITE.md` | 55 | `0475c6d1c6f73507f1c63e9da586c7078e39c25c91f0fd1b2a13848af264d540` |
| `docs/audits/M14_3DS_POINT_CLONE_DRAFT_PROJECTION.md` | 50 | `0ab9ac7a5274f38ad9cb8309a53c3b9f04f1d79512c7159f8434c7b6d4f911cc` |
| `docs/audits/M14_3DT_POINT_CLONE_XDATA_DRAFT.md` | 46 | `c9ff7377bfe2c674a0eddcb029e32f34a4c6e4b488adf76bef638e0788e04905` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 1,783 | `bd7a03e2dd6d62df79c4edeeea9fde589057f588789474467bf75040ecefe6ca` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,211 | `0996776d4c95249b31bcae3b9afe63280d0b5b6b142313af94bea937e33e3389` |

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
redaction. Require zero production matches:

```powershell
rg -n 'panic!|unwrap\(|expect\(|todo!|unimplemented!|unsafe' crates/seacad-dxf-core/src/entity_xdata_draft_write.rs crates/seacad-dxf-core/src/point_clone_draft_projection.rs crates/seacad-dxf-core/src/point_clone_xdata_draft.rs crates/seacad-dxf-core/src/entity_edit_session.rs
```

Validate every local Markdown link in all changed overview/plan/audit files.
Require empty protected diff and clean whitespace:

```powershell
git diff 028d726b7f04709927e0d6c8c8d419dc1eb49fcb..m14.3dt-point-clone-xdata-draft -- Cargo.toml Cargo.lock LICENSE NOTICE THIRD_PARTY_NOTICES.md schema corpus release .github .agents
git diff --check 028d726b7f04709927e0d6c8c8d419dc1eb49fcb..m14.3dt-point-clone-xdata-draft
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
git diff --check 028d726b7f04709927e0d6c8c8d419dc1eb49fcb..m14.3dt-point-clone-xdata-draft
```

Repeat preflight, receipts, protected diff, and whitespace. HEAD/tag/path sets/
hashes must match and worktree remain clean. PASS requires exact state,
receipts, 9/9, 11/11, 18/18, all contract/safety/link/protected checks, all
gates, exactly 1,057 tests, no mutation, and the external report.

Write UTF-8 YAML-shaped evidence with batch id
`seacad-m14.3dr-m14.3dt-point-clone-xdata-draft-2026-08-08`, status/root/
report/timestamps, HEAD/tags/Git/path sets before/after, every command and hash
receipt, focused tests, workspace total, links, forbidden scan, protected
surfaces, mutations, deviations, failures, blocker, and final assessment. After
writing, print only the report path and status.
