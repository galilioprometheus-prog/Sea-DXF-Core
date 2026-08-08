# Antigravity Mechanical Verification Batch

Status: READY

Checkpoint: `M14.3de-entity-xdata-symbol-destination`

Repository root: `D:\SeaCad\SeaCad`

Baseline commit: `120eab200bc04926349939db0e9690e6e3fc891c`

Required report file:
`D:\SeaCad\AntigravityReports\seacad-m14.3de-entity-xdata-symbol-destination-2026-08-08.yaml`

Prepared: `2026-08-08` (`Asia/Saigon`)

## 1. Authority and purpose

You are the mechanical verification worker for SeaCad. Verify the proposed
M14.3de per-application XDATA destination-symbol readiness checkpoint and
collect raw evidence. You do not approve architecture, normative-source
interpretation, support claims, licensing, commit, tag, merge, publication, or
release actions. Codex and the user retain that authority.

## 2. Mandatory procedure

1. Use exactly `D:\SeaCad\SeaCad` as working directory.
2. Read `AGENTS.md` completely.
3. Read this batch completely.
4. Confirm exactly one batch is marked `READY`.
5. Execute every phase in order unless preflight requires `BLOCKED`.

If `AGENTS.md` conflicts with this batch, stop and report the conflict.

## 3. Allowed writes and prohibited actions

The Git repository is read-only. Ignored `target` output and provisioned
tool-cache activity are tolerated. Create or replace exactly this external
report:

`D:\SeaCad\AntigravityReports\seacad-m14.3de-entity-xdata-symbol-destination-2026-08-08.yaml`

Do not edit, format or regenerate in write mode, copy source, create fixtures,
mutate Git, install/update tools, use a network, access legacy/private corpus
content, create a license/export/archive, or run vendor CAD software.

## 4. Expected preflight state

The exact tracked changed-path set is:

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

The exact untracked-path set is:

```text
crates/seacad-dxf-core/src/entity_xdata_symbol_destination.rs
crates/seacad-dxf-core/tests/entity_xdata_symbol_destination_tests.rs
docs/audits/M14_3DE_ENTITY_XDATA_SYMBOL_DESTINATION.md
```

Stop `BLOCKED` before quality commands if root, baseline, or either path set
differs.

## 5. Phase 1 - Preflight and artifact receipts

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
  'crates/seacad-dxf-core/src/entity_xdata_symbol_destination.rs',
  'crates/seacad-dxf-core/src/lib.rs',
  'crates/seacad-dxf-core/tests/entity_xdata_symbol_destination_tests.rs',
  'docs/IMPLEMENTATION_PLAN.md',
  'docs/SUPPORT_MATRIX.md',
  'docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md',
  'docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md',
  'docs/audits/M14_3DE_ENTITY_XDATA_SYMBOL_DESTINATION.md'
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
| `README.md` | 155 | `1c7647bceedb7ea76cad68861851bc8d472f27cc6a61cc29a83c74cd9a0a9902` |
| `README.vi.md` | 154 | `a40ca52b116044924beae189fd5ce331afc945e0633504bfb0aa3b5f011ed324` |
| `crates/seacad-dxf-core/src/entity_xdata_symbol_destination.rs` | 463 | `187db01d6c9a20cee113c8702ec705da8603cd3bee3d2f737d8372cad184db8c` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,173 | `e96d08e449b9b900bc77634f37ab7454afc2724bce2506d5566012d43f09bf78` |
| `crates/seacad-dxf-core/tests/entity_xdata_symbol_destination_tests.rs` | 624 | `d0d0600b266167df3fa65a7cc835eb47e52dfc88621432444c8a0aea634a7d7e` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `ead2b28d12541f57181abcdb0d50cf5a7cd132d771ed70868c7cd7a68fbe6ee6` |
| `docs/SUPPORT_MATRIX.md` | 2,634 | `7b2383b5c009d5dc7138fd6e1268f50e5bd4dc43a900c5fd8220f60a50efc3d6` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,009 | `abf04d37e8fa0fa2efd3e5ed600dcbadca01caf8bf8f73ecb710fc74daa7b317` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 1,583 | `e71ebbc5545b695436c01b3e10c4e2bb30f7252c7bd22fc3d52fc948c6f347e7` |
| `docs/audits/M14_3DE_ENTITY_XDATA_SYMBOL_DESTINATION.md` | 75 | `cad1e1bfe53af287150a812ad510cc6f170423dfee95e882f529168b80d6833f` |

## 6. Phase 2 - Focused behavior

Run:

```powershell
cargo +1.97.1 test -p seacad-dxf-core --test entity_xdata_symbol_destination_tests
cargo +1.97.1 test -p seacad-dxf-core --test entity_xdata_appid_destination_tests --test entity_xdata_layer_destination_tests --test entity_xdata_symbol_destination_tests
```

