# Antigravity CLI Mechanical Verification Batch

Status: `READY`

Batch ID: `seacad-r0.3-public-formats-workspace-export-design-2026-08-08`

Repository root: `D:\SeaCad\SeaCad`

Target milestone: `R0.3 - public seacad-formats workspace export design`

Baseline commit: `a0e5ec66b856d22dc278187da5937b17f8e083ba`

Required report file:
`D:\SeaCad\AntigravityReports\seacad-r0.3-public-formats-workspace-export-design-2026-08-08.yaml`

Prepared: `2026-08-08` (`Asia/Saigon`)

## 1. Authority and purpose

You are the mechanical verification worker for SeaCad. Verify the proposed
R0.3 design-only workspace export contract and collect raw evidence. You do not
approve ownership, licensing, architecture, support, publication, commit, tag,
merge, or release decisions. Codex and the user retain that authority.

R0.3 must not create an export tree or activate an open license. The current
SeaCad Proprietary License remains authoritative.

## 2. Mandatory procedure

1. Use exactly `D:\SeaCad\SeaCad` as the working directory.
2. Read `AGENTS.md` completely.
3. Read this batch completely.
4. Confirm exactly one batch is marked `READY`.
5. Execute every phase in order unless a preflight stop condition fires.

If `AGENTS.md` conflicts with this batch, stop and report the conflict.

## 3. Allowed writes and prohibited actions

The repository is read-only. Ignored build output under existing `target` and
provisioned tool caches is tolerated. Create or replace exactly one persistent
file outside the repository:

`D:\SeaCad\AntigravityReports\seacad-r0.3-public-formats-workspace-export-design-2026-08-08.yaml`

Do not edit, format in write mode, regenerate, export, copy source, create an
archive, install/update tools, mutate Git, access a private corpus or legacy
tree, run vendor CAD software, use network commands, or infer legal authority.

## 4. Expected preflight state

The tracked changed-path set must contain exactly:

```text
crates/seacad-dxf-core/src/entity_xdata_appid_destination.rs
crates/seacad-dxf-core/src/lib.rs
docs/ANTIGRAVITY_MECHANICAL_BATCH.md
docs/IMPLEMENTATION_PLAN.md
```

The exact untracked-path set must contain:

```text
crates/seacad-dxf-core/src/named_symbol_destination.rs
docs/audits/R0_3_PUBLIC_FORMATS_WORKSPACE_EXPORT_DESIGN.md
```

The three code paths are concurrent DXF work outside R0.3:

| Path | Lines | SHA-256 |
| --- | ---: | --- |
| `crates/seacad-dxf-core/src/lib.rs` | 1,162 | `18d9dfec0f1d87cb719aa80122faed59faa56dbeea24e8639d4aee9919536e02` |
| `crates/seacad-dxf-core/src/entity_xdata_appid_destination.rs` | 313 | `e3c65574d17d4f8c0fddaadb9bfd945949ebe8ccf23835a62bf0c7be5571374b` |
| `crates/seacad-dxf-core/src/named_symbol_destination.rs` | 129 | `474b3f8d48af6c00d62046c059adc476dc619630424f90059be9c41694b3807d` |

Stop `BLOCKED` before quality commands if root, baseline commit, path sets, or
code receipts differ.

## 5. Phase 1 - Preflight

Run and record:

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

## 6. Phase 2 - Design document receipts and links

Run:

