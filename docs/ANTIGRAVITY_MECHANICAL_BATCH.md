# Antigravity Mechanical Verification Batch

Status: READY

Checkpoint: `M14.3dd-entity-xdata-layer-destination`

Repository root: `D:\SeaCad\SeaCad`

Baseline commit: `5f77482a1c37fb6f8b2648f3e63bdad60b48c78a`

Required report file:
`D:\SeaCad\AntigravityReports\seacad-m14.3dd-entity-xdata-layer-destination-2026-08-08.yaml`

Prepared: `2026-08-08` (`Asia/Saigon`)

## 1. Authority and purpose

You are the mechanical verification worker for SeaCad. Verify the proposed
M14.3dd destination-document LAYER validation checkpoint and collect raw
evidence. You do not approve architecture, normative-source interpretation,
support claims, licensing, commit, tag, merge, publication, or release actions.
Codex and the user retain that authority.

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

`D:\SeaCad\AntigravityReports\seacad-m14.3dd-entity-xdata-layer-destination-2026-08-08.yaml`

Do not edit, format or regenerate in write mode, copy source, create fixtures,
mutate Git, install or update tools, use a network, access legacy/private corpus
content, create a license/export/archive, or run vendor CAD software.

## 4. Expected preflight state

The exact tracked changed-path set is:

```text
README.md
README.vi.md
crates/seacad-dxf-core/src/entity_xdata_appid_destination.rs
crates/seacad-dxf-core/src/lib.rs
docs/ANTIGRAVITY_MECHANICAL_BATCH.md
docs/IMPLEMENTATION_PLAN.md
docs/SUPPORT_MATRIX.md
docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md
docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md
```

The exact untracked-path set is:

```text
crates/seacad-dxf-core/src/entity_xdata_layer_destination.rs
crates/seacad-dxf-core/src/named_symbol_destination.rs
crates/seacad-dxf-core/tests/entity_xdata_layer_destination_tests.rs
docs/audits/M14_3DD_ENTITY_XDATA_LAYER_DESTINATION.md
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
  'crates/seacad-dxf-core/src/entity_xdata_appid_destination.rs',
  'crates/seacad-dxf-core/src/entity_xdata_layer_destination.rs',
  'crates/seacad-dxf-core/src/named_symbol_destination.rs',
  'crates/seacad-dxf-core/src/lib.rs',
  'crates/seacad-dxf-core/tests/entity_xdata_layer_destination_tests.rs',
  'docs/IMPLEMENTATION_PLAN.md',
  'docs/SUPPORT_MATRIX.md',
  'docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md',
  'docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md',
  'docs/audits/M14_3DD_ENTITY_XDATA_LAYER_DESTINATION.md'
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
| `README.md` | 154 | `b512ca35a20034c3349660d2f62d7480a3abf498d0473f17cb13ddc412fbcf91` |
| `README.vi.md` | 153 | `263ce8c5484508c049d74140abd53612672ab904c902d417916d4a53b05462db` |
| `crates/seacad-dxf-core/src/entity_xdata_appid_destination.rs` | 313 | `e3c65574d17d4f8c0fddaadb9bfd945949ebe8ccf23835a62bf0c7be5571374b` |
| `crates/seacad-dxf-core/src/entity_xdata_layer_destination.rs` | 360 | `12b3ce4ce1e278500de4a63f36687df1dd3539a244c69160b57ddd3ca28d8c75` |
| `crates/seacad-dxf-core/src/named_symbol_destination.rs` | 129 | `474b3f8d48af6c00d62046c059adc476dc619630424f90059be9c41694b3807d` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,167 | `ba28cf10c9ab49125667284a4c61e34896ea1b9d6d1c5857189e3e97d740ff53` |
| `crates/seacad-dxf-core/tests/entity_xdata_layer_destination_tests.rs` | 565 | `6bc34d766d03666585042020e5b1158a4dfe013a757eb90a2161d76e3fd2090d` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `61a5bcf2d7a97a97d0eb25fe63c11a4cf80ac09d6866bb1c006b609e4e4eea17` |
| `docs/SUPPORT_MATRIX.md` | 2,618 | `bb9c628f3845c3605d9892cd0473398d423e2548fb4e7c4036938c111f2b375a` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 2,994 | `f9b54d71f3c3a00441d2c1e378ca097483cdc70a84ed8a1738a6249e1467272c` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 1,567 | `438d7870a3b2688563ce65a989c4a8e90cd18dc848de4fc9611834043b83c5e4` |
| `docs/audits/M14_3DD_ENTITY_XDATA_LAYER_DESTINATION.md` | 85 | `85bec1e34d48a8f8c6cdc70023ae1fd38c67e7cdbbd40115f07635781d330588` |

## 6. Phase 2 - Focused behavior

Run:

```powershell
cargo +1.97.1 test -p seacad-dxf-core --test entity_xdata_layer_destination_tests
cargo +1.97.1 test -p seacad-dxf-core --test entity_xdata_layer_resolution_tests --test entity_xdata_appid_destination_tests --test entity_xdata_layer_destination_tests
```

Expected: `3/3` destination-LAYER tests and `9/9` combined adjacent tests pass.
The raw test names must demonstrate all four ASCII/Binary format pairings across
all nine supported dialects, fail-closed malformed destination tables, and the
dual-source/cancellation/bounds/redaction contract.

## 7. Phase 3 - Code and contract assertions

Run:

```powershell
rg -n 'DxfEntityXDataLayerDestination(Directory|Entry|State)|entity_xdata_layer_destination_directory|NamedSymbolDestinationIndex|DxfNamedSymbolTableKind::Layer|SourceMissing|SourceAmbiguous|DestinationMissing|DestinationUnique|DestinationAmbiguous|exact_matches' crates/seacad-dxf-core/src/entity_xdata_layer_destination.rs crates/seacad-dxf-core/src/named_symbol_destination.rs crates/seacad-dxf-core/src/lib.rs
rg -n 'every_supported_version_and_format_pair|malformed_destination_layer_tables|dual_source_bound|DxfAcadVersion::SUPPORTED|Ascii|Binary|Ac1009|Ac1032|ORPHAN|NearCase|WRONG_TABLE|MULTI_NAME|UNCLOSED|4_097|SourceIdentityMismatch|Cancelled' crates/seacad-dxf-core/tests/entity_xdata_layer_destination_tests.rs
$forbidden = @(rg -n 'panic!|unwrap\(|expect\(|todo!|unimplemented!|unsafe' crates/seacad-dxf-core/src/entity_xdata_appid_destination.rs crates/seacad-dxf-core/src/entity_xdata_layer_destination.rs crates/seacad-dxf-core/src/named_symbol_destination.rs)
if ($forbidden.Count -ne 0) { $forbidden; throw 'Forbidden production construct found.' }
```

Required evidence:

- source missing/ambiguity is preserved before destination lookup;
- the destination index is exact-kind filtered and source-identity bound;
- SHA-256 only narrows candidates and byte comparison remains authoritative;
- only destination-unique state can derive an owned LAYER target;
- both documents remain immutable and dual-source identities remain exact;
- the forbidden production scan reports zero matches.

## 8. Phase 4 - Documentation and protected surfaces

Run:

```powershell
rg -n 'M14\.3dd|destination LAYER|DxfEntityXDataLayerDestinationDirectory|Apache-2.0 only|r0-exit-apache-2.0-authorization' README.md README.vi.md docs/IMPLEMENTATION_PLAN.md docs/SUPPORT_MATRIX.md docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md docs/audits/M14_3DD_ENTITY_XDATA_LAYER_DESTINATION.md
git diff -- Cargo.toml Cargo.lock LICENSE NOTICE THIRD_PARTY_NOTICES.md schema corpus release .github .agents
git diff --check
```

Expected: M14.3dd scope is documented in English and Vietnamese status surfaces,
the R0 Apache-2.0-only decision remains bound, protected-surface diff is empty,
and diff check exits zero.

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
if ($listed.Count -ne 1013) { throw 'Workspace test count drift.' }
git diff --check
```

