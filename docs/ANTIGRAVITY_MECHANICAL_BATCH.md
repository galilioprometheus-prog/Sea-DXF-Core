# Antigravity Mechanical Verification Batch

Status: READY

Checkpoint: `M14.3dl-entity-xdata-encoded-destination`
Repository root: `D:\SeaCad\SeaCad`
Baseline: `c502d1a6c1f709c9fdc85e9e1fa8bc58d6d43a14`
Report: `D:\SeaCad\AntigravityReports\seacad-m14.3dl-entity-xdata-encoded-destination-2026-08-08.yaml`
Prepared: `2026-08-08` (`Asia/Saigon`)

## Authority, procedure, and writes

Mechanically verify M14.3dl only. Codex/user retain architecture, support,
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
crates/seacad-dxf-core/src/entity_xdata_encoded_destination.rs
crates/seacad-dxf-core/tests/entity_xdata_encoded_destination_tests.rs
docs/audits/M14_3DL_ENTITY_XDATA_ENCODED_DESTINATION.md
```

Record `Get-Location`, Git status, HEAD, tracked diff names, and untracked names.
Compare paths as sets.

## Artifact receipts

| Path | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 155 | `987abecfbe05336d61a7329cd4f4ffdee4978b38a4e2e11894fd8de728f3bb4d` |
| `README.vi.md` | 154 | `9b019e565350690509aa7093c562968b89352f0c4df8e0abf98a72e8c7d3320d` |
| `crates/seacad-dxf-core/src/entity_xdata_encoded_destination.rs` | 557 | `0fbc6fc07fa32680d501c19595b9aceb0429683e86b9b2c39200681b5204769b` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,209 | `0b1fc913b1aea3a4c5f8f45b79908c02a4945683125eb906ad5a4eaf087c87f9` |
| `crates/seacad-dxf-core/tests/entity_xdata_encoded_destination_tests.rs` | 596 | `5e37a0db8660e1adff5215fdf46703b9159801f8072396802f3c4594c20b5291` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `5b28eaa94f6d4d699f8a1051d108c3e37f659ccc8a07e832157290b99a97ce6e` |
| `docs/SUPPORT_MATRIX.md` | 2,732 | `b58deadcc8b57a719fc36259e9b8973d6f64d31a69131fb861697e1377eb0ac2` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,098 | `1b2fd4b5bf6e1c7784eee5849b236bab3b01bc77050dad29385aa58574b8a561` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 1,675 | `ce2f769802135f2a138ea5ce7540b9e5281f3693ba8a7baa6fb2fdef1e1c5e58` |
| `docs/audits/M14_3DL_ENTITY_XDATA_ENCODED_DESTINATION.md` | 59 | `b15602b92dde03f08e8c5a07925fa9c3fa6aa0190b258a6366dc0e39d294115a` |

Compute every line count and lowercase SHA-256 and require exact matches.

## Focused and contract checks

Run separately:

```powershell
cargo +1.97.1 test -p seacad-dxf-core --test entity_xdata_encoded_destination_tests
cargo +1.97.1 test -p seacad-dxf-core --test entity_value_encoder_tests --test entity_xdata_handle_replacement_tests --test entity_xdata_logical_destination_tests --test entity_xdata_encoded_destination_tests
rg -n 'DxfEntityXDataEncodedDestination|entity_xdata_encoded_destination_directory|TextTranscodingRequired|LogicalUnavailable|DialectUnavailable|EncodingUnavailable|encoded_bytes_for_entry|portable_text|read_binary_chunk|encode_raw|AC1009' crates/seacad-dxf-core/src/entity_xdata_encoded_destination.rs crates/seacad-dxf-core/src/lib.rs
rg -n 'every_supported_version_and_format_pair|non_ascii_text_requires|dual_source_bound|DxfAcadVersion::SUPPORTED|Ascii|Binary|Ac1009|Ac1032|Chunk|Transformed|ANSI_1252|ANSI_932|TextTranscodingRequired|SourceIdentityMismatch|Cancelled' crates/seacad-dxf-core/tests/entity_xdata_encoded_destination_tests.rs
```

Expected 3/3 and 16/16. Prove one encoded state per M14.3dk entry;
destination APPID/LAYER spans, source-exact values, transformed doubles,
remapped handles, integers, controls, and chunks use canonical destination
framing. Require AC1009 XDATA escapes, exact unavailable-byte suppression,
ASCII portability, non-ASCII decoder matching, dual-source identity,
cancellation, compact bounds, and redaction.

Require zero production matches:

```powershell
rg -n 'panic!|unwrap\(|expect\(|todo!|unimplemented!|unsafe' crates/seacad-dxf-core/src/entity_xdata_encoded_destination.rs
```

Validate all local Markdown links in the six changed overview/plan documents
plus the audit. Require an empty protected diff and clean whitespace:

```powershell
rg -n 'M14\.3dl|encoded destination|canonical destination|DxfEntityXDataEncodedDestinationDirectory|M14\.3dk|TextTranscodingRequired|AC1009' README.md README.vi.md docs/IMPLEMENTATION_PLAN.md docs/SUPPORT_MATRIX.md docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md docs/audits/M14_3DL_ENTITY_XDATA_ENCODED_DESTINATION.md
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
if ($listed.Count -ne 1037) { throw 'Workspace test count drift.' }
git diff --check
```

Repeat preflight, receipts, protected diff, and whitespace. HEAD/path sets/hashes
must match. PASS requires exact state, receipts, 3/3, 16/16, all contract/
safety/link/protected checks, all gates, exactly 1,037 tests, no mutation, and
the external report.

Write UTF-8 YAML-shaped evidence with batch id
`seacad-m14.3dl-entity-xdata-encoded-destination-2026-08-08`, status/root/report/
timestamps, HEAD/Git/path sets before/after, every command and hash receipt,
`focused_encoded_destination_tests`, `focused_adjacent_tests`,
`workspace_test_total`, links, forbidden scan,
`support_matrix_unchanged_except_m14_3dl`, protected surfaces, mutations,
deviations, failures, blocker, and final assessment. After writing, print only
the report path and status.
