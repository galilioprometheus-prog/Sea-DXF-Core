# Antigravity Mechanical Verification Batch

Status: READY

Checkpoint: `M14.3di-entity-xdata-handle-composed-destination`
Repository root: `D:\SeaCad\SeaCad`
Baseline: `8f1ba8cdf0759bf1c7f1d9f93a0b1ad3bb435e7f`
Report: `D:\SeaCad\AntigravityReports\seacad-m14.3di-entity-xdata-handle-composed-destination-2026-08-08.yaml`
Prepared: `2026-08-08` (`Asia/Saigon`)

## Authority, procedure, and writes

Mechanically verify M14.3di only. Codex/user retain architecture, support,
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
crates/seacad-dxf-core/src/entity_xdata_handle_destination.rs
crates/seacad-dxf-core/src/lib.rs
docs/ANTIGRAVITY_MECHANICAL_BATCH.md
docs/IMPLEMENTATION_PLAN.md
docs/SUPPORT_MATRIX.md
docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md
docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md
```

Untracked paths:

```text
crates/seacad-dxf-core/src/entity_xdata_handle_composed_destination.rs
crates/seacad-dxf-core/tests/entity_xdata_handle_composed_destination_tests.rs
docs/audits/M14_3DI_ENTITY_XDATA_HANDLE_COMPOSED_DESTINATION.md
```

Record `Get-Location`, Git status, HEAD, tracked diff names, and untracked names.
Compare paths as sets.

## Artifact receipts

| Path | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 155 | `fe599b817d6548c877e89fd6368485c001c3d54421854ebb05de8a6b1ea304f0` |
| `README.vi.md` | 154 | `c0421f63c25f1267a778dcdbbdeac161fb501d085427cd860e9c7a9bfec7925a` |
| `crates/seacad-dxf-core/src/entity_xdata_handle_destination.rs` | 312 | `ab269305667247c88e72a7be5fe497fc986d489b8d50c65cfbc19d6d71cfb59c` |
| `crates/seacad-dxf-core/src/entity_xdata_handle_composed_destination.rs` | 317 | `283fbb1ae7f7292315ad1b79057e295b5bbdf0a8935b331d4da90c89374d154f` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,193 | `d26076f570aa1dfc35ceb89bf351547376ca9ec7dc87aa32bb2c68fe15ac9a6d` |
| `crates/seacad-dxf-core/tests/entity_xdata_handle_composed_destination_tests.rs` | 562 | `e0bf6216cfa9d1cc9b482b1fe1ed31427b1f2101e4bf58e89ca31f4ef1b9245f` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `97283f71168a176fd5537a90777249ed781192b3fb2b040b584eee2129457eab` |
| `docs/SUPPORT_MATRIX.md` | 2,687 | `f19552f529fc5d5840f6210d03a99d00604ad785ef1f560491cd9da96eea61d7` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,057 | `b4a92802c38ca16eb33e5694ce45a78232cb3826878a5532753c75e0435da153` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 1,634 | `594ff18b95c3821b00a28ca9e47aa1e6d7bf931535b5d898445eb1c24c4ed10c` |
| `docs/audits/M14_3DI_ENTITY_XDATA_HANDLE_COMPOSED_DESTINATION.md` | 62 | `7988d1ce7311fdf29dab9645f6ff3721fe76eac4a328c72b710dd0af64d05df2` |

Compute every line count and lowercase SHA-256 and require exact matches.

## Focused and contract checks

Run separately:

```powershell
cargo +1.97.1 test -p seacad-dxf-core --test entity_xdata_handle_composed_destination_tests
cargo +1.97.1 test -p seacad-dxf-core --test entity_xdata_coordinate_destination_tests --test entity_xdata_handle_destination_tests --test entity_xdata_handle_composed_destination_tests
rg -n 'DxfEntityXDataHandleComposedDestination|entity_xdata_handle_composed_destination_directory|coordinate_destination_directory|handle_destination_directory|handle_entries_for_entry|entries_for_entity|source_entity_for_entry|unavailable_handle_count' crates/seacad-dxf-core/src/entity_xdata_handle_composed_destination.rs crates/seacad-dxf-core/src/entity_xdata_handle_destination.rs crates/seacad-dxf-core/src/lib.rs
rg -n 'every_supported_version_and_format_pair|coordinate_and_handle_failures|dual_source_bound|DxfAcadVersion::SUPPORTED|Ascii|Binary|Ac1009|Ac1032|Missing|Ambiguous|Unmapped|SourceIdentityMismatch|Cancelled|destination_handle_count' crates/seacad-dxf-core/tests/entity_xdata_handle_composed_destination_tests.rs
```

Expected 3/3 and 9/9. Prove one entry per indexed entity; Ready requires M14.3dh
Ready plus every entity-bound group-1005 destination state Unique; zero handles
remain Ready when the coordinate/base state is Ready. Coordinate, base, remap,
and destination failures remain independent and exact through owned evidence.
Require dual-source identity, cancellation, compact bounds, and redaction.

Require zero production matches:

```powershell
rg -n 'panic!|unwrap\(|expect\(|todo!|unimplemented!|unsafe' crates/seacad-dxf-core/src/entity_xdata_handle_composed_destination.rs crates/seacad-dxf-core/src/entity_xdata_handle_destination.rs
```

Validate all local Markdown links in the six changed overview/plan documents
plus the audit. Require an empty protected diff and clean whitespace:

```powershell
rg -n 'M14\.3di|handle composition|handle readiness|DxfEntityXDataHandleComposedDestinationDirectory|M14\.3dh|M14\.3cv' README.md README.vi.md docs/IMPLEMENTATION_PLAN.md docs/SUPPORT_MATRIX.md docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md docs/audits/M14_3DI_ENTITY_XDATA_HANDLE_COMPOSED_DESTINATION.md
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
if ($listed.Count -ne 1028) { throw 'Workspace test count drift.' }
git diff --check
```

Repeat preflight, receipts, protected diff, and whitespace. HEAD/path sets/hashes
must match. PASS requires exact state, receipts, 3/3, 9/9, all contract/safety/
link/protected checks, all gates, exactly 1,028 tests, no mutation, and report.

Write UTF-8 YAML-shaped evidence with batch id
`seacad-m14.3di-entity-xdata-handle-composed-destination-2026-08-08`, status/
root/report/timestamps, HEAD/Git/path sets before/after, every command and hash
receipt, `focused_handle_composed_destination_tests`,
`focused_adjacent_tests`, `workspace_test_total`, links, forbidden scan,
`support_matrix_unchanged_except_m14_3di`, protected surfaces, mutations,
deviations, failures, blocker, and final assessment. After writing, print only
the report path and status.
