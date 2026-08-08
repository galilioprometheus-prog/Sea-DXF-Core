# Antigravity Mechanical Verification Batch

Status: READY

Checkpoint: `M14.3dk-entity-xdata-logical-destination`
Repository root: `D:\SeaCad\SeaCad`
Baseline: `5e0ee4a035df4805e965b41c7dd62f078b98c974`
Report: `D:\SeaCad\AntigravityReports\seacad-m14.3dk-entity-xdata-logical-destination-2026-08-08.yaml`
Prepared: `2026-08-08` (`Asia/Saigon`)

## Authority, procedure, and writes

Mechanically verify M14.3dk only. Codex/user retain architecture, support,
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
crates/seacad-dxf-core/src/entity_xdata_payload_destination.rs
crates/seacad-dxf-core/src/lib.rs
docs/ANTIGRAVITY_MECHANICAL_BATCH.md
docs/IMPLEMENTATION_PLAN.md
docs/SUPPORT_MATRIX.md
docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md
docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md
```

Untracked paths:

```text
crates/seacad-dxf-core/src/entity_xdata_logical_destination.rs
crates/seacad-dxf-core/tests/entity_xdata_logical_destination_tests.rs
docs/audits/M14_3DK_ENTITY_XDATA_LOGICAL_DESTINATION.md
```

Record `Get-Location`, Git status, HEAD, tracked diff names, and untracked names.
Compare paths as sets.

## Artifact receipts

| Path | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 155 | `d8219e54a0f4ae727f5d300c72b67f925a43eaf576e965aa3b7d2ee7db84a7f7` |
| `README.vi.md` | 154 | `d8a7179e993c4f06d669bdf9b0de9fc7a2e1d12ae9b9d079b4bcce758e5f488e` |
| `crates/seacad-dxf-core/src/entity_xdata_handle_destination.rs` | 327 | `3e2ba2bf2f4c482fc9fcb81042d106567afee216537ee338a7d5069080660ef0` |
| `crates/seacad-dxf-core/src/entity_xdata_payload_destination.rs` | 339 | `47c7950a29441a9d990aa0a80541e0f10c177f4431cc6c4853f9fca1df89bfd5` |
| `crates/seacad-dxf-core/src/entity_xdata_logical_destination.rs` | 487 | `192eacdf21fb2cd027132002033ac88429b946ded381c9bb10a8977a15a35025` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,204 | `5ccea8aff5374ec856181c1f2ef4d9704bad843bf7261321d5c2f90342b2ffdf` |
| `crates/seacad-dxf-core/tests/entity_xdata_logical_destination_tests.rs` | 597 | `12936341bf11638e310610cdc0179fff4b02ce51a60d7fb4a3d0c28780616986` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `c674079e616666a52e4ba18c5a101ea68b67dff56bc915049e36c3fd0a15b9e5` |
| `docs/SUPPORT_MATRIX.md` | 2,716 | `fea82d6289230808cddb09c75cc7983188f1a63c9a07b9f21bfc07229d05401f` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,083 | `01d8e873b345fc378ad21294cbb03431c4c86846168727b45f91303c33aaa2c9` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 1,660 | `1d35b8baa82e3eaf277b530db8ad91f6250e439be21dedb4cf1b8c3a39c180fb` |
| `docs/audits/M14_3DK_ENTITY_XDATA_LOGICAL_DESTINATION.md` | 63 | `c15ff1bba42b66e802413971416b3b411921c1c1da80c4757ae46ff4f25e54e0` |

Compute every line count and lowercase SHA-256 and require exact matches.

## Focused and contract checks

Run separately:

```powershell
cargo +1.97.1 test -p seacad-dxf-core --test entity_xdata_logical_destination_tests
cargo +1.97.1 test -p seacad-dxf-core --test entity_xdata_appid_destination_tests --test entity_xdata_layer_destination_tests --test entity_xdata_coordinate_transform_tests --test entity_xdata_handle_destination_tests --test entity_xdata_logical_destination_tests
rg -n 'DxfEntityXDataLogicalDestination|entity_xdata_logical_destination_directory|SourceExact|Application|Layer|Handle|TransformedDouble|InvalidSource|Coordinate|entries_for_entity|payload_entry_for_entry|entry_for_typed' crates/seacad-dxf-core/src/entity_xdata_logical_destination.rs crates/seacad-dxf-core/src/entity_xdata_handle_destination.rs crates/seacad-dxf-core/src/entity_xdata_payload_destination.rs crates/seacad-dxf-core/src/lib.rs
rg -n 'every_supported_version_and_format_pair|specialized_and_source_exact|dual_source_bound|DxfAcadVersion::SUPPORTED|Ascii|Binary|Ac1009|Ac1032|Chunk|Int16|Int32|DestinationMissing|PartialTuple|Orphan|InvalidControlString|SourceIdentityMismatch|Cancelled' crates/seacad-dxf-core/tests/entity_xdata_logical_destination_tests.rs
```

Expected 3/3 and 15/15. Prove one logical entry per typed occurrence; source-
exact families, destination APPID/LAYER, remapped handle, and transformed tuple
components remain distinct. Require exact typed orphan/invalid/application/
layer/handle/coordinate blockers, per-entity slicing, payload association,
dual-source identity, cancellation, compact bounds, and redaction.

Require zero production matches:

```powershell
rg -n 'panic!|unwrap\(|expect\(|todo!|unimplemented!|unsafe' crates/seacad-dxf-core/src/entity_xdata_logical_destination.rs crates/seacad-dxf-core/src/entity_xdata_handle_destination.rs crates/seacad-dxf-core/src/entity_xdata_payload_destination.rs
```

Validate all local Markdown links in the six changed overview/plan documents
plus the audit. Require an empty protected diff and clean whitespace:

```powershell
rg -n 'M14\.3dk|logical destination|logical projection|DxfEntityXDataLogicalDestinationDirectory|M14\.3dj|SourceExact|TransformedDouble' README.md README.vi.md docs/IMPLEMENTATION_PLAN.md docs/SUPPORT_MATRIX.md docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md docs/audits/M14_3DK_ENTITY_XDATA_LOGICAL_DESTINATION.md
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
if ($listed.Count -ne 1034) { throw 'Workspace test count drift.' }
git diff --check
```

Repeat preflight, receipts, protected diff, and whitespace. HEAD/path sets/hashes
must match. PASS requires exact state, receipts, 3/3, 15/15, all contract/
safety/link/protected checks, all gates, exactly 1,034 tests, no mutation, and
the external report.

Write UTF-8 YAML-shaped evidence with batch id
`seacad-m14.3dk-entity-xdata-logical-destination-2026-08-08`, status/root/report/
timestamps, HEAD/Git/path sets before/after, every command and hash receipt,
`focused_logical_destination_tests`, `focused_adjacent_tests`,
`workspace_test_total`, links, forbidden scan,
`support_matrix_unchanged_except_m14_3dk`, protected surfaces, mutations,
deviations, failures, blocker, and final assessment. After writing, print only
the report path and status.
