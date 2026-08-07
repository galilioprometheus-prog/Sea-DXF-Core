# Antigravity CLI Mechanical Verification Batch

Status: `READY`

Batch ID: `seacad-r0.1-master-plan-reset-2026-08-08`

Repository root: `D:\SeaCad\SeaCad`

Target milestone: `R0.1 - master plan reset`

Baseline commit: `76883bbdda527a828ee01136835714bfb3b4c38e`

Required report file:
`D:\SeaCad\AntigravityReports\seacad-r0.1-master-plan-reset-2026-08-08.yaml`

Prepared: `2026-08-08` (`Asia/Saigon`)

## 1. Authority and purpose

You are the mechanical verification worker for SeaCad. Verify the proposed
R0.1 documentation-only worktree and collect raw evidence. You do not own
architecture, milestone scope, normative interpretation, provenance,
dependencies, support claims, licensing, commit, tag, merge, or release
decisions. Codex and the user retain that authority.

Do not use Agent Hub, an MCP server, another agent, or an external coordination
service. This file is the complete task specification. Do not infer, repair,
add, remove, reorder, or expand work.

## 2. Mandatory read-first procedure

1. Set the working directory to exactly `D:\SeaCad\SeaCad`.
2. Read `AGENTS.md` completely.
3. Read this file completely.
4. Confirm that this file contains exactly one batch marked `READY`.
5. Execute every phase below in order unless an immediate stop condition fires.

If `AGENTS.md` conflicts with this batch, stop and report the conflict.

## 3. Allowed writes

The repository is logically read-only. Ordinary ignored compiler/test output
under the existing `target` directory and already-provisioned tool caches are
tolerated. No project source, documentation, manifest, lockfile, schema,
fixture, release artifact, Git metadata, tag, or branch may be changed.

Create or replace exactly one non-ephemeral output outside the repository:

`D:\SeaCad\AntigravityReports\seacad-r0.1-master-plan-reset-2026-08-08.yaml`

Do not create a sidecar, temporary report, repository report, or second output.
Do not install or update a tool, target, dependency, package, or runtime.

## 4. Prohibited actions

- Do not edit or fix a failure.
- Do not run a formatter in write mode.
- Do not run `cargo update` or a command that changes generated evidence.
- Do not mutate Git state with add, commit, tag, reset, restore, checkout,
  switch, stash, clean, merge, rebase, fetch, pull, push, or hosting commands.
- Do not run ODA, RealDWG, Bentley SDK, AutoCAD, MicroStation, a GUI, an MCP
  server, a scripting runtime, or a network-dependent command.
- Do not inspect external parser source or any legacy source tree.
- Do not reinterpret support, licensing, architecture, or historical receipts.

## 5. Expected preflight state

The exact root and baseline commit above are required. The tracked changed-path
set must contain exactly:

```text
AGENTS.md
README.md
README.vi.md
crates/seacad-dxf-core/src/entity_xdata_appid_destination.rs
crates/seacad-dxf-core/src/lib.rs
docs/ANTIGRAVITY_MECHANICAL_BATCH.md
docs/ARCHITECTURE.md
docs/DXF_ENTITY_COMPLETION_PLAN.md
docs/IMPLEMENTATION_PLAN.md
```

`docs/DXF_ENTITY_COMPLETION_PLAN.md` is expected to be a tracked deletion. The
exact untracked-path set must contain:

```text
crates/seacad-dxf-core/src/named_symbol_destination.rs
docs/audits/R0_1_MASTER_PLAN_RESET.md
docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md
docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md
```

The three code paths are concurrent code work outside R0.1. Their required
preflight and postflight receipts are:

| Path | Lines | SHA-256 |
| --- | ---: | --- |
| `crates/seacad-dxf-core/src/lib.rs` | 1,162 | `18d9dfec0f1d87cb719aa80122faed59faa56dbeea24e8639d4aee9919536e02` |
| `crates/seacad-dxf-core/src/entity_xdata_appid_destination.rs` | 313 | `e3c65574d17d4f8c0fddaadb9bfd945949ebe8ccf23835a62bf0c7be5571374b` |
| `crates/seacad-dxf-core/src/named_symbol_destination.rs` | 129 | `474b3f8d48af6c00d62046c059adc476dc619630424f90059be9c41694b3807d` |

