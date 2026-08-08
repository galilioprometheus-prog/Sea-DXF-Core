# Antigravity Mechanical Verification Batch

Status: READY

Checkpoint: `M14.3dj-entity-xdata-payload-destination`
Repository root: `D:\SeaCad\SeaCad`
Baseline: `d6207a5e9cb5580ffb57d22fd64390099a814df0`
Report: `D:\SeaCad\AntigravityReports\seacad-m14.3dj-entity-xdata-payload-destination-2026-08-08.yaml`
Prepared: `2026-08-08` (`Asia/Saigon`)

## Authority, procedure, and writes

Mechanically verify M14.3dj only. Codex/user retain architecture, support,
license, commit/tag/merge/release decisions. Use the exact root, read `AGENTS.md`
and this note completely, confirm exactly one READY, and run phases in order.
The repository is read-only; ignored target/cache activity is tolerated. Write
only the external report. No edits, Git mutation, installs, network/vendor CAD,
legacy/private corpus, license/export/archive, or fixture creation. A root,
baseline, path, or receipt mismatch is BLOCKED.

## Exact preflight state

Tracked paths:

```text
README.md
README.vi.md
crates/seacad-dxf-core/src/entity_xdata_value.rs
crates/seacad-dxf-core/src/lib.rs
docs/ANTIGRAVITY_MECHANICAL_BATCH.md
docs/IMPLEMENTATION_PLAN.md
docs/SUPPORT_MATRIX.md
docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md
docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md
```

Untracked paths:

```text
crates/seacad-dxf-core/src/entity_xdata_payload_destination.rs
crates/seacad-dxf-core/tests/entity_xdata_payload_destination_tests.rs
docs/audits/M14_3DJ_ENTITY_XDATA_PAYLOAD_DESTINATION.md
```

Record `Get-Location`, Git status, HEAD, tracked diff names, and untracked names.
Compare paths as sets.

## Artifact receipts

| Path | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 155 | `27861aa017c6e78eda4917717d34f7401c00b9283f91ba2002e5258410f9ba57` |
| `README.vi.md` | 154 | `56820c860b9779d3a18eae40d660bc57fe2c728793dc0327386b41395b2501b3` |
| `crates/seacad-dxf-core/src/entity_xdata_value.rs` | 531 | `53bba08f8303b9f7f8dbf3e5b21bcdced8c266e5b9c2e502ec2b01d73ae5f63d` |
| `crates/seacad-dxf-core/src/entity_xdata_payload_destination.rs` | 334 | `f2de80d95679d457d589901784c59bba006d4cc9b0c8f3e3a8a76967e9ee4777` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,198 | `89766745ec427bcdbe14d975d48aafb15f719000607fec0f74f96715ee8cf384` |
| `crates/seacad-dxf-core/tests/entity_xdata_payload_destination_tests.rs` | 498 | `6b8f0457754162ba5b484fa3e8939a350cf3337328eaa99e1a95e34c54661305` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `485b47cc5883bd92b34ca49d8f63635aa020145a2427631adbeb84fd9eb89832` |
| `docs/SUPPORT_MATRIX.md` | 2,701 | `2f5221cd1f517ec8b123ee8bd13896f0b1936e60c19fbf9724f2be1f2e561441` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,070 | `ab032a89900bd85991d560935d60d76e9d888d0666731be3431867aa18a68905` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 1,648 | `94d43a791de32b19fb7b896775f5f91e6616d363090bbc72fca1a30ca8e63984` |
| `docs/audits/M14_3DJ_ENTITY_XDATA_PAYLOAD_DESTINATION.md` | 59 | `d6122d03afbbf679da1a27ae084f0c4014aa5d7a1df91592d2dea678188e0efd` |

Compute every line count and lowercase SHA-256 and require exact matches.

## Focused and contract checks

Run separately:

```powershell
cargo +1.97.1 test -p seacad-dxf-core --test entity_xdata_payload_destination_tests
cargo +1.97.1 test -p seacad-dxf-core --test entity_xdata_capacity_tests --test entity_xdata_handle_composed_destination_tests --test entity_xdata_payload_destination_tests
rg -n 'DxfEntityXDataPayloadDestination|entity_xdata_payload_destination_directory|handle_composed_destination_directory|typed_entries_for_entry|applications_for_entry|entries_for_entity|payload_value_count|orphan_value_count' crates/seacad-dxf-core/src/entity_xdata_payload_destination.rs crates/seacad-dxf-core/src/entity_xdata_value.rs crates/seacad-dxf-core/src/lib.rs
rg -n 'every_supported_version_and_format_pair|orphan_and_composed_failures|dual_source_bound|DxfAcadVersion::SUPPORTED|Ascii|Binary|Ac1009|Ac1032|SECRET_ORPHAN_PAYLOAD|SourceIdentityMismatch|Cancelled|payload_value_count|orphan_value_count' crates/seacad-dxf-core/tests/entity_xdata_payload_destination_tests.rs
```

Expected 3/3 and 9/9. Prove one entry per indexed entity; payload values exclude
group-1001 names; Ready requires M14.3di Ready plus zero orphan values; orphan,
handle, and coordinate failures retain distinct exact nested evidence. Require
owned typed/application slices, zero-payload readiness, dual-source identity,
cancellation, compact bounds, and redaction.

Require zero production matches:

```powershell
rg -n 'panic!|unwrap\(|expect\(|todo!|unimplemented!|unsafe' crates/seacad-dxf-core/src/entity_xdata_payload_destination.rs crates/seacad-dxf-core/src/entity_xdata_value.rs
```

Validate all local Markdown links in the six changed overview/plan documents
plus the audit. Require an empty protected diff and clean whitespace:

```powershell
rg -n 'M14\.3dj|payload-envelope|payload readiness|DxfEntityXDataPayloadDestinationDirectory|M14\.3di|orphan' README.md README.vi.md docs/IMPLEMENTATION_PLAN.md docs/SUPPORT_MATRIX.md docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md docs/audits/M14_3DJ_ENTITY_XDATA_PAYLOAD_DESTINATION.md
git diff -- Cargo.toml Cargo.lock LICENSE NOTICE THIRD_PARTY_NOTICES.md schema corpus release .github .agents
git diff --check
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
if ($listed.Count -ne 1031) { throw 'Workspace test count drift.' }
git diff --check
```

Repeat preflight, receipts, protected diff, and whitespace. HEAD/path sets/hashes
must match. PASS requires exact state, receipts, 3/3, 9/9, all contract/safety/
link/protected checks, all gates, exactly 1,031 tests, no mutation, and report.

Write UTF-8 YAML-shaped evidence with batch id
`seacad-m14.3dj-entity-xdata-payload-destination-2026-08-08`, status/root/report/
timestamps, HEAD/Git/path sets before/after, every command and hash receipt,
`focused_payload_destination_tests`, `focused_adjacent_tests`,
`workspace_test_total`, links, forbidden scan,
`support_matrix_unchanged_except_m14_3dj`, protected surfaces, mutations,
deviations, failures, blocker, and final assessment. After writing, print only
the report path and status.
