# Antigravity Mechanical Verification Batch

Status: READY

Checkpoint: `M14.3dl..M14.3dm-entity-xdata-encoded-applications`
Repository root: `D:\SeaCad\SeaCad`
Baseline: `c502d1a6c1f709c9fdc85e9e1fa8bc58d6d43a14`
Review target: annotated tag `m14.3dm-entity-xdata-encoded-application-destination`
Report: `D:\SeaCad\AntigravityReports\seacad-m14.3dl-m14.3dm-entity-xdata-encoded-applications-2026-08-08.yaml`
Prepared: `2026-08-08` (`Asia/Saigon`)

## Authority, procedure, and writes

Mechanically verify the bounded cumulative M14.3dl and M14.3dm checkpoints
only. Codex/user retain architecture, support, license, commit/tag/merge/release
decisions. Use the exact root, read `AGENTS.md` and this note completely,
confirm exactly one READY, and run phases in order. The repository is read-only;
ignored target/cache activity is tolerated. Write only the external report. No
edits, Git mutation, installs, network/vendor CAD, legacy/private corpus,
license/export/archive, or fixture creation. A root, baseline, target-tag,
path, or receipt mismatch is BLOCKED.

## Exact preflight state

Require a clean tracked and untracked worktree. Record `Get-Location`, Git
status, HEAD, target tag object/type/message/peeled commit, both checkpoint tag
targets, and the exact baseline-to-target changed path set. Require:

```text
HEAD == m14.3dm-entity-xdata-encoded-application-destination^{}
m14.3dl-entity-xdata-encoded-destination^{} == 76a9179796178c2c683165fc04844b6448459df1
baseline is an ancestor of M14.3dl, which is an ancestor of M14.3dm
both checkpoint tags are annotated tag objects
```

The cumulative changed path set must be exactly:

```text
README.md
README.vi.md
crates/seacad-dxf-core/src/entity_xdata_encoded_application_destination.rs
crates/seacad-dxf-core/src/entity_xdata_encoded_destination.rs
crates/seacad-dxf-core/src/lib.rs
crates/seacad-dxf-core/tests/entity_xdata_encoded_destination_tests.rs
docs/ANTIGRAVITY_MECHANICAL_BATCH.md
docs/IMPLEMENTATION_PLAN.md
docs/SUPPORT_MATRIX.md
docs/audits/M14_3DL_ENTITY_XDATA_ENCODED_DESTINATION.md
docs/audits/M14_3DM_ENTITY_XDATA_ENCODED_APPLICATION_DESTINATION.md
docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md
docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md
```

Compare paths as sets.

## Artifact receipts

| Path | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 157 | `d00841242127c76acadff6ede3087c6dc5b13f6e9c5dd0aef58e2952a63d32d2` |
| `README.vi.md` | 156 | `ce1f1dca97ace9c1568903f2fc1320b655a7911b7456e5184265b4d9602111e2` |
| `crates/seacad-dxf-core/src/entity_xdata_encoded_application_destination.rs` | 462 | `9b8d1dce1370fa2369c1c97296c3e8aa7e3f67d16a96b9ec1dbbf083f454784b` |
| `crates/seacad-dxf-core/src/entity_xdata_encoded_destination.rs` | 580 | `011b9f8771af6f62a652094283787990f48bbf32e248743a3a5d24ddaa4c9e33` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,215 | `7237a8af75a285a419dcac3887e6d4e62f84027ab816f2967385a4baef6cc55f` |
| `crates/seacad-dxf-core/tests/entity_xdata_encoded_destination_tests.rs` | 972 | `530a01c40da583606d4b35bba90751a37e78f1feb7b24b7269999889454dc920` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `43fb2d233bb694698f574d75f44cae1fb941c5f0f76a0ca736a01629e5856736` |
| `docs/SUPPORT_MATRIX.md` | 2,749 | `5e3b575cc1370a9b67f4c7d34c9bdd805c2d36088945f6d1b55e1ff883b5f4e6` |
| `docs/audits/M14_3DL_ENTITY_XDATA_ENCODED_DESTINATION.md` | 59 | `b15602b92dde03f08e8c5a07925fa9c3fa6aa0190b258a6366dc0e39d294115a` |
| `docs/audits/M14_3DM_ENTITY_XDATA_ENCODED_APPLICATION_DESTINATION.md` | 59 | `16ea8a958a007412e9489744dca579f36f4cbb95e56df4c9ab14edcc1ca26739` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 1,691 | `44942461b0c11f3af95dc5f1fc65c055419389a0b8cde36a0c248286049faf52` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,113 | `959ecc6195a3b2da0f5ddb0ba8debae50e1575eae8ed16048cfa0e6e92a4c766` |

