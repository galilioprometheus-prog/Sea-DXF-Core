# Antigravity Mechanical Verification Batch

Status: READY

Checkpoint: `M14.3dh-entity-xdata-coordinate-destination`
Repository root: `D:\SeaCad\SeaCad`
Baseline: `2bc34f9cb455fd56e881dccbe7e29fbd17880830`
Report: `D:\SeaCad\AntigravityReports\seacad-m14.3dh-entity-xdata-coordinate-destination-2026-08-08.yaml`
Prepared: `2026-08-08` (`Asia/Saigon`)

## Authority, procedure, and writes

Mechanically verify M14.3dh only. Codex/user retain architecture, support,
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
crates/seacad-dxf-core/src/lib.rs
docs/ANTIGRAVITY_MECHANICAL_BATCH.md
docs/IMPLEMENTATION_PLAN.md
docs/SUPPORT_MATRIX.md
docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md
docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md
```

Untracked paths:

```text
crates/seacad-dxf-core/src/entity_xdata_coordinate_destination.rs
crates/seacad-dxf-core/tests/entity_xdata_coordinate_destination_tests.rs
docs/audits/M14_3DH_ENTITY_XDATA_COORDINATE_DESTINATION.md
```

Record `Get-Location`, Git status, HEAD, tracked diff names, and untracked names.
Compare paths as sets.

## Artifact receipts

| Path | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 155 | `70c614d3d3fa83f0411b950502bcafe803413bbb01ca1bc1c1d5036ab3ef7037` |
| `README.vi.md` | 154 | `e740bc7547b556de1c5a5b32dd72911d807496ecbdabc45db2668dc1446e5174` |
| `crates/seacad-dxf-core/src/entity_xdata_coordinate_destination.rs` | 312 | `063576469eb7b093a9129b74f22a38c703229dcd473c43d5188f2226c5130118` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,188 | `acc67765569ce1ab4f864986f6fc535d10c35298ae781aa00616125cafc9e9bf` |
| `crates/seacad-dxf-core/tests/entity_xdata_coordinate_destination_tests.rs` | 457 | `ec21af152e6ef2c54b070d0b0a2d7827cae2c8c7f58a7a6ccaf866b5a59f8ec8` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `e40fe949c33744a4d5c746ac61547fce7215d430417411caea7c4090f150229a` |
| `docs/SUPPORT_MATRIX.md` | 2,673 | `b75193dec9e44b433f55f7beb19632982b7234727f051bfda113efd4af93b6e5` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,044 | `7b7d3d351104fc10912d0d9011a1a97e0d6ebb5434f3c3624985ce8b36fbc24a` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 1,621 | `28783c41f89f4977847eef584efc10be43d8a502f93f3841d1b81d7b11e0d5fd` |
| `docs/audits/M14_3DH_ENTITY_XDATA_COORDINATE_DESTINATION.md` | 55 | `ac7dfc73319a2c0e6c66b52b11ed8ecfcf77a7ac88f744f5e11212cf7bd6defb` |

Compute every line count and lowercase SHA-256 and require exact matches.

## Focused and contract checks

Run separately:

```powershell
cargo +1.97.1 test -p seacad-dxf-core --test entity_xdata_coordinate_destination_tests
cargo +1.97.1 test -p seacad-dxf-core --test entity_xdata_coordinate_transform_tests --test entity_xdata_entity_destination_tests --test entity_xdata_coordinate_destination_tests
rg -n 'DxfEntityXDataCoordinateDestination|entity_xdata_coordinate_destination_directory|entity_destination_directory|coordinate_transform_directory|transformed_entries_for_entry|NonFiniteDerivedPoint|PartialTuple|unavailable_tuple_count' crates/seacad-dxf-core/src/entity_xdata_coordinate_destination.rs crates/seacad-dxf-core/src/lib.rs
rg -n 'every_supported_version_and_format_pair|entity_destination_and_transform_failures|dual_source_bound|DxfAcadVersion::SUPPORTED|Ascii|Binary|Ac1009|Ac1032|1010|1011|NonFiniteDerivedPoint|PartialTuple|SourceIdentityMismatch|Cancelled' crates/seacad-dxf-core/tests/entity_xdata_coordinate_destination_tests.rs
```

Expected 3/3 and 9/9. Prove one entry per indexed entity; Ready requires base
Ready plus all entity tuples Available; zero tuples remain Ready; base failure,
non-finite role transform, and partial tuple remain distinct; owned slices,
dual-source identity, cancellation, compact bounds, and redaction hold.

Require zero production matches:

```powershell
rg -n 'panic!|unwrap\(|expect\(|todo!|unimplemented!|unsafe' crates/seacad-dxf-core/src/entity_xdata_coordinate_destination.rs
```

Validate all local Markdown links in six changed docs plus audit. Require an
empty protected diff and clean whitespace:

```powershell
rg -n 'M14\.3dh|coordinate destination|coordinate readiness|DxfEntityXDataCoordinateDestinationDirectory|M14\.3dg|M14\.3cs' README.md README.vi.md docs/IMPLEMENTATION_PLAN.md docs/SUPPORT_MATRIX.md docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md docs/audits/M14_3DH_ENTITY_XDATA_COORDINATE_DESTINATION.md
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
if ($listed.Count -ne 1025) { throw 'Workspace test count drift.' }
git diff --check
```

Repeat preflight, receipts, protected diff, and whitespace. HEAD/path sets/hashes
must match. PASS requires exact state, receipts, 3/3, 9/9, all contract/safety/
link/protected checks, all gates, exactly 1,025 tests, no mutation, and report.

Write UTF-8 YAML-shaped evidence with batch id
`seacad-m14.3dh-entity-xdata-coordinate-destination-2026-08-08`, status/root/
report/timestamps, HEAD/Git/path sets before/after, every command and hash
receipt, `focused_coordinate_destination_tests`, `focused_adjacent_tests`,
`workspace_test_total`, links, forbidden scan,
`support_matrix_unchanged_except_m14_3dh`, protected surfaces, mutations,
deviations, failures, blocker, and final assessment. After writing, print only
the report path and status.
