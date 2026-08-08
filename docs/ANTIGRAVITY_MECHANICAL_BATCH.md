# Antigravity Mechanical Verification Batch

Status: READY

Checkpoint: `M14.3dl..M14.3do-entity-xdata-draft-composition`
Repository root: `D:\SeaCad\SeaCad`
Baseline: `c502d1a6c1f709c9fdc85e9e1fa8bc58d6d43a14`
Review target: annotated tag `m14.3do-entity-xdata-draft-record`
Report: `D:\SeaCad\AntigravityReports\seacad-m14.3dl-m14.3do-entity-xdata-draft-composition-2026-08-08.yaml`
Prepared: `2026-08-08` (`Asia/Saigon`)

## Authority, procedure, and writes

Mechanically verify the bounded cumulative M14.3dl through M14.3do checkpoints
only. Codex/user retain architecture, support, license, commit/tag/merge/release
decisions. Use the exact root, read `AGENTS.md` and this note completely,
confirm exactly one READY, and run phases in order. The repository is read-only;
ignored target/cache activity is tolerated. Write only the external report. No
edits, Git mutation, installs, network/vendor CAD, legacy/private corpus,
license/export/archive, or fixture creation. A root, baseline, target-tag,
path, or receipt mismatch is BLOCKED.

## Exact preflight state

Require a clean tracked and untracked worktree. Record `Get-Location`, Git
status, HEAD, target tag object/type/message/peeled commit, all checkpoint tag
targets, and the exact baseline-to-target changed path set. Require:

```text
HEAD == m14.3do-entity-xdata-draft-record^{}
m14.3dl-entity-xdata-encoded-destination^{} == 76a9179796178c2c683165fc04844b6448459df1
m14.3dm-entity-xdata-encoded-application-destination^{} == 74e33dd9024484d394877146c429ffd32067ec90
m14.3dn-entity-xdata-encoded-entity-destination^{} == 88eb6411beff577cecd2e5b1a8ba4f04dc6bc07b
baseline is an ancestor of M14.3dl, M14.3dm, M14.3dn, and M14.3do in that order
all four checkpoint tags are annotated tag objects
```

The cumulative changed path set must be exactly:

```text
README.md
README.vi.md
crates/seacad-dxf-core/src/entity_xdata_encoded_application_destination.rs
crates/seacad-dxf-core/src/entity_xdata_encoded_destination.rs
crates/seacad-dxf-core/src/entity_xdata_encoded_entity_destination.rs
crates/seacad-dxf-core/src/entity_draft_record.rs
crates/seacad-dxf-core/src/entity_xdata_draft_record.rs
crates/seacad-dxf-core/src/lib.rs
crates/seacad-dxf-core/tests/entity_xdata_encoded_destination_tests.rs
crates/seacad-dxf-core/tests/entity_xdata_draft_record_tests.rs
docs/ANTIGRAVITY_MECHANICAL_BATCH.md
docs/IMPLEMENTATION_PLAN.md
docs/SUPPORT_MATRIX.md
docs/audits/M14_3DL_ENTITY_XDATA_ENCODED_DESTINATION.md
docs/audits/M14_3DM_ENTITY_XDATA_ENCODED_APPLICATION_DESTINATION.md
docs/audits/M14_3DN_ENTITY_XDATA_ENCODED_ENTITY_DESTINATION.md
docs/audits/M14_3DO_ENTITY_XDATA_DRAFT_RECORD.md
docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md
docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md
```

Compare paths as sets.

## Artifact receipts

