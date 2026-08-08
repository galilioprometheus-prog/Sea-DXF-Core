# Antigravity Mechanical Verification Batch

Status: READY

Checkpoint: `M14.3df-entity-xdata-application-destination`

Repository root: `D:\SeaCad\SeaCad`

Baseline commit: `61433f14716b511f0d5e4230a5f51496fd8d93ab`

Required report file:
`D:\SeaCad\AntigravityReports\seacad-m14.3df-entity-xdata-application-destination-2026-08-08.yaml`

Prepared: `2026-08-08` (`Asia/Saigon`)

## 1. Authority and purpose

You are the mechanical verification worker for SeaCad. Verify the proposed
M14.3df per-application XDATA destination symbol-and-structure readiness
checkpoint and collect raw evidence. You do not approve architecture,
normative-source interpretation, support claims, licensing, commit, tag, merge,
publication, or release actions. Codex and the user retain that authority.

## 2. Mandatory procedure

1. Use exactly `D:\SeaCad\SeaCad` as working directory.
2. Read `AGENTS.md` completely.
3. Read this batch completely.
4. Confirm exactly one batch is marked `READY`.
5. Execute every phase in order unless preflight requires `BLOCKED`.

If `AGENTS.md` conflicts with this batch, stop and report the conflict.

## 3. Allowed writes and prohibited actions

The Git repository is read-only. Ignored `target` output and provisioned
tool-cache activity are tolerated. Create or replace exactly this report:

`D:\SeaCad\AntigravityReports\seacad-m14.3df-entity-xdata-application-destination-2026-08-08.yaml`

Do not edit, format/regenerate in write mode, copy source, create fixtures,
mutate Git, install/update tools, use a network, access legacy/private corpus
content, create a license/export/archive, or run vendor CAD software.

