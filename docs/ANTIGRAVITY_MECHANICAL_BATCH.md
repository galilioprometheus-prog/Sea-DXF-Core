# Antigravity Mechanical Verification Batch

Status: READY

Checkpoint: `M14.3dl..M14.3dn-entity-xdata-encoded-payloads`
Repository root: `D:\SeaCad\SeaCad`
Baseline: `c502d1a6c1f709c9fdc85e9e1fa8bc58d6d43a14`
Review target: annotated tag `m14.3dn-entity-xdata-encoded-entity-destination`
Report: `D:\SeaCad\AntigravityReports\seacad-m14.3dl-m14.3dn-entity-xdata-encoded-payloads-2026-08-08.yaml`
Prepared: `2026-08-08` (`Asia/Saigon`)

## Authority, procedure, and writes

Mechanically verify the bounded cumulative M14.3dl through M14.3dn checkpoints
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
HEAD == m14.3dn-entity-xdata-encoded-entity-destination^{}
m14.3dl-entity-xdata-encoded-destination^{} == 76a9179796178c2c683165fc04844b6448459df1
m14.3dm-entity-xdata-encoded-application-destination^{} == 74e33dd9024484d394877146c429ffd32067ec90
baseline is an ancestor of M14.3dl, M14.3dm, and M14.3dn in that order
all three checkpoint tags are annotated tag objects
```

The cumulative changed path set must be exactly:

```text
README.md
README.vi.md
crates/seacad-dxf-core/src/entity_xdata_encoded_application_destination.rs
crates/seacad-dxf-core/src/entity_xdata_encoded_destination.rs
crates/seacad-dxf-core/src/entity_xdata_encoded_entity_destination.rs
crates/seacad-dxf-core/src/lib.rs
crates/seacad-dxf-core/tests/entity_xdata_encoded_destination_tests.rs
docs/ANTIGRAVITY_MECHANICAL_BATCH.md
docs/IMPLEMENTATION_PLAN.md
docs/SUPPORT_MATRIX.md
docs/audits/M14_3DL_ENTITY_XDATA_ENCODED_DESTINATION.md
docs/audits/M14_3DM_ENTITY_XDATA_ENCODED_APPLICATION_DESTINATION.md
docs/audits/M14_3DN_ENTITY_XDATA_ENCODED_ENTITY_DESTINATION.md
docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md
docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md
```

Compare paths as sets.

## Artifact receipts

| Path | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 158 | `745e1f69a18a893a509b2f5e6f9283c1f6c080dc62df7d1849f60f597e79f3a9` |
| `README.vi.md` | 156 | `ad0fcb7ff9fb3c388647ccfd2557470e006b9d2d675638a839ee41287bb8b139` |
| `crates/seacad-dxf-core/src/entity_xdata_encoded_application_destination.rs` | 462 | `9b8d1dce1370fa2369c1c97296c3e8aa7e3f67d16a96b9ec1dbbf083f454784b` |
| `crates/seacad-dxf-core/src/entity_xdata_encoded_destination.rs` | 580 | `011b9f8771af6f62a652094283787990f48bbf32e248743a3a5d24ddaa4c9e33` |
| `crates/seacad-dxf-core/src/entity_xdata_encoded_entity_destination.rs` | 414 | `8ad238f45655582faa7a54d23d0ded371150c913d1fd3989a5c560bd4a9f9ad6` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,220 | `6d101f7edb8c04e42c3599531e658fdf96b4df92b52f6a3fc4dc1d6059670a13` |
| `crates/seacad-dxf-core/tests/entity_xdata_encoded_destination_tests.rs` | 1,225 | `198a4f2be745da5e71ac51e543a544d3720ff82a6f94623c293d88d9d1fe3afd` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `5c628f493d19203eb09b7ce2587808c09350bd84f8904cc0d62e4270acc454ba` |
| `docs/SUPPORT_MATRIX.md` | 2,762 | `fe46277b9bb753e9780c43520bfcf4d9a2314cd548a8e6901a12f936602dba27` |
| `docs/audits/M14_3DL_ENTITY_XDATA_ENCODED_DESTINATION.md` | 59 | `b15602b92dde03f08e8c5a07925fa9c3fa6aa0190b258a6366dc0e39d294115a` |
| `docs/audits/M14_3DM_ENTITY_XDATA_ENCODED_APPLICATION_DESTINATION.md` | 59 | `16ea8a958a007412e9489744dca579f36f4cbb95e56df4c9ab14edcc1ca26739` |
| `docs/audits/M14_3DN_ENTITY_XDATA_ENCODED_ENTITY_DESTINATION.md` | 51 | `62a16d9e227b150754655ccf5b77b712330659a34a17f4c5a87ed80ab21a4ae0` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 1,704 | `567cec13a589b539d8d1eae75cf8ba55de52093f784cdbc5d3f972e944d1c5c5` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,126 | `2610254e8482db999361bcaa35ee22302e42f15f196fa1980f2bd66e31a935b5` |

Compute every line count and lowercase SHA-256 and require exact matches. The
active batch note intentionally omits its own self-referential receipt.

## Focused and contract checks

Run separately:

```powershell
cargo +1.97.1 test -p seacad-dxf-core --test entity_xdata_encoded_destination_tests
cargo +1.97.1 test -p seacad-dxf-core --test entity_value_encoder_tests --test entity_xdata_handle_replacement_tests --test entity_xdata_logical_destination_tests --test entity_xdata_encoded_destination_tests
rg -n 'DxfEntityXDataEncodedDestination|DxfEntityXDataEncodedApplicationDestination|DxfEntityXDataEncodedEntityDestination|encoded_bytes_for_ready_range|first_unavailable_member_ordinal|first_unavailable_application_ordinal|payload_for_entry|applications_for_entry|encoded_bytes_for_entry' crates/seacad-dxf-core/src/entity_xdata_encoded_destination.rs crates/seacad-dxf-core/src/entity_xdata_encoded_application_destination.rs crates/seacad-dxf-core/src/entity_xdata_encoded_entity_destination.rs crates/seacad-dxf-core/src/lib.rs
rg -n 'encoded_application|encoded_entit|DxfAcadVersion::SUPPORTED|Ascii|Binary|Ac1009|Ac1032|TextTranscodingRequired|APP_EMPTY|APP_NESTED|ORPHAN_SECRET|Cancelled|foreign|non_disclosing' crates/seacad-dxf-core/tests/entity_xdata_encoded_destination_tests.rs
```

Expected 10/10 and 23/23. Reconfirm the M14.3dl one-state-per-logical-occurrence,
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

Require zero production matches:

```powershell
rg -n 'panic!|unwrap\(|expect\(|todo!|unimplemented!|unsafe' crates/seacad-dxf-core/src/entity_xdata_encoded_destination.rs crates/seacad-dxf-core/src/entity_xdata_encoded_application_destination.rs crates/seacad-dxf-core/src/entity_xdata_encoded_entity_destination.rs
```

Validate every local Markdown link in the changed overview/plan/audit files.
Require an empty protected diff and clean whitespace:

```powershell
rg -n 'M14\.3d[klmn]|EncodedApplication|EncodedEntity|per-application|per-entity|nhóm encoded' README.md README.vi.md docs/IMPLEMENTATION_PLAN.md docs/SUPPORT_MATRIX.md docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md docs/audits/M14_3DL_ENTITY_XDATA_ENCODED_DESTINATION.md docs/audits/M14_3DM_ENTITY_XDATA_ENCODED_APPLICATION_DESTINATION.md docs/audits/M14_3DN_ENTITY_XDATA_ENCODED_ENTITY_DESTINATION.md
git diff c502d1a6c1f709c9fdc85e9e1fa8bc58d6d43a14..m14.3dn-entity-xdata-encoded-entity-destination -- Cargo.toml Cargo.lock LICENSE NOTICE THIRD_PARTY_NOTICES.md schema corpus release .github .agents
git diff --check c502d1a6c1f709c9fdc85e9e1fa8bc58d6d43a14..m14.3dn-entity-xdata-encoded-entity-destination
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
if ($listed.Count -ne 1044) { throw 'Workspace test count drift.' }
git diff --check c502d1a6c1f709c9fdc85e9e1fa8bc58d6d43a14..m14.3dn-entity-xdata-encoded-entity-destination
```

Repeat preflight, receipts, protected diff, and whitespace. HEAD/tag/path sets/
hashes must match and the worktree must remain clean. PASS requires exact state,
receipts, 10/10, 23/23, all contract/safety/link/protected checks, all gates,
exactly 1,044 tests, no mutation, and the external report.

Write UTF-8 YAML-shaped evidence with batch id
`seacad-m14.3dl-m14.3dn-entity-xdata-encoded-payloads-2026-08-08`, status/
root/report/timestamps, HEAD and all three annotated tags, Git/path sets before/after,
every command and hash receipt, `focused_encoded_application_tests`,
`focused_adjacent_tests`, `workspace_test_total`, links, forbidden scan,
`support_matrix_unchanged_except_m14_3dl_through_m14_3dn`, protected surfaces,
mutations, deviations, failures, blocker, and final assessment. After writing,
print only the report path and status.