Stop and report `BLOCKED` before quality commands if the root, baseline commit,
changed path sets, or either code receipt differs. A test or documentation
failure after valid preflight is `FAIL`, not permission to edit.

## 6. Phase 1 - Preflight evidence

Run each command exactly and record stdout, stderr, exit code, timestamps, and
working directory:

```powershell
Get-Location
git status --short --branch
git rev-parse HEAD
git diff --name-only
git ls-files --others --exclude-standard
```

Compare changed paths as sets, not by incidental output order. Then run:

```powershell
$userPaths = @(
  'crates/seacad-dxf-core/src/lib.rs',
  'crates/seacad-dxf-core/src/entity_xdata_appid_destination.rs',
  'crates/seacad-dxf-core/src/named_symbol_destination.rs'
)
foreach ($userPath in $userPaths) {
  [PSCustomObject]@{
    Path = $userPath
    Lines = (Get-Content -LiteralPath $userPath).Count
    Sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $userPath).Hash.ToLowerInvariant()
  }
}
```

## 7. Phase 2 - Plan preservation and navigation

Run:

```powershell
$planPaths = @(
  'docs/IMPLEMENTATION_PLAN.md',
  'docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md',
  'docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md',
  'docs/audits/R0_1_MASTER_PLAN_RESET.md'
)
foreach ($planPath in $planPaths) {
  [PSCustomObject]@{
    Path = $planPath
    Lines = (Get-Content -LiteralPath $planPath).Count
    Sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $planPath).Hash.ToLowerInvariant()
  }
}
```

Required receipts:

| Path | Lines | SHA-256 |
| --- | ---: | --- |
| `docs/IMPLEMENTATION_PLAN.md` | 413 | `28193ac61e1a2222e26d30d31063df09425c7827f0797af56583b8131bb87baf` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 2,977 | `ceef1e69d38a0160979d875f6c9955e02bf943a17f98d4ca11698bf302fcf706` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 1,550 | `c3c622870f0176a3138beba293f7013cdcd41279451b0a2942ec78ff22ca1584` |
| `docs/audits/R0_1_MASTER_PLAN_RESET.md` | 86 | `c9c2bb9ea31ab1e870b02366eb8ed7c964d9c852b50a0b5cca6ea3c6a7058f16` |

Validate local Markdown links in the R0.1 document set with this exact block:

```powershell
$files = @(
  'AGENTS.md',
  'README.md',
  'README.vi.md',
  'docs/ARCHITECTURE.md',
  'docs/IMPLEMENTATION_PLAN.md',
  'docs/audits/R0_1_MASTER_PLAN_RESET.md',
  'docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md',
  'docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md'
)
$failures = @()
foreach ($file in $files) {
  $directory = Split-Path -Parent (Resolve-Path -LiteralPath $file)
  $lineNumber = 0
  foreach ($line in Get-Content -LiteralPath $file) {
    $lineNumber++
    foreach ($match in [regex]::Matches($line, '\]\(([^)]+)\)')) {
      $target = $match.Groups[1].Value
      if ($target -match '^(https?://|mailto:|#)') { continue }
      $pathPart = ($target -split '#', 2)[0].Trim('<','>')
      if ([string]::IsNullOrWhiteSpace($pathPart)) { continue }
      $resolved = Join-Path $directory $pathPart
      if (-not (Test-Path -LiteralPath $resolved)) {
        $failures += [PSCustomObject]@{
          File = $file
          Line = $lineNumber
          Target = $target
          Resolved = $resolved
        }
      }
    }
  }
}
if ($failures.Count -gt 0) {
  $failures | Format-Table -AutoSize
  exit 1
}
'All local Markdown links in the R0.1 document set resolve.'
```

Expected output is the single success sentence and exit `0`.

Run the stale live-link scan:

```powershell
rg -n "docs/DXF_ENTITY_COMPLETION_PLAN\.md" AGENTS.md README.md README.vi.md docs -g '*.md' -g '!docs/audits/**' -g '!docs/ANTIGRAVITY_MECHANICAL_BATCH.md'
```