Every command must exit zero. Expected workspace total: `1,013` tests.

## 10. Phase 6 - Postflight

Repeat Phase 1 Git commands and all receipts, the protected-surface diff, and
`git diff --check`. Initial and final HEAD, path sets, receipts, and protected
surfaces must match exactly. Only ignored `target` output is tolerated.

## 11. Acceptance criteria

The report is `PASS` only when:

1. root, baseline, exact path sets, and every artifact receipt match;
2. focused destination-LAYER and adjacent suites pass 3/3 and 9/9;
3. source-state precedence, destination exactness, and owned-target contracts
   are present and exercised;
4. all four ASCII/Binary pairings cover all nine Core dialects plus stated
   malformed, identity, cancellation, bounds, and redaction cases;
5. forbidden production scan is empty;
6. documentation, local links, and support boundaries are consistent;
7. protected repository surfaces are unchanged;
8. every required gate passes and workspace total is exactly 1,013;
9. postflight matches preflight without repository mutation;
10. the complete external report is written.

An unmet criterion is `FAIL` unless preflight requires `BLOCKED`.

## 12. Required report format

Write one UTF-8 YAML-shaped report:

```yaml
batch_id: seacad-m14.3dd-entity-xdata-layer-destination-2026-08-08
status: PASS | FAIL | BLOCKED
root: D:\SeaCad\SeaCad
report_file: D:\SeaCad\AntigravityReports\seacad-m14.3dd-entity-xdata-layer-destination-2026-08-08.yaml
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
focused_layer_destination_tests:
focused_adjacent_tests:
workspace_test_total:
links: PASS | FAIL
forbidden_production_scan: PASS | FAIL
support_matrix_unchanged_except_m14_3dd: PASS | FAIL
protected_surfaces_unchanged: PASS | FAIL
mutations: none | <exact mutation>
deviations: none | <exact deviation>
failures: none | <exact failure>
blocker: none | <exact blocker>
final_assessment: <one literal sentence>
```

After writing the report, print only its absolute path and final status.
