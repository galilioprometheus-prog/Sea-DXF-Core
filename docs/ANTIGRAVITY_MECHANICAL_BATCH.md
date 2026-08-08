# Antigravity CLI Mechanical Verification Batch

Status: `READY`

Batch ID: `seacad-r0-exit-apache-2.0-authorization-2026-08-08`

Repository root: `D:\SeaCad\SeaCad`

Target milestone: `R0 exit - Apache-2.0 authorization`

Baseline commit: `dafd6a5882ea7a3360cbc503d64ce75f8123ead1`

Required report file:
`D:\SeaCad\AntigravityReports\seacad-r0-exit-apache-2.0-authorization-2026-08-08.yaml`

Prepared: `2026-08-08` (`Asia/Saigon`)

## 1. Authority and purpose

You are the mechanical verification worker for SeaCad. Verify that the R0 exit
decision records Apache-2.0 only for the exact future R0.3 public boundary while
leaving the current mixed repository proprietary and unchanged outside two
decision documents.

Do not provide legal advice, infer ownership, expand the authorized boundary,
activate a license, create an export, or decide commit/tag/release actions.

## 2. Mandatory procedure

1. Use exactly `D:\SeaCad\SeaCad` as working directory.
2. Read `AGENTS.md` completely.
3. Read this batch completely.
4. Confirm exactly one batch is marked `READY`.
5. Execute every phase in order unless preflight requires `BLOCKED`.

## 3. Allowed writes and prohibited actions

The repository is read-only. Ignored `target` output and provisioned tool-cache
activity are tolerated. Create or replace exactly this external report:

`D:\SeaCad\AntigravityReports\seacad-r0-exit-apache-2.0-authorization-2026-08-08.yaml`

Do not edit, format/regenerate in write mode, copy source, create a public
workspace/archive/license file, mutate Git, install/update tools, use a network,
read legacy/private corpus content, or run vendor CAD software.

## 4. Expected preflight state

Tracked changed paths exactly:

```text
crates/seacad-dxf-core/src/entity_xdata_appid_destination.rs
crates/seacad-dxf-core/src/lib.rs
docs/ANTIGRAVITY_MECHANICAL_BATCH.md
docs/IMPLEMENTATION_PLAN.md
```

Untracked paths exactly:

```text
crates/seacad-dxf-core/src/named_symbol_destination.rs
docs/audits/R0_EXIT_APACHE_2_0_AUTHORIZATION.md
```

Concurrent DXF code receipts:

| Path | Lines | SHA-256 |
| --- | ---: | --- |
| `crates/seacad-dxf-core/src/lib.rs` | 1,162 | `18d9dfec0f1d87cb719aa80122faed59faa56dbeea24e8639d4aee9919536e02` |
| `crates/seacad-dxf-core/src/entity_xdata_appid_destination.rs` | 313 | `e3c65574d17d4f8c0fddaadb9bfd945949ebe8ccf23835a62bf0c7be5571374b` |
| `crates/seacad-dxf-core/src/named_symbol_destination.rs` | 129 | `474b3f8d48af6c00d62046c059adc476dc619630424f90059be9c41694b3807d` |

Stop `BLOCKED` before quality commands if root, baseline, path sets, or these
receipts differ.

## 5. Phase 1 - Preflight

Run:

```powershell
Get-Location
git status --short --branch
git rev-parse HEAD
git diff --name-only
git ls-files --others --exclude-standard
$paths = @(
  'crates/seacad-dxf-core/src/lib.rs',
  'crates/seacad-dxf-core/src/entity_xdata_appid_destination.rs',
  'crates/seacad-dxf-core/src/named_symbol_destination.rs'
)
foreach ($path in $paths) {
  [PSCustomObject]@{
    Path = $path
    Lines = (Get-Content -LiteralPath $path).Count
    Sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $path).Hash.ToLowerInvariant()
  }
}
```

Compare changed paths as sets.

## 6. Phase 2 - Decision receipts and links

Run:

```powershell
$paths = @(
  'docs/IMPLEMENTATION_PLAN.md',
  'docs/audits/R0_EXIT_APACHE_2_0_AUTHORIZATION.md'
)
foreach ($path in $paths) {
  [PSCustomObject]@{
    Path = $path
    Lines = (Get-Content -LiteralPath $path).Count
    Sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $path).Hash.ToLowerInvariant()
  }
}
```

Required receipts:

| Path | Lines | SHA-256 |
| --- | ---: | --- |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `85818190f3cc1b8e7c124c17cd432233c069fae2c63c75800b0784045686cfee` |
| `docs/audits/R0_EXIT_APACHE_2_0_AUTHORIZATION.md` | 156 | `1c935154908acaf1cba22cebf032d6291c106250c156f4ef99f4281f9449584a` |

Validate every non-HTTP, non-mail, non-anchor Markdown link in both files
relative to its containing file. Missing links fail.

## 7. Phase 3 - Apache-only scope assertions

Run:

```powershell
$decision = Get-Content -Raw docs/audits/R0_EXIT_APACHE_2_0_AUTHORIZATION.md
$required = @(
  'Apache License 2.0 only',
  'rejects the earlier tentative `MIT OR Apache-2.0` target',
  'The current mixed private repository remains under',
  'SeaCad Proprietary',
  'authorization applies only to the future allowlist-derived',
  'does not apply to',
  'No public repository, crate, package, archive, installer, binary, tag, or',
  'declare `license = "Apache-2.0"`',
  'No SeaCad-authored public export may offer an MIT alternative'
)
foreach ($text in $required) {
  if (-not $decision.Contains($text)) { throw "Missing R0 exit decision text: $text" }
}
$master = Get-Content -Raw docs/IMPLEMENTATION_PLAN.md
if (-not $master.Contains('Apache-2.0 only') -or
    -not $master.Contains('R0_EXIT_APACHE_2_0_AUTHORIZATION.md')) {
  throw 'Master plan does not bind the Apache-2.0-only R0 exit decision.'
}
if ($master.Contains('MIT OR Apache-2.0')) {
  throw 'Live master plan retains superseded dual-license target.'
}
'Apache-2.0-only decision scope assertions passed.'
```