| Path | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 158 | `d502b50ef66930108571b45503d064dde81cd253b4f6f18becb0cf1d524adb79` |
| `README.vi.md` | 157 | `11908f60a8da8ac58ca0b90658e33c035b04969719604df0c85952bb521ce9c7` |
| `crates/seacad-dxf-core/src/entity_xdata_encoded_application_destination.rs` | 462 | `9b8d1dce1370fa2369c1c97296c3e8aa7e3f67d16a96b9ec1dbbf083f454784b` |
| `crates/seacad-dxf-core/src/entity_xdata_encoded_destination.rs` | 580 | `011b9f8771af6f62a652094283787990f48bbf32e248743a3a5d24ddaa4c9e33` |
| `crates/seacad-dxf-core/src/entity_xdata_encoded_entity_destination.rs` | 414 | `8ad238f45655582faa7a54d23d0ded371150c913d1fd3989a5c560bd4a9f9ad6` |
| `crates/seacad-dxf-core/src/entity_draft_record.rs` | 1,329 | `a9862c284937154a8d7680abc17376abf71e3dd67709ff984415bb94fadecb7a` |
| `crates/seacad-dxf-core/src/entity_xdata_draft_record.rs` | 142 | `ab56e23fd3cb7e1a524f0c4d958cff1e51f12f6cdcd8f6a44020bbdd98e9aea6` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,224 | `61728d85e1c4752015acd360633feffe10e046cc71927e749194626335e10edc` |
| `crates/seacad-dxf-core/tests/entity_xdata_encoded_destination_tests.rs` | 1,225 | `198a4f2be745da5e71ac51e543a544d3720ff82a6f94623c293d88d9d1fe3afd` |
| `crates/seacad-dxf-core/tests/entity_xdata_draft_record_tests.rs` | 517 | `8af4aebc756aa50042a1d5de9814dc6d6ca43b125dbf21a92d2d63c7bedf2960` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `90bf59a072167a7979a9a35a550af5f42b51d8af68d62bd41bdcdf7366223403` |
| `docs/SUPPORT_MATRIX.md` | 2,776 | `0d184e0bcffe5c07670e993a6341b52bc40c29af67be4e10182b0f4c5130153e` |
| `docs/audits/M14_3DL_ENTITY_XDATA_ENCODED_DESTINATION.md` | 59 | `b15602b92dde03f08e8c5a07925fa9c3fa6aa0190b258a6366dc0e39d294115a` |
| `docs/audits/M14_3DM_ENTITY_XDATA_ENCODED_APPLICATION_DESTINATION.md` | 59 | `16ea8a958a007412e9489744dca579f36f4cbb95e56df4c9ab14edcc1ca26739` |
| `docs/audits/M14_3DN_ENTITY_XDATA_ENCODED_ENTITY_DESTINATION.md` | 51 | `62a16d9e227b150754655ccf5b77b712330659a34a17f4c5a87ed80ab21a4ae0` |
| `docs/audits/M14_3DO_ENTITY_XDATA_DRAFT_RECORD.md` | 53 | `b6009122501120a53d8926814626266577db3d73f6303ebb92af9d31e6e2be5c` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 1,718 | `6cdc09ce0a495aae7191e1c5eeda15b305e5823191953158157a2ecc6eea1702` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,139 | `84091fdb9a981edb60665e8bbca65f7c5e85b0e964359102cf3386b86c4c1b25` |

Compute every line count and lowercase SHA-256 and require exact matches. The
active batch note intentionally omits its own self-referential receipt.

## Focused and contract checks

Run separately:

```powershell
cargo +1.97.1 test -p seacad-dxf-core --test entity_xdata_encoded_destination_tests
cargo +1.97.1 test -p seacad-dxf-core --test entity_xdata_draft_record_tests
cargo +1.97.1 test -p seacad-dxf-core --test entity_value_encoder_tests --test entity_draft_record_tests --test entity_xdata_handle_replacement_tests --test entity_xdata_logical_destination_tests --test entity_xdata_encoded_destination_tests --test entity_xdata_draft_record_tests
rg -n 'DxfEntityXDataEncodedDestination|DxfEntityXDataEncodedApplicationDestination|DxfEntityXDataEncodedEntityDestination|encoded_bytes_for_ready_range|first_unavailable_member_ordinal|first_unavailable_application_ordinal|payload_for_entry|applications_for_entry|encoded_bytes_for_entry' crates/seacad-dxf-core/src/entity_xdata_encoded_destination.rs crates/seacad-dxf-core/src/entity_xdata_encoded_application_destination.rs crates/seacad-dxf-core/src/entity_xdata_encoded_entity_destination.rs crates/seacad-dxf-core/src/lib.rs
rg -n 'encoded_application|encoded_entit|DxfAcadVersion::SUPPORTED|Ascii|Binary|Ac1009|Ac1032|TextTranscodingRequired|APP_EMPTY|APP_NESTED|ORPHAN_SECRET|Cancelled|foreign|non_disclosing' crates/seacad-dxf-core/tests/entity_xdata_encoded_destination_tests.rs
rg -n 'compose_entity_draft_record|DxfEntityXDataDraftRecord|append_exact_groups|ready_xdata_appends|zero_xdata|unavailable|SourceIdentityMismatch|Cancelled|SECRET_DRAFT_XDATA' crates/seacad-dxf-core/src/entity_xdata_draft_record.rs crates/seacad-dxf-core/src/entity_draft_record.rs crates/seacad-dxf-core/tests/entity_xdata_draft_record_tests.rs crates/seacad-dxf-core/src/lib.rs
```