Compute every line count and lowercase SHA-256 and require exact matches. The
active batch note intentionally omits its own self-referential receipt.

## Focused and contract checks

Run separately:

```powershell
cargo +1.97.1 test -p seacad-dxf-core --test entity_xdata_encoded_destination_tests
cargo +1.97.1 test -p seacad-dxf-core --test entity_value_encoder_tests --test entity_xdata_handle_replacement_tests --test entity_xdata_logical_destination_tests --test entity_xdata_encoded_destination_tests
rg -n 'DxfEntityXDataEncodedDestination|DxfEntityXDataEncodedApplicationDestination|entity_xdata_encoded_application_destination_directory|encoded_bytes_for_ready_range|first_unavailable_member_ordinal|payload_for_entry|encoded_entries_for_entry|encoded_bytes_for_entry' crates/seacad-dxf-core/src/entity_xdata_encoded_destination.rs crates/seacad-dxf-core/src/entity_xdata_encoded_application_destination.rs crates/seacad-dxf-core/src/lib.rs
rg -n 'encoded_application|DxfAcadVersion::SUPPORTED|Ascii|Binary|Ac1009|Ac1032|TextTranscodingRequired|APP_EMPTY|APP_NESTED|ORPHAN_SECRET|Cancelled|foreign|non_disclosing' crates/seacad-dxf-core/tests/entity_xdata_encoded_destination_tests.rs
```

Expected 7/7 and 20/20. Reconfirm the M14.3dl one-state-per-logical-occurrence,
canonical ASCII/Binary framing, AC1009 escape, bounded provenance, transcoding,
and unavailable-byte contracts. For M14.3dm prove one set per exact source
application; source order and member order preserved; no orphan assigned; ready
requires M14.3dj payload ready and every M14.3dl member ready; unavailable state
retains payload state, member counts, and first unavailable ordinal; aggregate
bytes are exposed only for wholly ready sets. Require all format/dialect pairs,
cross-dialect boundaries, empty/nested applications, source-exact/transformed/
handle cases, cancellation, identity, bounds, and redaction.

Require zero production matches:

```powershell
rg -n 'panic!|unwrap\(|expect\(|todo!|unimplemented!|unsafe' crates/seacad-dxf-core/src/entity_xdata_encoded_destination.rs crates/seacad-dxf-core/src/entity_xdata_encoded_application_destination.rs
```

Validate every local Markdown link in the changed overview/plan/audit files.
Require an empty protected diff and clean whitespace:

```powershell
rg -n 'M14\.3d[klm]|EncodedApplication|encoded application|per-application|nhóm encoded' README.md README.vi.md docs/IMPLEMENTATION_PLAN.md docs/SUPPORT_MATRIX.md docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md docs/audits/M14_3DL_ENTITY_XDATA_ENCODED_DESTINATION.md docs/audits/M14_3DM_ENTITY_XDATA_ENCODED_APPLICATION_DESTINATION.md
git diff c502d1a6c1f709c9fdc85e9e1fa8bc58d6d43a14..m14.3dm-entity-xdata-encoded-application-destination -- Cargo.toml Cargo.lock LICENSE NOTICE THIRD_PARTY_NOTICES.md schema corpus release .github .agents
git diff --check c502d1a6c1f709c9fdc85e9e1fa8bc58d6d43a14..m14.3dm-entity-xdata-encoded-application-destination
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
if ($listed.Count -ne 1041) { throw 'Workspace test count drift.' }
git diff --check c502d1a6c1f709c9fdc85e9e1fa8bc58d6d43a14..m14.3dm-entity-xdata-encoded-application-destination
```

Repeat preflight, receipts, protected diff, and whitespace. HEAD/tag/path sets/
hashes must match and the worktree must remain clean. PASS requires exact state,
receipts, 7/7, 20/20, all contract/safety/link/protected checks, all gates,
exactly 1,041 tests, no mutation, and the external report.

Write UTF-8 YAML-shaped evidence with batch id
`seacad-m14.3dl-m14.3dm-entity-xdata-encoded-applications-2026-08-08`, status/
root/report/timestamps, HEAD and both annotated tags, Git/path sets before/after,
every command and hash receipt, `focused_encoded_application_tests`,
`focused_adjacent_tests`, `workspace_test_total`, links, forbidden scan,
`support_matrix_unchanged_except_m14_3dl_m14_3dm`, protected surfaces,
mutations, deviations, failures, blocker, and final assessment. After writing,
print only the report path and status.