Expected: `3/3` symbol-readiness tests and `9/9` combined adjacent tests pass.
The test names and raw assertions must cover all four ASCII/Binary source-
destination pairings across all nine Core dialects, accumulated malformed
APPID/LAYER issues, dual-source identity, cancellation, bounds, and redaction.

## 7. Phase 3 - Composition contract and safety scan

Run:

```powershell
rg -n 'DxfEntityXDataSymbolDestination(Directory|Entry|Issue|IssueKind|IssueRange|State)|entity_xdata_symbol_destination_directory|appid_destination_directory|layer_destination_directory|Ready|Unavailable|collect_appid_issue|collect_layer_issues|destination_appid_target_for_entry|layer_entries_for_entry' crates/seacad-dxf-core/src/entity_xdata_symbol_destination.rs crates/seacad-dxf-core/src/lib.rs
rg -n 'every_supported_version_and_format_pair|malformed_appid_and_layer_destinations|dual_source_bound|DxfAcadVersion::SUPPORTED|Ascii|Binary|Ac1009|Ac1032|DestinationMissing|DestinationAmbiguous|SourceMissing|SourceAmbiguous|CaseLayer|MULTI_ISSUE|SourceIdentityMismatch|Cancelled' crates/seacad-dxf-core/tests/entity_xdata_symbol_destination_tests.rs
$forbidden = @(rg -n 'panic!|unwrap\(|expect\(|todo!|unimplemented!|unsafe' crates/seacad-dxf-core/src/entity_xdata_symbol_destination.rs)
if ($forbidden.Count -ne 0) { $forbidden; throw 'Forbidden production construct found.' }
```

Required evidence:

- one source-order entry exists per source XDATA application;
- Ready requires destination-unique APPID plus destination-unique state for
  every application-bound group-1003 occurrence;
- unavailable state accumulates APPID first, then LAYER blockers in source order;
- orphan LAYER values remain visible in the owned LAYER directory and are not
  assigned to an application;
- source and destination identities, owned lookups, cancellation, fallible
  allocation, compact ordinals, and redaction remain explicit;
- the forbidden production scan reports zero matches.

## 8. Phase 4 - Documentation, links, and protected surfaces

Run:

```powershell
rg -n 'M14\.3de|symbol readiness|symbol-destination|DxfEntityXDataSymbolDestinationDirectory|M14\.3dc|M14\.3dd' README.md README.vi.md docs/IMPLEMENTATION_PLAN.md docs/SUPPORT_MATRIX.md docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md docs/audits/M14_3DE_ENTITY_XDATA_SYMBOL_DESTINATION.md
git diff -- Cargo.toml Cargo.lock LICENSE NOTICE THIRD_PARTY_NOTICES.md schema corpus release .github .agents
git diff --check
```

Expected: M14.3de scope is consistent in English and Vietnamese status surfaces,
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
if ($listed.Count -ne 1016) { throw 'Workspace test count drift.' }
git diff --check
```

Every command must exit zero. Expected workspace total: `1,016` tests.

## 10. Phase 6 - Postflight

Repeat Phase 1 Git commands and all receipts, the protected-surface diff, and
`git diff --check`. Initial and final HEAD, path sets, receipts, and protected
surfaces must match exactly. Only ignored `target` output is tolerated.

## 11. Acceptance criteria

The report is `PASS` only when:

1. root, baseline, exact path sets, and every artifact receipt match;
2. focused and adjacent suites pass 3/3 and 9/9;
3. exact APPID/LAYER composition, blocker ordering, and owned lookup contracts
   are present and exercised;
4. all four ASCII/Binary pairings cover all nine Core dialects plus stated
   malformed, identity, cancellation, bounds, and redaction cases;
5. forbidden production scan is empty;
6. documentation, local links, and support boundaries are consistent;
7. protected repository surfaces are unchanged;
8. every required gate passes and workspace total is exactly 1,016;
9. postflight matches preflight without repository mutation;
10. the complete external report is written.

An unmet criterion is `FAIL` unless preflight requires `BLOCKED`.

## 12. Required report format

Write one UTF-8 YAML-shaped report:

```yaml
batch_id: seacad-m14.3de-entity-xdata-symbol-destination-2026-08-08
status: PASS | FAIL | BLOCKED
root: D:\SeaCad\SeaCad
report_file: D:\SeaCad\AntigravityReports\seacad-m14.3de-entity-xdata-symbol-destination-2026-08-08.yaml
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
focused_symbol_destination_tests:
focused_adjacent_tests:
workspace_test_total:
links: PASS | FAIL
forbidden_production_scan: PASS | FAIL
support_matrix_unchanged_except_m14_3de: PASS | FAIL
protected_surfaces_unchanged: PASS | FAIL
mutations: none | <exact mutation>
deviations: none | <exact deviation>
failures: none | <exact failure>
blocker: none | <exact blocker>
final_assessment: <one literal sentence>
```

After writing the report, print only its absolute path and final status.