## 4. Expected preflight state

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
crates/seacad-dxf-core/src/entity_xdata_application_destination.rs
crates/seacad-dxf-core/tests/entity_xdata_application_destination_tests.rs
docs/audits/M14_3DF_ENTITY_XDATA_APPLICATION_DESTINATION.md
```

Stop `BLOCKED` before quality commands if root, baseline, or either path set
differs.

## 5. Phase 1 - Preflight and receipts

Run and record:

```powershell
Get-Location
git status --short --branch
git rev-parse HEAD
git diff --name-only
git ls-files --others --exclude-standard
$paths = @(
  'README.md',
  'README.vi.md',
  'crates/seacad-dxf-core/src/entity_xdata_application_destination.rs',
  'crates/seacad-dxf-core/src/lib.rs',
  'crates/seacad-dxf-core/tests/entity_xdata_application_destination_tests.rs',
  'docs/IMPLEMENTATION_PLAN.md',
  'docs/SUPPORT_MATRIX.md',
  'docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md',
  'docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md',
  'docs/audits/M14_3DF_ENTITY_XDATA_APPLICATION_DESTINATION.md'
)
foreach ($path in $paths) {
  [PSCustomObject]@{
    Path = $path
    Lines = (Get-Content -LiteralPath $path).Count
    Sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $path).Hash.ToLowerInvariant()
  }
}
```

Compare changed paths as sets. Required receipts:

| Path | Lines | SHA-256 |
| --- | ---: | --- |
| `README.md` | 155 | `7e4ac2e09fec9fc3f91e71e59a674b0b0e7ef2319ad329f0d11ad7bc1a0877c1` |
| `README.vi.md` | 154 | `e3fa31d45b54cf5562fdc686da2b5d9b3c76cb81020e271ca33b7ecb14004d4d` |
| `crates/seacad-dxf-core/src/entity_xdata_application_destination.rs` | 359 | `4f7c6a006810b2fca259eb3338d5637af12e58219ea5265b4272a09b8b701b70` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,178 | `d7f797981ba30b16c0a98bff36ea9920bfbe7430e299a1613ccf7c9aa160bc81` |
| `crates/seacad-dxf-core/tests/entity_xdata_application_destination_tests.rs` | 547 | `7afaf499110498c5db2815ae1b10aee71309180bc09685b58e17001bb56f7bb7` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `6163913461de5dcb5cc4a4221c3c254dc30c57591f4b327652d5a2322775f45a` |
| `docs/SUPPORT_MATRIX.md` | 2,648 | `03cab635982965fa45c94025664fede8fdf070b86ffaf4265a1230c585d4367a` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,022 | `0f0af5a8486ae651d59cfd9ca8b4b9d0ed84d81ab19188c1536c9f8596727023` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 1,597 | `d125a575ffed055eb1051eed7a9532b410a83916ab813c617628e896fc45d062` |
| `docs/audits/M14_3DF_ENTITY_XDATA_APPLICATION_DESTINATION.md` | 76 | `53e6058b5f2ed88f8cc0822ccf7f0720e546943067103d284225443c1d3efe26` |

## 6. Phase 2 - Focused behavior

Run:

```powershell
cargo +1.97.1 test -p seacad-dxf-core --test entity_xdata_application_destination_tests
cargo +1.97.1 test -p seacad-dxf-core --test entity_xdata_appid_destination_tests --test entity_xdata_layer_destination_tests --test entity_xdata_symbol_destination_tests --test entity_xdata_application_destination_tests
```

Expected: `3/3` application-readiness tests and `12/12` combined adjacent tests
pass. Raw assertions must cover all four ASCII/Binary source-destination
pairings across all nine Core dialects, independent symbol/structure blockers,
dual-source identity, cancellation, bounds, and redaction.

## 7. Phase 3 - Composition and safety assertions

Run:

```powershell
rg -n 'DxfEntityXDataApplicationDestination(Directory|Entry|State)|entity_xdata_application_destination_directory|symbol_destination_directory|structure_directory|Ready|Unavailable|symbol_issue_count|structure_issue_count|symbol_issues_for_entry|structure_issues_for_entry|destination_appid_target_for_entry|layer_entries_for_entry' crates/seacad-dxf-core/src/entity_xdata_application_destination.rs crates/seacad-dxf-core/src/lib.rs
rg -n 'every_supported_version_and_format_pair|symbol_and_structure_blockers|dual_source_bound|DxfAcadVersion::SUPPORTED|Ascii|Binary|Ac1009|Ac1032|APP_SYMBOL_BAD|APP_STRUCTURE_BAD|APP_BOTH_BAD|UnclosedLists|DestinationMissing|SourceIdentityMismatch|Cancelled' crates/seacad-dxf-core/tests/entity_xdata_application_destination_tests.rs
$forbidden = @(rg -n 'panic!|unwrap\(|expect\(|todo!|unimplemented!|unsafe' crates/seacad-dxf-core/src/entity_xdata_application_destination.rs)
if ($forbidden.Count -ne 0) { $forbidden; throw 'Forbidden production construct found.' }
```

Required evidence:

- one entry exists per source XDATA application in source order;
- Ready requires M14.3de symbol Ready and M14.3cn structure Valid;
- unavailable state retains independent symbol and structure issue counts;
- exact issue slices remain derived from owned evidence, not copied;
- orphan XDATA is not guessed into an application;
- dual-source identity, cancellation, fallible allocation, compact metadata,
  owned lookup, and redaction remain explicit;
- forbidden production scan reports zero matches.

## 8. Phase 4 - Documentation, links, and protected surfaces

Run:

```powershell
rg -n 'M14\.3df|application destination|symbol and structure|DxfEntityXDataApplicationDestinationDirectory|M14\.3de|M14\.3cn' README.md README.vi.md docs/IMPLEMENTATION_PLAN.md docs/SUPPORT_MATRIX.md docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md docs/audits/M14_3DF_ENTITY_XDATA_APPLICATION_DESTINATION.md
git diff -- Cargo.toml Cargo.lock LICENSE NOTICE THIRD_PARTY_NOTICES.md schema corpus release .github .agents
git diff --check
```

Expected: M14.3df scope is consistent in English and Vietnamese status surfaces,
protected-surface diff is empty, and diff check exits zero.

Validate every non-HTTP, non-mail, non-anchor Markdown link in the six changed
Markdown documentation files and the audit relative to its containing file.
Expected: no missing link.

## 9. Phase 5 - Required gates

Run each command separately and record its complete result:

```powershell
cargo deny --locked check
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 run --locked -p seacad-schema-gen -- --check
cargo +1.97.1 run --locked -p seacad-schema-gen --bin seacad-release-evidence -- --check
cargo +1.97.1 clippy --workspace --all-targets -- -D warnings
cargo +1.97.1 test --workspace
$listed = @(cargo +1.97.1 test --workspace -- --list 2>$null | Select-String ': test$')
"workspace_test_total=$($listed.Count)"
if ($listed.Count -ne 1019) { throw 'Workspace test count drift.' }
git diff --check
```

Every command must exit zero. Expected workspace total: `1,019` tests.

## 10. Phase 6 - Postflight

Repeat Phase 1 Git commands and receipts, protected-surface diff, and
`git diff --check`. Initial and final HEAD, path sets, receipts, and protected
surfaces must match exactly. Only ignored `target` output is tolerated.

## 11. Acceptance criteria

The report is `PASS` only when:

1. root, baseline, exact path sets, and every artifact receipt match;
2. focused and adjacent suites pass 3/3 and 12/12;
3. symbol/structure composition and independently owned blocker contracts are
   present and exercised;
4. all four ASCII/Binary pairings cover all nine Core dialects plus stated
   failure, identity, cancellation, bounds, and redaction cases;
5. forbidden production scan is empty;
6. documentation, local links, and support boundaries are consistent;
7. protected repository surfaces are unchanged;
8. every required gate passes and workspace total is exactly 1,019;
9. postflight matches preflight without repository mutation;
10. the complete external report is written.

An unmet criterion is `FAIL` unless preflight requires `BLOCKED`.

## 12. Required report format

Write one UTF-8 YAML-shaped report:

```yaml
batch_id: seacad-m14.3df-entity-xdata-application-destination-2026-08-08
status: PASS | FAIL | BLOCKED
root: D:\SeaCad\SeaCad
report_file: D:\SeaCad\AntigravityReports\seacad-m14.3df-entity-xdata-application-destination-2026-08-08.yaml
started_at:
finished_at:
head_before:
head_after:
git_before: |
git_after: |
changed_paths_before:
changed_paths_after:
commands:
  - ordinal:
    phase:
    command: |
    cwd:
    started_at:
    finished_at:
    exit_code:
    stdout: |
    stderr: |
    result: PASS | FAIL
hash_receipts:
  - path:
    lines:
    observed_sha256:
    expected_sha256:
    result: PASS | FAIL
focused_application_destination_tests:
focused_adjacent_tests:
workspace_test_total:
links: PASS | FAIL
forbidden_production_scan: PASS | FAIL
support_matrix_unchanged_except_m14_3df: PASS | FAIL
protected_surfaces_unchanged: PASS | FAIL
mutations: none | <exact mutation>
deviations: none | <exact deviation>
failures: none | <exact failure>
blocker: none | <exact blocker>
final_assessment: <one literal sentence>
```

After writing the report, print only its absolute path and final status.