```powershell
$paths = @(
  'docs/IMPLEMENTATION_PLAN.md',
  'docs/audits/R0_3_PUBLIC_FORMATS_WORKSPACE_EXPORT_DESIGN.md'
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
| `docs/IMPLEMENTATION_PLAN.md` | 419 | `ce3ef316c5f78fa351665c63e0a31cdd658464dcb3c28688bbc973ad03a513cb` |
| `docs/audits/R0_3_PUBLIC_FORMATS_WORKSPACE_EXPORT_DESIGN.md` | 375 | `3a97c6362137335502d4fa3e4fde7f94f9fb9c9e0eb454f37cc9d5125392b9ec` |

Validate every non-HTTP, non-mail, non-anchor Markdown link in both files
relative to its containing file. Expected: no missing link and exit `0`.

## 7. Phase 3 - Source/layout evidence

Run:

```powershell
$core = @(git ls-files 'crates/seacad-dxf-core/**')
$schema = @(git ls-files 'schema/dxf/v1/**')
$generator = @(git ls-files 'crates/seacad-schema-gen/**')
$coreBytes = ($core | ForEach-Object { (Get-Item -LiteralPath $_).Length } | Measure-Object -Sum).Sum
$schemaBytes = ($schema | ForEach-Object { (Get-Item -LiteralPath $_).Length } | Measure-Object -Sum).Sum
$generatorBytes = ($generator | ForEach-Object { (Get-Item -LiteralPath $_).Length } | Measure-Object -Sum).Sum
[PSCustomObject]@{
  CoreFiles = $core.Count
  CoreBytes = $coreBytes
  SchemaFiles = $schema.Count
  SchemaBytes = $schemaBytes
  GeneratorFiles = $generator.Count
  GeneratorBytes = $generatorBytes
}
if ($core.Count -ne 433 -or $coreBytes -ne 6240044 -or
    $schema.Count -ne 7 -or $schemaBytes -ne 103938 -or
    $generator.Count -ne 5 -or $generatorBytes -ne 207705) {
  throw 'R0.3 source/layout count drift.'
}
$generator | Sort-Object
$main = 'crates/seacad-schema-gen/src/main.rs'
"generator_main_lines=$((Get-Content -LiteralPath $main).Count)"
"generator_main_bytes=$((Get-Item -LiteralPath $main).Length)"
"generator_main_sha256=$((Get-FileHash -Algorithm SHA256 -LiteralPath $main).Hash.ToLowerInvariant())"
rg -n 'const (MANIFEST|HEADER_OUTPUT|ENTITY_OUTPUT)_PATH|CARGO_MANIFEST_DIR|name = "seacad-(schema-gen|release-evidence|release-packager|release-receipts)"' crates/seacad-schema-gen
```

Required evidence:

- core `433` files / `6,240,044` bytes;
- schema `7` files / `103,938` bytes;
- mixed generator `5` files / `207,705` bytes;
- generator main `3,006` lines / `118,048` bytes / SHA-256
  `4c6f5074fd5f7654d9ca74fe3f56d1386c3f4ffd75519e6ff52026f70c96fdb2`;
- one schema generator and three separately named release binaries;
- root resolution remains `CARGO_MANIFEST_DIR/../..` with paths under
  `schema/dxf/v1` and `crates/seacad-dxf-core/src/generated`.

## 8. Phase 4 - Dependency closure evidence

Run this read-only metadata traversal:

```powershell
$meta = cargo metadata --locked --format-version 1 | ConvertFrom-Json
$byId = @{}
foreach ($package in $meta.packages) { $byId[$package.id] = $package }
$nodes = @{}
foreach ($node in $meta.resolve.nodes) { $nodes[$node.id] = $node }
$roots = @($meta.packages | Where-Object {
  $_.name -in @('seacad-dxf-core', 'seacad-schema-gen')
} | ForEach-Object { $_.id })
$seen = @{}
$queue = [Collections.Generic.Queue[string]]::new()
foreach ($id in $roots) { $queue.Enqueue($id) }
while ($queue.Count -gt 0) {
  $id = $queue.Dequeue()
  if ($seen.ContainsKey($id)) { continue }
  $seen[$id] = $true
  foreach ($dependency in $nodes[$id].dependencies) { $queue.Enqueue($dependency) }
}
$included = @($seen.Keys | ForEach-Object { $byId[$_] } | Sort-Object name)
$excluded = @($meta.packages | Where-Object { -not $seen.ContainsKey($_.id) } | Sort-Object name)
"included_count=$($included.Count)"
$included | ForEach-Object { "$($_.name)|$($_.version)|$($_.license)" }
"excluded_count=$($excluded.Count)"
$excluded | ForEach-Object { "$($_.name)|$($_.version)" }
$expectedExcluded = @('anstyle', 'clap', 'clap_builder', 'clap_lex', 'seacad-cli', 'strsim')
if ($included.Count -ne 23 -or
    @(Compare-Object $expectedExcluded @($excluded.name)).Count -ne 0) {
  throw 'R0.3 dependency closure drift.'
}
```

Expected inclusion count `23`. Expected excluded set exactly:
`anstyle`, `clap`, `clap_builder`, `clap_lex`, `seacad-cli`, and `strsim`.

## 9. Phase 5 - Generated, docs, CI, and non-authorization evidence

Run:

```powershell
$paths = @(
  'crates/seacad-dxf-core/src/generated/header_schema.rs',
  'crates/seacad-dxf-core/src/generated/entity_schema.rs'
)
foreach ($path in $paths) {
  "$path|$((Get-FileHash -Algorithm SHA256 -LiteralPath $path).Hash.ToLowerInvariant())"
}
$expectedGenerated = @(
  '34675e171859d3075f70344e80c96217ed7994c7dd4e38b6bf09de33b6699b7c',
  '0872b9e0bb85361a549fc77ee5265d05d4420cc8796ddb75ffd2ce6c83eaa610'
)
for ($index = 0; $index -lt $paths.Count; $index++) {
  $observed = (Get-FileHash -Algorithm SHA256 -LiteralPath $paths[$index]).Hash.ToLowerInvariant()
  if ($observed -ne $expectedGenerated[$index]) { throw 'R0.3 generated output drift.' }
}
$auditCount = @(git ls-files 'docs/audits/*.md').Count
"audit_count=$auditCount"
if ($auditCount -ne 339) { throw 'R0.3 committed audit count drift.' }
rg -n 'exactly two workspace members|seacad-dxf-schema-gen|23 packages|21 third-party|excluded exactly|all 339|Linux x64|Windows x64|macOS x64|publish = false|does not authorize|prohibited until|Proprietary' docs/audits/R0_3_PUBLIC_FORMATS_WORKSPACE_EXPORT_DESIGN.md
git ls-files '.github/workflows/**'
git diff -- docs/SUPPORT_MATRIX.md Cargo.toml Cargo.lock LICENSE NOTICE THIRD_PARTY_NOTICES.md crates schema corpus release .github .agents
git diff --check
```

Required generated hashes:

- header: `34675e171859d3075f70344e80c96217ed7994c7dd4e38b6bf09de33b6699b7c`;
- entity: `0872b9e0bb85361a549fc77ee5265d05d4420cc8796ddb75ffd2ce6c83eaa610`.

Expected current audit count: `339`. The design must specify two members, 23
SBOM components, 21 legal packages, exact CLI exclusions, six public native
targets, `publish = false`, and explicit non-authorization. The scoped diff may
contain only the three concurrent DXF code paths. No support matrix, manifest,
lockfile, license, schema, corpus, release, workflow, or skill mutation is
allowed.

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

Every command must exit `0`. Record the exact workspace test count.

## 11. Phase 7 - Postflight

Repeat Phase 1 Git commands/receipts, Phase 2 document receipts, Phase 3
counts/hash, Phase 4 closure, and Phase 5 generated hashes/audit count. Initial
and final state must match exactly. Only ignored `target` output is tolerated.

## 12. Acceptance criteria

The result is `PASS` only if:

1. root, baseline, path sets, and concurrent-code receipts match;
2. design receipts and links match;
3. source/layout counts, generator targets, and path assumptions match;
4. public closure is exactly 23 with the exact six exclusions;
5. generated hashes and 339-audit exclusion evidence match;
6. design explicitly grants no license and performs no export/publication;
7. protected repository surfaces are unchanged;
8. all quality gates pass;
9. postflight matches preflight;
10. the complete external report is written.

An unmet criterion is `FAIL` unless preflight requires `BLOCKED`.

## 13. Required report format

Write one UTF-8 YAML-shaped report:

```yaml
batch_id: seacad-r0.3-public-formats-workspace-export-design-2026-08-08
status: PASS | FAIL | BLOCKED
root: D:\SeaCad\SeaCad
report_file: D:\SeaCad\AntigravityReports\seacad-r0.3-public-formats-workspace-export-design-2026-08-08.yaml
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
core_file_count:
core_bytes:
schema_file_count:
schema_bytes:
generator_file_count:
generator_bytes:
public_dependency_count:
excluded_dependencies:
current_audit_count:
workspace_test_total:
links: PASS | FAIL
license_activated: false
export_created: false
support_matrix_unchanged: PASS | FAIL
protected_surfaces_unchanged: PASS | FAIL
mutations: none | <exact list>
deviations: none | <exact list>
failures: none | <exact list>
blocker: none | <exact blocker>
final_assessment: <one literal sentence>
```

After writing the report, print only its absolute path and final status to the
Antigravity chat. Chat output without the file is not delivery.
