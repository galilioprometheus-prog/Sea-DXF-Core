# Antigravity Mechanical Verification Batch

Status: READY

Checkpoint: `M14.3dg-entity-xdata-entity-destination`

Repository root: `D:\SeaCad\SeaCad`
Baseline commit: `16ac004a8bbee98f8a2c8ae1c233e9e8e37437f8`
Report: `D:\SeaCad\AntigravityReports\seacad-m14.3dg-entity-xdata-entity-destination-2026-08-08.yaml`
Prepared: `2026-08-08` (`Asia/Saigon`)

## Authority and procedure

Mechanically verify M14.3dg only. Codex and the user retain architecture,
support, license, commit, tag, merge, and release decisions. Use exactly the
repository root above, read `AGENTS.md` and this complete note, confirm exactly
one `READY`, then execute all phases in order. A preflight mismatch is
`BLOCKED`.

The repository is read-only. Ignored `target`/tool-cache activity is tolerated.
Write only the external report above. Do not edit/regenerate, mutate Git,
install tools, use network/vendor CAD, access private/legacy corpus, or create
licenses, exports, archives, or fixtures.

## Preflight

Tracked changed paths exactly:

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

Untracked paths exactly:

```text
crates/seacad-dxf-core/src/entity_xdata_entity_destination.rs
crates/seacad-dxf-core/tests/entity_xdata_entity_destination_tests.rs
docs/audits/M14_3DG_ENTITY_XDATA_ENTITY_DESTINATION.md
```

Run `Get-Location`, `git status --short --branch`, `git rev-parse HEAD`,
`git diff --name-only`, and `git ls-files --others --exclude-standard`. Compare
paths as sets and require the exact baseline.

## Receipts

For every path below record line count and lowercase SHA-256:

| Path | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 155 | `21b281752d54d37f6c5bb4cc7865d55e56a8cf1b5e690eb4a136203189974622` |
| `README.vi.md` | 154 | `73bbebc1ddcf8d3252b402fb0694e478bbee043f8be5a356fc6429f68a82122b` |
| `crates/seacad-dxf-core/src/entity_xdata_entity_destination.rs` | 320 | `67e0c54fe8c7998f302bfe681bddf092b561f6cb5b783440573a2ae4f8629df1` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,183 | `fcbf9efed5945d220138aef46e7a097df85947da4d86bba85aa2fdb66c9bd24e` |
| `crates/seacad-dxf-core/tests/entity_xdata_entity_destination_tests.rs` | 475 | `21a4697ffde052669d8d72613f35d02d494e9987beedeb05364fd0126062873f` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `2e021a1b232b84610f1b6924a0f4882041244e550a997c58449fea2f05a0714c` |
| `docs/SUPPORT_MATRIX.md` | 2,661 | `d4df6b83e31093385b78af7d2e9560880937de05418fd86b94d3204c6ae7fad3` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,034 | `d7e0d186dc3e6063488a4a65982ec414864fa760b5b86a00738af191fbc181d4` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 1,610 | `e61ae64fbb657882651956da9a6f422d130ed676d3e7c3a50bfd49572ca87e97` |
| `docs/audits/M14_3DG_ENTITY_XDATA_ENTITY_DESTINATION.md` | 58 | `faab68abbbe37a239b70d01ba0371215d28732bd43f849a50ede879713e5b378` |

## Focused and contract checks

Run separately:

```powershell
cargo +1.97.1 test -p seacad-dxf-core --test entity_xdata_entity_destination_tests
cargo +1.97.1 test -p seacad-dxf-core --test entity_xdata_application_destination_tests --test entity_xdata_capacity_tests --test entity_xdata_entity_destination_tests
rg -n 'DxfEntityXDataEntityDestination|entity_xdata_entity_destination_directory|application_destination_directory|capacity_directory|WithinLimit|Exceeded|Indeterminate|unavailable_application_count|capacity_issues_for_entry|application_entries_for_entry' crates/seacad-dxf-core/src/entity_xdata_entity_destination.rs crates/seacad-dxf-core/src/lib.rs
rg -n 'every_supported_version_and_format_pair|destination_and_capacity_failures|dual_source_bound|DxfAcadVersion::SUPPORTED|Ascii|Binary|Ac1009|Ac1032|70|Exceeded|Indeterminate|WithinLimit|SourceIdentityMismatch|Cancelled' crates/seacad-dxf-core/tests/entity_xdata_entity_destination_tests.rs
```

Expected 3/3 and 9/9. Prove one entry per indexed entity, including zero-XDATA;
Ready requires all applications ready and within-limit capacity; Unavailable
retains application counts and exact capacity state; destination-only,
indeterminate, and exceeded cases remain distinct; dual-source identity,
cancellation, bounded metadata, owned lookup, and redaction hold.

Run the production forbidden scan and require zero matches:

```powershell
rg -n 'panic!|unwrap\(|expect\(|todo!|unimplemented!|unsafe' crates/seacad-dxf-core/src/entity_xdata_entity_destination.rs
```

Validate all local Markdown links in the six changed documentation files and
the audit. Run and require an empty protected diff plus clean whitespace:

```powershell
rg -n 'M14\.3dg|entity destination|capacity readiness|DxfEntityXDataEntityDestinationDirectory|M14\.3df|M14\.3cr' README.md README.vi.md docs/IMPLEMENTATION_PLAN.md docs/SUPPORT_MATRIX.md docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md docs/audits/M14_3DG_ENTITY_XDATA_ENTITY_DESTINATION.md
git diff -- Cargo.toml Cargo.lock LICENSE NOTICE THIRD_PARTY_NOTICES.md schema corpus release .github .agents
git diff --check
```

## Required gates

Run each separately and require exit zero:

```powershell
cargo deny --locked check
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 run --locked -p seacad-schema-gen -- --check
cargo +1.97.1 run --locked -p seacad-schema-gen --bin seacad-release-evidence -- --check
cargo +1.97.1 clippy --workspace --all-targets -- -D warnings
cargo +1.97.1 test --workspace
$listed = @(cargo +1.97.1 test --workspace -- --list 2>$null | Select-String ': test$')
if ($listed.Count -ne 1022) { throw 'Workspace test count drift.' }
git diff --check
```

Repeat preflight, receipts, protected diff, and whitespace check. HEAD, path
sets, hashes, and protected surfaces must match exactly.

## Acceptance and report

PASS requires exact pre/post state and receipts, 3/3 focused, 9/9 adjacent,
all contract/safety/link/protected checks, every gate, exactly 1,022 workspace
tests, no mutation, and the complete external report. Otherwise FAIL, except a
preflight mismatch is BLOCKED.

Write UTF-8 YAML-shaped evidence with batch id
`seacad-m14.3dg-entity-xdata-entity-destination-2026-08-08`, status, root,
report path, timestamps, HEAD/Git/path sets before/after, every command with
ordinal/phase/cwd/timestamps/exit/stdout/stderr/result, every hash receipt,
`focused_entity_destination_tests`, `focused_adjacent_tests`,
`workspace_test_total`, links, forbidden scan,
`support_matrix_unchanged_except_m14_3dg`, protected surfaces, mutations,
deviations, failures, blocker, and one-sentence final assessment. After writing,
print only the absolute report path and final status.