Expected 10/10, 3/3, and 33/33. Reconfirm the M14.3dl one-state-per-logical-occurrence,
canonical ASCII/Binary framing, AC1009 escape, bounded provenance, transcoding,
and unavailable-byte contracts. For M14.3dm prove one set per exact source
application; source order and member order preserved; no orphan assigned; ready
requires M14.3dj payload ready and every M14.3dl member ready; unavailable state
retains payload state, member counts, and first unavailable ordinal; aggregate
bytes are exposed only for wholly ready sets. Require all format/dialect pairs,
cross-dialect boundaries, empty/nested applications, source-exact/transformed/
handle cases, cancellation, identity, bounds, and redaction.
For M14.3dn prove one payload per indexed source entity; zero-XDATA entries are
ready with empty bytes; ready requires M14.3dj and every M14.3dm set ready; and
failed-handle/orphan entities expose no aggregate bytes while retaining counts.
For M14.3do prove exact draft prefix and XDATA suffix composition, zero-XDATA
no-op behavior, source-entity evidence, destination identity binding, bounded
growth, unavailable-payload suppression, cancellation, and redaction.

Require zero production matches:

```powershell
rg -n 'panic!|unwrap\(|expect\(|todo!|unimplemented!|unsafe' crates/seacad-dxf-core/src/entity_xdata_encoded_destination.rs crates/seacad-dxf-core/src/entity_xdata_encoded_application_destination.rs crates/seacad-dxf-core/src/entity_xdata_encoded_entity_destination.rs
rg -n 'panic!|unwrap\(|expect\(|todo!|unimplemented!|unsafe' crates/seacad-dxf-core/src/entity_xdata_draft_record.rs
```

Validate every local Markdown link in the changed overview/plan/audit files.
Require an empty protected diff and clean whitespace:

```powershell
rg -n 'M14\.3d[klmno]|DraftRecord|EncodedApplication|EncodedEntity|per-application|per-entity|nhóm encoded|draft record' README.md README.vi.md docs/IMPLEMENTATION_PLAN.md docs/SUPPORT_MATRIX.md docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md docs/audits/M14_3DL_ENTITY_XDATA_ENCODED_DESTINATION.md docs/audits/M14_3DM_ENTITY_XDATA_ENCODED_APPLICATION_DESTINATION.md docs/audits/M14_3DN_ENTITY_XDATA_ENCODED_ENTITY_DESTINATION.md docs/audits/M14_3DO_ENTITY_XDATA_DRAFT_RECORD.md
git diff c502d1a6c1f709c9fdc85e9e1fa8bc58d6d43a14..m14.3do-entity-xdata-draft-record -- Cargo.toml Cargo.lock LICENSE NOTICE THIRD_PARTY_NOTICES.md schema corpus release .github .agents
git diff --check c502d1a6c1f709c9fdc85e9e1fa8bc58d6d43a14..m14.3do-entity-xdata-draft-record
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
if ($listed.Count -ne 1047) { throw 'Workspace test count drift.' }
git diff --check c502d1a6c1f709c9fdc85e9e1fa8bc58d6d43a14..m14.3do-entity-xdata-draft-record
```

Repeat preflight, receipts, protected diff, and whitespace. HEAD/tag/path sets/
hashes must match and the worktree must remain clean. PASS requires exact state,
receipts, 10/10, 3/3, 33/33, all contract/safety/link/protected checks, all
gates, exactly 1,047 tests, no mutation, and the external report.

Write UTF-8 YAML-shaped evidence with batch id
`seacad-m14.3dl-m14.3do-entity-xdata-draft-composition-2026-08-08`, status/
root/report/timestamps, HEAD and all four annotated tags, Git/path sets before/after,
every command and hash receipt, `focused_encoded_application_tests`,
`focused_adjacent_tests`, `workspace_test_total`, links, forbidden scan,
`support_matrix_unchanged_except_m14_3dl_through_m14_3do`, protected surfaces,
mutations, deviations, failures, blocker, and final assessment. After writing,
print only the report path and status.