The historical audit is allowed to mention the superseded dual-license option
only to reject it. The live master plan must not offer it.

## 8. Phase 4 - Current license/non-export assertions

Run:

```powershell
$paths = @('LICENSE', 'Cargo.toml', 'Cargo.lock', 'NOTICE', 'THIRD_PARTY_NOTICES.md')
foreach ($path in $paths) {
  "$path|$((Get-FileHash -Algorithm SHA256 -LiteralPath $path).Hash.ToLowerInvariant())"
}
Get-Content -Raw LICENSE
rg -n '^publish = false$|^license-file\.workspace = true$' crates -g Cargo.toml
$forbidden = @(
  'LICENSE-APACHE',
  'LICENSE-MIT',
  'seacad-formats',
  'public/seacad-formats',
  'dist/seacad-formats'
)
foreach ($path in $forbidden) {
  if (Test-Path -LiteralPath $path) { throw "Unexpected license/export path: $path" }
}
'No Apache/MIT project license or public export path exists.'
```

Required hashes:

| Path | SHA-256 |
| --- | --- |
| `LICENSE` | `131b152ed468c95e5cd12f97a5dbeabffc73a20f4dd0c6d5d9c16dc619480b1d` |
| `Cargo.toml` | `45bf631ead2fd0681597a2d313976d79ac22d6088d0d97a451a54dcf42080259` |
| `Cargo.lock` | `f48f459ed5a7c7b9fb9d184d78099c5c6b31b260b3d13df632dbf539ea07ad70` |
| `NOTICE` | `4b7ab578a58970d42a8ff9d2b81133eab04c901ebbee7d0db6ebd823197a3cdb` |
| `THIRD_PARTY_NOTICES.md` | `47aab18e969b87c863a404998970e187e5d180c6395be11f2cc442a40dda86aa` |

Expected: proprietary root license, three `publish = false`, three inherited
license-file declarations, and no forbidden path.

## 9. Phase 5 - Protected-surface review

Run:

```powershell
git diff -- docs/SUPPORT_MATRIX.md Cargo.toml Cargo.lock LICENSE NOTICE THIRD_PARTY_NOTICES.md crates schema corpus release .github .agents
git diff --check
```

The scoped diff may contain only the two tracked concurrent DXF code diffs.
Untracked `named_symbol_destination.rs` remains separately hash-bound. No
support matrix, manifest, lockfile, legal file, schema, corpus, release,
workflow, or agent mutation is allowed.

## 10. Phase 6 - Required gates

Run in order:

```powershell
cargo deny --locked check
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 run --locked -p seacad-schema-gen -- --check
cargo +1.97.1 run --locked -p seacad-schema-gen --bin seacad-release-evidence -- --check
cargo +1.97.1 clippy --workspace --all-targets -- -D warnings
cargo +1.97.1 test --workspace
git diff --check
```

Every command must exit `0`. Record exact workspace test count.

## 11. Phase 7 - Postflight

Repeat Phase 1, Phase 2, Phase 3, Phase 4 hashes/assertions, and the protected
surface review. Initial and final HEAD, path sets, receipts, and license state
must match exactly. Only ignored `target` output is tolerated.

## 12. Acceptance criteria

The report is `PASS` only when:

1. preflight baseline/path sets/concurrent receipts match;
2. decision document hashes and links match;
3. Apache-2.0-only scope assertions pass and the master rejects dual-license;
4. current proprietary license/manifests/lockfile/notices are unchanged;
5. no Apache/MIT project license or export path is created;
6. protected repository surfaces are unchanged;
7. all repository gates pass;
8. postflight matches preflight without mutation;
9. the complete external report is written.

An unmet criterion is `FAIL` unless preflight requires `BLOCKED`.

## 13. Required report format

Write one UTF-8 YAML-shaped report:

```yaml
batch_id: seacad-r0-exit-apache-2.0-authorization-2026-08-08
status: PASS | FAIL | BLOCKED
root: D:\SeaCad\SeaCad
report_file: D:\SeaCad\AntigravityReports\seacad-r0-exit-apache-2.0-authorization-2026-08-08.yaml
started_at:
finished_at:
head_before:
head_after:
git_before: |
  <verbatim>
git_after: |
  <verbatim>
changed_paths_before:
  - <path>
changed_paths_after:
  - <path>
commands:
  - ordinal:
    phase:
    command:
    cwd:
    started_at:
    finished_at:
    exit_code:
    stdout: |
      <verbatim or empty>
    stderr: |
      <verbatim or empty>
    result: PASS | FAIL | BLOCKED
hash_receipts:
  - path:
    observed_lines:
    expected_lines:
    observed_sha256:
    expected_sha256:
    result: PASS | FAIL
workspace_test_total:
links: PASS | FAIL
authorized_future_license: Apache-2.0
mit_alternative_authorized: false
current_repository_license: SeaCad Proprietary License
license_files_created: false
export_created: false
support_matrix_unchanged: PASS | FAIL
protected_surfaces_unchanged: PASS | FAIL
mutations: none | <exact list>
deviations: none | <exact list>
failures: none | <exact list>
blocker: none | <exact blocker>
final_assessment: <one literal sentence>
```

After writing the report, print only its absolute path and final status.
