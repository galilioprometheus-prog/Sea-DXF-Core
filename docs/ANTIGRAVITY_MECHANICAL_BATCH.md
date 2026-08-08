# Antigravity Mechanical Verification Batch

Status: READY

Checkpoint: `M14.3dr-entity-xdata-draft-write`
Repository root: `D:\SeaCad\SeaCad`
Baseline: `028d726b7f04709927e0d6c8c8d419dc1eb49fcb`
Review target: annotated tag `m14.3dr-entity-xdata-draft-write`
Report: `D:\SeaCad\AntigravityReports\seacad-m14.3dr-entity-xdata-draft-write-2026-08-08.yaml`
Prepared: `2026-08-08` (`Asia/Saigon`)

The preceding M14.3dl-M14.3dq cumulative batch was independently reviewed PASS
at exact baseline HEAD. Its external report SHA-256 is
`b27b982faee5ad86a47ca87346b4ac383a210097596ab80c9ed9c41ceec8e665`.
Do not repeat that retired batch.

## Authority, procedure, and writes

Mechanically verify M14.3dr only. Codex/user retain architecture, support,
license, commit/tag/merge/release decisions. Use the exact root, read `AGENTS.md`
and this note completely, confirm exactly one READY, and run phases in order.
The repository is read-only; ignored target/cache activity is tolerated. Write
only the external report. No edits, Git mutation, installs, network/vendor CAD,
legacy/private corpus, license/export/archive, or fixture creation. A root,
baseline, target-tag, path, or receipt mismatch is BLOCKED.

## Exact preflight state

Require a clean tracked and untracked worktree. Record `Get-Location`, Git
status, HEAD, target annotated-tag object/message/peeled commit, baseline tag
target, and baseline-to-target changed path set. Require:

```text
HEAD == m14.3dr-entity-xdata-draft-write^{}
m14.3dq-entity-xdata-draft-verification^{} == 028d726b7f04709927e0d6c8c8d419dc1eb49fcb
baseline is the direct parent of M14.3dr
M14.3dr is an annotated tag object
```

The changed path set must be exactly:

```text
README.md
README.vi.md
crates/seacad-dxf-core/src/entity_xdata_draft_write.rs
crates/seacad-dxf-core/src/lib.rs
crates/seacad-dxf-core/tests/entity_xdata_draft_record_tests.rs
docs/ANTIGRAVITY_MECHANICAL_BATCH.md
docs/IMPLEMENTATION_PLAN.md
docs/SUPPORT_MATRIX.md
docs/audits/M14_3DR_ENTITY_XDATA_DRAFT_WRITE.md
docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md
docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md
```

Compare paths as sets.

## Artifact receipts

| Path | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 160 | `925c1748f3800994fe453e7669b436d5e0a49b3044435d3fd42baf0a5afbd967` |
| `README.vi.md` | 158 | `d8f1454a238f238f014a69be60beb9cab89df49e9e7c27262fce49b9b67bda95` |
| `crates/seacad-dxf-core/src/entity_xdata_draft_write.rs` | 206 | `bbcf63d4f01a6f0d6baa3eae84bd92f71cd21371e36b9bd6b69cc8e9b80691ee` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,235 | `6470f8a7ae74f59f0c11c1371354fd0ade87a95235d3b511532c4991dddfdde3` |
| `crates/seacad-dxf-core/tests/entity_xdata_draft_record_tests.rs` | 998 | `f64a367bc754303c3105a87d85875238394aa736770b52a2d716769657d578ec` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `b438c8ea2ed3d500692d0335858288fd2583bb4c811a1c76a2d31a269853593e` |
| `docs/SUPPORT_MATRIX.md` | 2,819 | `61daa0885f4de6c70530705f2e520d90f0d4947eeddce37c8e899df0a5d4ed26` |
| `docs/audits/M14_3DR_ENTITY_XDATA_DRAFT_WRITE.md` | 55 | `0475c6d1c6f73507f1c63e9da586c7078e39c25c91f0fd1b2a13848af264d540` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 1,759 | `aefee03139d693f7c4958c1d8c2801568512e408421c63ce915d00cbc8b53d3b` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,182 | `95097ff98bf813932a3615214edb01855285f5f9b892f14465bb7cdc0349cda6` |

Compute every line count and lowercase SHA-256 and require exact matches. The
active batch note intentionally omits its self-referential receipt.

## Focused and contract checks

Run separately:

```powershell
cargo +1.97.1 test -p seacad-dxf-core --test entity_xdata_draft_record_tests
cargo +1.97.1 test -p seacad-dxf-core --test entity_value_encoder_tests --test entity_draft_record_tests --test entity_xdata_handle_replacement_tests --test entity_xdata_logical_destination_tests --test entity_xdata_encoded_destination_tests --test entity_xdata_draft_record_tests
rg -n 'DxfEntityXDataDraftWrite|write_reparse_verify_and_journal_to_new_file|write_to_new_file|reparse_written_destination|finish_written_verification|remove_created_destination|validate_written_identities|FinalTamperObserver|existing|cancelled|tampered' crates/seacad-dxf-core/src/entity_xdata_draft_write.rs crates/seacad-dxf-core/tests/entity_xdata_draft_record_tests.rs crates/seacad-dxf-core/src/lib.rs
```

Expected 7/7 and 37/37. Prove create-new-only behavior, all four format pairs,
all nine Core dialects and AC1009/AC1032 cross-dialect boundaries, non-empty
and zero-XDATA writes, exact output, strict reparse, family-plus-XDATA receipt
identity binding, inverse restoration, existing-file preservation,
pre-cancellation, final-progress tamper rejection and cleanup, bounds, traits,
and redaction. Require zero production matches:

```powershell
rg -n 'panic!|unwrap\(|expect\(|todo!|unimplemented!|unsafe' crates/seacad-dxf-core/src/entity_xdata_draft_write.rs
```

Validate every local Markdown link in the changed overview/plan/audit files.
Require empty protected diff and clean whitespace:

```powershell
rg -n 'M14\.3dr|DraftWrite|create-new|write journal|cleanup|1,051' README.md README.vi.md docs/IMPLEMENTATION_PLAN.md docs/SUPPORT_MATRIX.md docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md docs/audits/M14_3DR_ENTITY_XDATA_DRAFT_WRITE.md
git diff 028d726b7f04709927e0d6c8c8d419dc1eb49fcb..m14.3dr-entity-xdata-draft-write -- Cargo.toml Cargo.lock LICENSE NOTICE THIRD_PARTY_NOTICES.md schema corpus release .github .agents
git diff --check 028d726b7f04709927e0d6c8c8d419dc1eb49fcb..m14.3dr-entity-xdata-draft-write
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
if ($listed.Count -ne 1051) { throw 'Workspace test count drift.' }
git diff --check 028d726b7f04709927e0d6c8c8d419dc1eb49fcb..m14.3dr-entity-xdata-draft-write
```

Repeat preflight, receipts, protected diff, and whitespace. HEAD/tag/path sets/
hashes must match and worktree remain clean. PASS requires exact state,
receipts, 7/7, 37/37, all contract/safety/link/protected checks, all gates,
exactly 1,051 tests, no mutation, and the external report.

Write UTF-8 YAML-shaped evidence with batch id
`seacad-m14.3dr-entity-xdata-draft-write-2026-08-08`, status/root/report/
timestamps, HEAD/tag/Git/path sets before/after, every command and hash receipt,
focused tests, workspace total, links, forbidden scan,
`support_matrix_unchanged_except_m14_3dr`, protected surfaces, mutations,
deviations, failures, blocker, and final assessment. After writing, print only
the report path and status.
