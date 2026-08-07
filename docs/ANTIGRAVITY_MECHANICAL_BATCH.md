# Antigravity CLI Mechanical Verification Batch

Status: `READY`

Batch ID: `seacad-r0.2-provenance-ownership-boundary-2026-08-08`

Repository root: `D:\SeaCad\SeaCad`

Target milestone: `R0.2 - provenance, ownership, and boundary inventory`

Baseline commit: `a84d1985958d58949326f2c857d4e3e8e4e51210`

Required report file:
`D:\SeaCad\AntigravityReports\seacad-r0.2-provenance-ownership-boundary-2026-08-08.yaml`

Prepared: `2026-08-08` (`Asia/Saigon`)

## 1. Authority and purpose

You are the mechanical verification worker for SeaCad. Verify the proposed
R0.2 engineering inventory and collect raw evidence. You do not decide legal
ownership, relicensing authority, public/private architecture, support claims,
milestone scope, provenance interpretation, commit, tag, merge, or release.
Codex and the user retain those decisions.

This inventory explicitly does not activate an open-source license. The current
SeaCad Proprietary License remains authoritative.

## 2. Mandatory read-first procedure

1. Set the working directory to exactly `D:\SeaCad\SeaCad`.
2. Read `AGENTS.md` completely.
3. Read this file completely.
4. Confirm this file contains exactly one batch marked `READY`.
5. Execute every phase below in order unless an immediate stop condition fires.

If `AGENTS.md` conflicts with this batch, stop and report the conflict.

## 3. Allowed writes

The repository is logically read-only. Ordinary ignored compiler/test output
under the existing `target` directory and provisioned tool caches are tolerated.
No source, documentation, manifest, lockfile, schema, fixture, legal artifact,
release artifact, Git metadata, tag, or branch may be changed.

Create or replace exactly one non-ephemeral output outside the repository:

`D:\SeaCad\AntigravityReports\seacad-r0.2-provenance-ownership-boundary-2026-08-08.yaml`

Do not create a sidecar, temporary report, repository report, or second output.
Do not install or update a tool, target, dependency, package, or runtime.

## 4. Prohibited actions

- Do not edit or fix a failure.
- Do not run a formatter or generator in write mode.
- Do not run `cargo update`.
- Do not mutate Git with add, commit, tag, reset, restore, checkout, switch,
  stash, clean, merge, rebase, fetch, pull, push, or hosting commands.
- Do not inspect a legacy source tree, private corpus, Cargo registry source, or
  external parser implementation.
- Do not run AutoCAD, MicroStation, ODA, RealDWG, a GUI, an MCP server, or a
  network-dependent command.
- Do not infer a legal owner or approve `MIT OR Apache-2.0`.

## 5. Expected preflight state

The exact root and baseline commit above are required. The tracked changed-path
set must contain exactly:

```text
crates/seacad-dxf-core/src/entity_xdata_appid_destination.rs
crates/seacad-dxf-core/src/lib.rs
docs/ANTIGRAVITY_MECHANICAL_BATCH.md
docs/IMPLEMENTATION_PLAN.md
```

The exact untracked-path set must contain:

```text
crates/seacad-dxf-core/src/named_symbol_destination.rs
docs/audits/R0_2_PROVENANCE_OWNERSHIP_BOUNDARY.md
```

The three code paths are concurrent DXF work outside R0.2. Their required
preflight and postflight receipts are:

| Path | Lines | SHA-256 |
| --- | ---: | --- |
| `crates/seacad-dxf-core/src/lib.rs` | 1,162 | `18d9dfec0f1d87cb719aa80122faed59faa56dbeea24e8639d4aee9919536e02` |
| `crates/seacad-dxf-core/src/entity_xdata_appid_destination.rs` | 313 | `e3c65574d17d4f8c0fddaadb9bfd945949ebe8ccf23835a62bf0c7be5571374b` |
| `crates/seacad-dxf-core/src/named_symbol_destination.rs` | 129 | `474b3f8d48af6c00d62046c059adc476dc619630424f90059be9c41694b3807d` |

Stop and report `BLOCKED` before quality commands if the root, baseline commit,
changed path sets, or code receipts differ.

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

Compare changed paths as sets. Then run:

```powershell
$concurrentPaths = @(
  'crates/seacad-dxf-core/src/lib.rs',
  'crates/seacad-dxf-core/src/entity_xdata_appid_destination.rs',
  'crates/seacad-dxf-core/src/named_symbol_destination.rs'
)
foreach ($path in $concurrentPaths) {
  [PSCustomObject]@{
    Path = $path
    Lines = (Get-Content -LiteralPath $path).Count
    Sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $path).Hash.ToLowerInvariant()
  }
}
```