Expected result is no output and exit `1`. Historical audit receipts are
excluded intentionally and must remain unchanged.

## 8. Phase 3 - Documentation and support boundaries

Run:

```powershell
rg -n "R0\.1|M14\.3dc|DXF Core 1\.0|support matrix|MIT OR Apache-2\.0|seacad\.theme/v1|English|Vietnamese|BCP 47|ODA|RealDWG|Bentley SDK" AGENTS.md README.md README.vi.md docs/ARCHITECTURE.md docs/IMPLEMENTATION_PLAN.md docs/audits/R0_1_MASTER_PLAN_RESET.md
git diff -- docs/SUPPORT_MATRIX.md Cargo.toml Cargo.lock crates schema release
git diff --check
```

Required evidence:

- The master plan identifies R0.1 and points to both preserved DXF subplans.
- M14.3dc remains the documented completed checkpoint.
- `docs/SUPPORT_MATRIX.md` remains normative and unchanged.
- The open-source license is described only as a future audited target; the
  current proprietary license remains authoritative.
- English is the canonical fallback, Vietnamese is a first-class locale, later
  languages use normalized BCP 47 catalogs, and stable protocol/document
  identifiers remain locale-neutral.
- DWG, DGN, renderer, themes, GUI, plugins, AI, and B-rep are planned, not
  claimed implemented.
- The scoped non-document diff command emits only the concurrent code-work
  diff, with no manifest, lockfile, schema, fixture, or release change.
- `git diff --check` exits `0` with no error output.

## 9. Phase 4 - Required repository gates

Run in this exact order:

```powershell
cargo deny --locked check
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 run --locked -p seacad-schema-gen -- --check
cargo +1.97.1 run --locked -p seacad-schema-gen --bin seacad-release-evidence -- --check
cargo +1.97.1 clippy --workspace --all-targets -- -D warnings
cargo +1.97.1 test --workspace
git diff --check
```

Every command must exit `0`. Formatting and generators are check-only. Record
the exact observed workspace test count; do not guess or repair a mismatch.

## 10. Phase 5 - Postflight mutation audit

Repeat the Phase 1 Git commands and user-code receipt block. Then repeat the
Phase 2 plan-receipt block. Initial and final `HEAD`, changed path sets, user
code hashes, and plan hashes must match exactly. List any new ignored output
under `target` as tolerated ephemeral output.

## 11. Batch acceptance criteria

The batch is `PASS` only when:

1. Preflight root, commit, path sets, and user-code receipts match.
2. Both preserved DXF plans and the new master plan match their receipts.
3. Every checked local Markdown link resolves and the stale live-link scan is
   clean with expected exit `1`.
4. Support matrix, production Rust, manifests, lockfile, schemas, fixtures, and
   release evidence receive no R0.1 mutation.
5. Every required quality gate passes.
6. Postflight matches preflight and no repository mutation occurs.
7. The complete report is written to the exact required external path.

An unmet criterion is `FAIL` unless an explicit immediate stop condition makes
the result `BLOCKED`.

## 12. Required report format

Write one UTF-8 YAML-shaped report with no prose before or after it:

```yaml
batch_id: seacad-r0.1-master-plan-reset-2026-08-08
status: PASS | FAIL | BLOCKED
root: D:\SeaCad\SeaCad
report_file: D:\SeaCad\AntigravityReports\seacad-r0.1-master-plan-reset-2026-08-08.yaml
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
    result: PASS | FAIL | EXPECTED_NO_MATCH | BLOCKED
hash_receipts:
  - path:
    observed_lines:
    expected_lines:
    observed_sha256:
    expected_sha256:
    result: PASS | FAIL
link_validation:
stale_link_scan:
support_matrix_unchanged:
non_document_scope_unchanged:
workspace_test_total:
mutations: none | <exact list>
tolerated_ephemeral_outputs: none | <exact list>
deviations: none | <exact list>
retries: none | <exact list>
failures: none | <exact list>
blocker: none | <exact blocker>
final_assessment: <one literal sentence explaining PASS, FAIL, or BLOCKED>
```

After closing the report, print only its absolute path and final status to the
Antigravity chat. Chat output without the report file is not delivery.