## 7. Phase 2 - R0.2 document receipts and links

Run:

```powershell
$paths = @(
  'docs/IMPLEMENTATION_PLAN.md',
  'docs/audits/R0_2_PROVENANCE_OWNERSHIP_BOUNDARY.md'
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
| `docs/IMPLEMENTATION_PLAN.md` | 417 | `cbfee301fceb02b83b3c8deee51042460d53dd41656c30f2220f1b527dd1c65e` |
| `docs/audits/R0_2_PROVENANCE_OWNERSHIP_BOUNDARY.md` | 237 | `9d504a797b58dcc73c631365af6db7f696d49b0dc565af3d413ab0e0ed59de3e` |

Validate local Markdown links in both files. Resolve each non-HTTP, non-mail,
non-anchor target relative to its containing file. Expected result: no missing
target and exit `0`.

## 8. Phase 3 - Git ownership evidence

Run:

```powershell
git rev-list --count HEAD
git shortlog -sne HEAD
git log HEAD --format='%aN|%aE' | Sort-Object -Unique
git log HEAD --format='%cN|%cE' | Sort-Object -Unique
git log HEAD --format='%G?' | Group-Object | Select-Object Name,Count
$signedOff = @(git log HEAD --format='%B' | Select-String -Pattern '^Signed-off-by:' -CaseSensitive).Count
"signed_off_by_count=$signedOff"
$rightsPaths = @(git ls-tree -r --name-only HEAD | Where-Object { $_ -match '(^|/)(CLA|DCO|CONTRIBUTING|COPYRIGHT|AUTHORS)(\.|$)' })
"rights_artifact_path_count=$($rightsPaths.Count)"
$rightsPaths
$tracked = @(git ls-tree -r --name-only HEAD)
"tracked_path_count=$($tracked.Count)"
$cad = @($tracked | Where-Object { $_ -match '\.(dxf|dxb|dwg|dgn)$' })
"tracked_cad_path_count=$($cad.Count)"
$cad
```

Required evidence:

- commit count `363`;
- one author and one committer identity, both exactly
  `SeaCad|209146803+seaflower205@users.noreply.github.com`;
- signature state exactly `N: 363`;
- zero `Signed-off-by` trailers;
- zero tracked CLA, DCO, CONTRIBUTING, COPYRIGHT, or AUTHORS artifacts;
- 907 tracked paths;
- zero tracked `.dxf`, `.dxb`, `.dwg`, or `.dgn` paths.

These are metadata observations only. Do not infer legal ownership.

## 9. Phase 4 - License, generated data, and dependency evidence

Run:

```powershell
$paths = @(
  'LICENSE',
  'NOTICE',
  'THIRD_PARTY_NOTICES.md',
  'Cargo.toml',
  'Cargo.lock',
  'crates/seacad-dxf-core/src/johab_decode_le.bin',
  'schema/dxf/v1/manifest.json',
  'schema/dxf/v1/sources.json',
  'crates/seacad-dxf-core/src/generated/header_schema.rs',
  'crates/seacad-dxf-core/src/generated/entity_schema.rs',
  'release/sbom.cdx.json',
  'release/legal/manifest.json',
  'audits/m1/legacy-dxf-fixtures.csv'
)
foreach ($path in $paths) {
  [PSCustomObject]@{
    Path = $path
    Bytes = (Get-Item -LiteralPath $path).Length
    Sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $path).Hash.ToLowerInvariant()
  }
}
```

Required SHA-256 values:

| Path | SHA-256 |
| --- | --- |
| `LICENSE` | `131b152ed468c95e5cd12f97a5dbeabffc73a20f4dd0c6d5d9c16dc619480b1d` |
| `NOTICE` | `4b7ab578a58970d42a8ff9d2b81133eab04c901ebbee7d0db6ebd823197a3cdb` |
| `THIRD_PARTY_NOTICES.md` | `47aab18e969b87c863a404998970e187e5d180c6395be11f2cc442a40dda86aa` |
| `Cargo.toml` | `45bf631ead2fd0681597a2d313976d79ac22d6088d0d97a451a54dcf42080259` |
| `Cargo.lock` | `f48f459ed5a7c7b9fb9d184d78099c5c6b31b260b3d13df632dbf539ea07ad70` |
| `crates/seacad-dxf-core/src/johab_decode_le.bin` | `d04a1a13d5f4706df6fa46394cda98a817570e774d601acd042e0fa57249f7ea` |
| `schema/dxf/v1/manifest.json` | `2b3ca697a91b29d68ba8b3a850abf10ce017023919540d87d382da2b902718c0` |
| `schema/dxf/v1/sources.json` | `1d233ace387be55f8691699307de8d0e560255ba094d4279a6dec56d9564c028` |
| `generated/header_schema.rs` | `34675e171859d3075f70344e80c96217ed7994c7dd4e38b6bf09de33b6699b7c` |
| `generated/entity_schema.rs` | `0872b9e0bb85361a549fc77ee5265d05d4420cc8796ddb75ffd2ce6c83eaa610` |
| `release/sbom.cdx.json` | `ca7c5d001cf2f2745392def707e46c699d6fdc7f163fe514886fd526af8830d0` |
| `release/legal/manifest.json` | `83a7ca58381bdde9371a23bda94a224ead894c51b07241246cc662ed20b1dfb0` |
| `audits/m1/legacy-dxf-fixtures.csv` | `ac73ecd420e23b11334805931abd2f1a7ba330f29c0eeb0ae92e72b1c1529783` |

Run:

```powershell
Get-Content -Raw LICENSE
rg -n '^publish = false$|^license-file\.workspace = true$' crates -g Cargo.toml
$legal = Get-Content -Raw release/legal/manifest.json | ConvertFrom-Json
$sbom = Get-Content -Raw release/sbom.cdx.json | ConvertFrom-Json
"legal_package_count=$(@($legal.packages).Count)"
"sbom_component_count=$(@($sbom.components).Count)"
rg -n 'Direct transfers approved|approved zero direct transfers|direct_transfer_approved' docs/audits/M1_LEGACY_DXF_AUDIT.md audits/m1/legacy-dxf-fixtures.csv
rg -n '@generated by seacad-schema-gen|Normalized .* SHA-256' crates/seacad-dxf-core/src/generated -g '*.rs'
```

Expected: proprietary root text; three `publish = false` and three inherited
license-file declarations; 26 legal packages; 29 SBOM components; M1 direct
transfers remain zero/false; both generated modules identify the generator and
normalized input hash.

## 10. Phase 5 - Boundary and non-mutation review

Run:

```powershell
rg -n 'Public candidate|Private|Mixed|relicens|authority|DCO|CLA|assignment|Autodesk|Johab|fixture|corpus|seacad-dxf-core|seacad-cli|seacad-schema-gen|schema/dxf/v1|release/|\.github|\.agents' docs/audits/R0_2_PROVENANCE_OWNERSHIP_BOUNDARY.md
git diff -- docs/SUPPORT_MATRIX.md Cargo.toml Cargo.lock LICENSE NOTICE THIRD_PARTY_NOTICES.md crates schema corpus release .github .agents
git diff --check
```

Required evidence:

- the audit names all three workspace crates and every root class that could
  cross the export boundary;
- the license target remains a non-authorized future proposal;
- the scoped non-document diff contains only the three expected concurrent DXF
  code paths;
- no support matrix, manifest, lockfile, current legal file, schema, corpus
  policy, release artifact, workflow, or agent-skill change exists;
- `git diff --check` exits `0`.

## 11. Phase 6 - Required repository gates

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

Every command must exit `0`. Record the exact workspace test count. Formatting
and generators are check-only.

## 12. Phase 7 - Postflight mutation audit

Repeat Phase 1 Git commands and concurrent-code receipts. Repeat Phase 2
document receipts and Phase 4 artifact hashes/counts. Initial and final `HEAD`,
changed path sets, receipts, and artifact hashes must match exactly. List only
ignored `target` output as tolerated ephemeral output.

## 13. Acceptance criteria

The batch is `PASS` only when:

1. root, commit, changed path sets, and concurrent-code receipts match;
2. R0.2 document receipts and local links match;
3. Git contribution observations match without a legal inference;
4. current license, M1, generated data, SBOM, and legal evidence match;
5. the boundary inventory covers every required class and grants no license;
6. no R0.2-prohibited repository surface changes;
7. every quality gate passes;
8. postflight matches preflight with no repository mutation;
9. the complete report is written to the exact external path.

An unmet criterion is `FAIL` unless a preflight stop condition makes it
`BLOCKED`.

## 14. Required report format

Write one UTF-8 YAML-shaped report with no prose before or after it:

```yaml
batch_id: seacad-r0.2-provenance-ownership-boundary-2026-08-08
status: PASS | FAIL | BLOCKED
root: D:\SeaCad\SeaCad
report_file: D:\SeaCad\AntigravityReports\seacad-r0.2-provenance-ownership-boundary-2026-08-08.yaml
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
git_commit_count:
git_author_count:
git_committer_count:
git_signature_states:
signed_off_by_count:
tracked_path_count:
tracked_cad_path_count:
legal_package_count:
sbom_component_count:
workspace_test_total:
links: PASS | FAIL
current_license_unchanged: PASS | FAIL
relicense_authorized: false
support_matrix_unchanged: PASS | FAIL
non_document_scope_unchanged: PASS | FAIL
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
