# Antigravity CLI Mechanical Verification Batch

Status: `READY`

Batch ID: `seacad-entity-xdata-coordinate-transform-verification-2026-08-04`

Repository root: `D:\SeaCad\SeaCad`

Target code checkpoint: `M14.3cs - entity XDATA coordinate transform`

Target checkpoint commit: `fbdb3e2de5170d853dfc9d96f3ed522fca061754`

Target annotated tag: `m14.3cs-entity-xdata-coordinate-transform`

Required report file: `D:\SeaCad\AntigravityReports\seacad-entity-xdata-coordinate-transform-verification-2026-08-04.yaml`

Prepared: `2026-08-04` (`Asia/Saigon`)

## 1. Authority and purpose

You are the mechanical verification worker for SeaCad. Execute this batch and
collect raw evidence. You do not own architecture, milestone scope, normative
interpretation, provenance, dependencies, support claims, or release decisions.
Codex and the user retain that authority.

Do not use Agent Hub, an MCP server, a coordination service, or another agent.
This file is the sole task specification for the current batch. Do not infer,
add, remove, reorder, or expand work.

## 2. Mandatory read-first procedure

1. Set the working directory to exactly `D:\SeaCad\SeaCad`.
2. Read `D:\SeaCad\SeaCad\AGENTS.md` completely.
3. Read this file completely.
4. Confirm that this file has exactly one active batch and its status is
   `READY`.
5. Execute the phases below in their stated order.

If `AGENTS.md` conflicts with this batch, stop and report the conflict. If the
root is not exact, stop before running repository commands.

## 3. Allowed writes

The repository is logically read-only. The only tolerated writes are ordinary
ephemeral compiler, linker, test, and analysis outputs under the existing
repository `target` directory, plus tool caches that the already-installed Rust
and `cargo-deny` commands normally use.

One additional write is required: create the parent directory when absent and
write exactly one complete report file at:

`D:\SeaCad\AntigravityReports\seacad-entity-xdata-coordinate-transform-verification-2026-08-04.yaml`

This path is outside the Git repository. It is the only non-ephemeral output
file permitted by this batch. If the file already exists for this same batch,
replace it with the complete result of the current run. Do not create a second
report, sidecar, temporary project file, or copy inside `D:\SeaCad\SeaCad`.

No project source, test, fixture, documentation, configuration, lockfile, Git
metadata, or untracked project file may be created, edited, deleted, renamed, or
reformatted. `cargo fmt` is allowed only with `--check`.

Do not install or update a toolchain, component, binary, crate, dependency, or
system package. If an exact command cannot run without an installation, update,
network download, permission change, or credential prompt, do not approve it;
stop and report the blocker.

## 4. Prohibited actions

- Do not edit or fix any failure.
- Do not run a formatter in write mode.
- Do not run `cargo update` or change `Cargo.lock` or any `Cargo.toml`.
- Do not change file permissions, ownership, environment policy, or security
  settings.
- Do not run Git operations that mutate history, index, worktree, branches,
  remotes, or tags. This includes `add`, `commit`, `amend`, `reset`, `restore`,
  `checkout`, `switch`, `merge`, `rebase`, `cherry-pick`, `clean`, and `stash`.
- Do not fetch, pull, push, force-push, create/delete tags, open/update a pull
  request, publish a release, or dispatch/cancel/re-run a workflow.
- Do not invoke `gh` or any hosting-provider mutation.
- Do not use Agent Hub, MCP, another agent, or an external coordination layer.
- Do not make architecture, scope, support, dependency, provenance, milestone,
  commit, tag, merge, or release decisions.
- Do not replace an exact command with an alternative command.

## 5. Stop and failure policy

Stop the batch immediately and report `BLOCKED` if any of these preflight
conditions occur:

- The root differs from `D:\SeaCad\SeaCad`.
- The initial worktree is not clean.
- The target tag is missing, is not an annotated tag, or peels to a commit other
  than `fbdb3e2de5170d853dfc9d96f3ed522fca061754`.
- The target checkpoint is not an ancestor of current `HEAD`.
- The paths changed after the target checkpoint are not exactly `AGENTS.md`
  and `docs/ANTIGRAVITY_MECHANICAL_BATCH.md`.
- A required installed tool is missing and running the command would require an
  installation or update.
- A repository mutation is detected.

After preflight succeeds, an ordinary test, check, scan, or hash mismatch is a
`FAIL`, not permission to fix anything. Continue running later independent
read-only commands so the report contains the full failure surface. Stop
immediately only if a repository mutation, destructive prompt, credential
prompt, required installation/update, or other prohibited action is encountered.

Never hide, retry away, or repair a failure. A retry is permitted only for a
clearly transient process-launch failure, and both attempts must appear in the
report with the reason for the retry.

## 6. Evidence capture rules

For every command, capture:

- its ordinal number and phase;
- the exact command text;
- exact working directory;
- start and finish timestamps with UTC offset;
- process exit code;
- raw standard output and standard error, kept separate when possible;
- the exact test summary when the command runs tests;
- any deviation or retry.

Do not replace raw evidence with a verbal conclusion. Shorten output only when
it is repetitive; if shortened, retain the first relevant lines, last summary,
and exact omitted-line count.

For the prohibited-API `rg` scan, exit code `1` with empty output means no match
and is the expected pass result. Exit code `0` means findings exist and is a
failure. Any other exit code is a command error.

## 7. Phase 0 — Repository preflight

Run these exact commands in order:

```powershell
Get-Location
Get-Content -Raw AGENTS.md
Get-Content -Raw docs/ANTIGRAVITY_MECHANICAL_BATCH.md
git status --short --branch
git rev-parse HEAD
git diff --check
git tag --points-at HEAD
git cat-file -t m14.3cs-entity-xdata-coordinate-transform
git rev-list -n 1 m14.3cs-entity-xdata-coordinate-transform
git merge-base --is-ancestor fbdb3e2de5170d853dfc9d96f3ed522fca061754 HEAD
git diff --name-only m14.3cs-entity-xdata-coordinate-transform..HEAD
```

Expected evidence:

- `Get-Location` resolves exactly to `D:\SeaCad\SeaCad`.
- Initial `git status --short --branch` reports a clean worktree.
- `git diff --check` exits `0` with no error output.
- `git cat-file -t` prints `tag`, proving the checkpoint is annotated.
- `git rev-list -n 1` prints
  `fbdb3e2de5170d853dfc9d96f3ed522fca061754`.
- `git merge-base --is-ancestor` exits `0`.
- `git diff --name-only` prints exactly `AGENTS.md` followed by
  `docs/ANTIGRAVITY_MECHANICAL_BATCH.md`, and no other path.

Record the initial branch/status text and `HEAD` verbatim for postflight
comparison. The workflow documentation commit is intentionally newer than the
target code checkpoint, so current `HEAD` is not expected to equal the target
checkpoint commit.

## 8. Phase 1 — Recent checkpoint lineage and annotated tags

For each tag below, run `git cat-file -t <tag>` and then
`git rev-list -n 1 <tag>`, preserving the listed order:

```powershell
git cat-file -t m14.3ca-point-reference-safe-delete
git rev-list -n 1 m14.3ca-point-reference-safe-delete
git cat-file -t m14.3cb-canonical-point-clone
git rev-list -n 1 m14.3cb-canonical-point-clone
git cat-file -t m14.3cc-handleless-point-delete
git rev-list -n 1 m14.3cc-handleless-point-delete
git cat-file -t m14.3cd-multi-point-delete
git rev-list -n 1 m14.3cd-multi-point-delete
git cat-file -t m14.3ce-mixed-point-delete-session
git rev-list -n 1 m14.3ce-mixed-point-delete-session
git cat-file -t m14.3cf-point-scalar-common-clone
git rev-list -n 1 m14.3cf-point-scalar-common-clone
git cat-file -t m14.3cg-point-linetype-clone
git rev-list -n 1 m14.3cg-point-linetype-clone
git cat-file -t m14.3ch-point-object-reference-clone
git rev-list -n 1 m14.3ch-point-object-reference-clone
git cat-file -t m14.3ci-point-color-book-clone
git rev-list -n 1 m14.3ci-point-color-book-clone
git cat-file -t m14.3cj-point-proxy-graphics-clone
git rev-list -n 1 m14.3cj-point-proxy-graphics-clone
git cat-file -t m14.3ck-point-delete-graph-scope
git rev-list -n 1 m14.3ck-point-delete-graph-scope
git cat-file -t m14.3cl-entity-xdata-evidence
git rev-list -n 1 m14.3cl-entity-xdata-evidence
git cat-file -t m14.3cm-entity-xdata-appid-resolution
git rev-list -n 1 m14.3cm-entity-xdata-appid-resolution
git cat-file -t m14.3cn-entity-xdata-structure
git rev-list -n 1 m14.3cn-entity-xdata-structure
git cat-file -t m14.3co-entity-xdata-typed-values
git rev-list -n 1 m14.3co-entity-xdata-typed-values
git cat-file -t m14.3cp-entity-xdata-point-tuples
git rev-list -n 1 m14.3cp-entity-xdata-point-tuples
git cat-file -t m14.3cq-entity-xdata-layer-resolution
git rev-list -n 1 m14.3cq-entity-xdata-layer-resolution
git cat-file -t m14.3cr-entity-xdata-capacity
git rev-list -n 1 m14.3cr-entity-xdata-capacity
git cat-file -t m14.3cs-entity-xdata-coordinate-transform
git rev-list -n 1 m14.3cs-entity-xdata-coordinate-transform
```

Every `git cat-file -t` must print `tag`. Expected peeled commits:

| Tag | Expected commit |
| --- | --- |
| `m14.3ca-point-reference-safe-delete` | `e1388672463d9bddeece24f1733185cb7b01ab28` |
| `m14.3cb-canonical-point-clone` | `181946d52524fbc7f6b7b46738abf9482353d163` |
| `m14.3cc-handleless-point-delete` | `bfd35a4a9f5cb9a8947974554fd635c6daddd381` |
| `m14.3cd-multi-point-delete` | `f8d8dfa456515bad11bf358e5ec3de48eca1c638` |
| `m14.3ce-mixed-point-delete-session` | `e91d2fe174649fef04c35b587d5452a174e704f8` |
| `m14.3cf-point-scalar-common-clone` | `9228f0c51d75522d67962946ecfe83ba59c0238b` |
| `m14.3cg-point-linetype-clone` | `c7abb77aabae6be265d4529d67b1f90864097443` |
| `m14.3ch-point-object-reference-clone` | `347cceb79f1a6bf3a73313417d84951e3b025fa6` |
| `m14.3ci-point-color-book-clone` | `562964ec107321781407cf7b491301936d74c7f6` |
| `m14.3cj-point-proxy-graphics-clone` | `78f1e916bfe0cb7c51912a4d942337087ab67391` |
| `m14.3ck-point-delete-graph-scope` | `13ebd4acf15d7b59afb2829176af278c6a51e11b` |
| `m14.3cl-entity-xdata-evidence` | `96bacc15dd6693b1dbf8d1083ab271b5f6b9b65a` |
| `m14.3cm-entity-xdata-appid-resolution` | `bc1288b62a398fcf45fb96d3360ff018adc4fa27` |
| `m14.3cn-entity-xdata-structure` | `3a51ea061118b42c582f5d09aa40f52b65f99b06` |
| `m14.3co-entity-xdata-typed-values` | `0961c02cf1565fdfa45b243610949b30e99a2b5f` |
| `m14.3cp-entity-xdata-point-tuples` | `167bc04a253a60655361208059559c12931c7ef8` |
| `m14.3cq-entity-xdata-layer-resolution` | `38cdf2ad74be1fda6ea04ec5e16478fe204ab3c3` |
| `m14.3cr-entity-xdata-capacity` | `a33791ef7289514d846722f3acb7074b742e28b8` |
| `m14.3cs-entity-xdata-coordinate-transform` | `fbdb3e2de5170d853dfc9d96f3ed522fca061754` |

## 9. Phase 2 — Focused semantic and edit tests

Run each test target independently and preserve its exact test count:

```powershell
cargo +1.97.1 test -p seacad-dxf-core --test basic_geometry_tests
cargo +1.97.1 test -p seacad-dxf-core --test basic_geometry_semantic_tests
cargo +1.97.1 test -p seacad-dxf-core --test entity_draft_record_tests
cargo +1.97.1 test -p seacad-dxf-core --test entity_insert_session_tests
cargo +1.97.1 test -p seacad-dxf-core --test point_edit_session_tests
cargo +1.97.1 test -p seacad-dxf-core --test transaction_plan_tests
cargo +1.97.1 test -p seacad-dxf-core --test transaction_plan_composition_tests
cargo +1.97.1 test -p seacad-dxf-core --test transaction_inverse_tests
cargo +1.97.1 test -p seacad-dxf-core --test transaction_write_tests
cargo +1.97.1 test -p seacad-dxf-core --test entity_edit_verification_tests
cargo +1.97.1 test -p seacad-dxf-core --test entity_edit_write_tests
cargo +1.97.1 test -p seacad-dxf-core --test entity_xdata_tests
cargo +1.97.1 test -p seacad-dxf-core --test entity_xdata_appid_resolution_tests
cargo +1.97.1 test -p seacad-dxf-core --test entity_xdata_structure_tests
cargo +1.97.1 test -p seacad-dxf-core --test entity_xdata_value_tests
cargo +1.97.1 test -p seacad-dxf-core --test entity_xdata_point_tuple_tests
cargo +1.97.1 test -p seacad-dxf-core --test entity_xdata_layer_resolution_tests
cargo +1.97.1 test -p seacad-dxf-core --test entity_xdata_capacity_tests
cargo +1.97.1 test -p seacad-dxf-core --test entity_xdata_coordinate_transform_tests
```

The `entity_insert_session_tests` target must report exactly
`18 passed; 0 failed`. The `point_edit_session_tests` target must report exactly
`44 passed; 0 failed`. The `entity_xdata_tests` target must report exactly
`3 passed; 0 failed`. The `entity_xdata_appid_resolution_tests` target must
also report exactly `3 passed; 0 failed`.
The `entity_xdata_structure_tests` target must report exactly
`3 passed; 0 failed`.
The `entity_xdata_value_tests` target must report exactly
`3 passed; 0 failed`.
The `entity_xdata_point_tuple_tests` target must report exactly
`3 passed; 0 failed`.
The `entity_xdata_layer_resolution_tests` target must report exactly
`3 passed; 0 failed`.
The `entity_xdata_capacity_tests` target must report exactly
`3 passed; 0 failed`.
The `entity_xdata_coordinate_transform_tests` target must report exactly
`3 passed; 0 failed`.
For every other target, report its exact observed count rather than guessing.

## 10. Phase 3 — Generated schema and release evidence

Run:

```powershell
cargo +1.97.1 run --locked -p seacad-schema-gen -- --check
cargo +1.97.1 run --locked -p seacad-schema-gen --bin seacad-release-evidence -- --check
```

Both commands must exit `0`. They are check-only commands and must not change
tracked or untracked project files.

## 11. Phase 4 — Full repository quality gates

Run in this exact order:

```powershell
cargo deny --locked check
cargo +1.97.1 fmt --all -- --check
cargo +1.97.1 clippy --workspace --all-targets -- -D warnings
cargo +1.97.1 test --workspace
git diff --check
```

Expected results:

- Every command exits `0`.
- Formatting is check-only.
- Clippy emits no warnings because warnings are denied.
- The workspace test command reports exactly `992 passed; 0 failed` across its
  complete output. Preserve every per-target summary needed to substantiate the
  aggregate count.
- The final `git diff --check` emits no error output.

## 12. Phase 5 — Prohibited production Rust API scan

Run this exact command:

```powershell
rg -n "unsafe\s*\{|panic!\(|unwrap\(|expect\(|todo!\(|unimplemented!\(" crates/seacad-dxf-core/src crates/seacad-cli/src crates/seacad-schema-gen/src
```

Expected result: no output and exit code `1`. Report that exit code as the
expected clean-scan result, not as a command failure.

## 13. Phase 6 — Documentation, support, and audit consistency

Run:

```powershell
rg -n "M14\.3c[a-s]|entity XDATA|APPID resolution|XDATA structure|typed values|point tuples|layer resolution|capacity|coordinate transform|graph.scope|proxy-graphics clone|color-book clone|object-reference clone|linetype clone|scalar common|mixed POINT|multi-POINT|reference-safe|handleless" README.md docs
Get-Item docs/audits/M14_3CA_POINT_REFERENCE_SAFE_DELETE.md,docs/audits/M14_3CB_CANONICAL_POINT_CLONE.md,docs/audits/M14_3CC_HANDLELESS_POINT_DELETE.md,docs/audits/M14_3CD_MULTI_POINT_DELETE.md,docs/audits/M14_3CE_MIXED_POINT_DELETE_SESSION.md,docs/audits/M14_3CF_POINT_SCALAR_COMMON_CLONE.md,docs/audits/M14_3CG_POINT_LINETYPE_CLONE.md,docs/audits/M14_3CH_POINT_OBJECT_REFERENCE_CLONE.md,docs/audits/M14_3CI_POINT_COLOR_BOOK_CLONE.md,docs/audits/M14_3CJ_POINT_PROXY_GRAPHICS_CLONE.md,docs/audits/M14_3CK_POINT_DELETE_GRAPH_SCOPE.md,docs/audits/M14_3CL_ENTITY_XDATA_EVIDENCE.md,docs/audits/M14_3CM_ENTITY_XDATA_APPID_RESOLUTION.md,docs/audits/M14_3CN_ENTITY_XDATA_STRUCTURE.md,docs/audits/M14_3CO_ENTITY_XDATA_TYPED_VALUES.md,docs/audits/M14_3CP_ENTITY_XDATA_POINT_TUPLES.md,docs/audits/M14_3CQ_ENTITY_XDATA_LAYER_RESOLUTION.md,docs/audits/M14_3CR_ENTITY_XDATA_CAPACITY.md,docs/audits/M14_3CS_ENTITY_XDATA_COORDINATE_TRANSFORM.md | Select-Object FullName,Length
```

Mechanically verify and report whether the output establishes all of the
following, without editing or reinterpreting the documents:

- The current documented completed entity checkpoint is M14.3cs.
- Audit files exist for M14.3ca through M14.3cs.
- The documentation does not claim completed support for cross-container clone,
  reference/text common-property clone, graph payload clone, or complete entity
  editing.

If the evidence is ambiguous, record `FAIL` for this phase and quote the
ambiguous lines. Do not decide how the text should be changed.

## 14. Phase 7 — Checkpoint artifact receipts

Run the following exact PowerShell block:

```powershell
$batchArtifacts = @(
  'README.md',
  'crates/seacad-dxf-core/src/entity_xdata_coordinate_transform.rs',
  'crates/seacad-dxf-core/src/entity_xdata_coordinate_transform_math.rs',
  'crates/seacad-dxf-core/src/lib.rs',
  'crates/seacad-dxf-core/tests/entity_xdata_coordinate_transform_tests.rs',
  'docs/DXF_ENTITY_COMPLETION_PLAN.md',
  'docs/IMPLEMENTATION_PLAN.md',
  'docs/SUPPORT_MATRIX.md'
)
foreach ($batchArtifact in $batchArtifacts) {
  $batchLines = (Get-Content -LiteralPath $batchArtifact).Count
  $batchHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $batchArtifact).Hash.ToLowerInvariant()
  [PSCustomObject]@{
    Path = $batchArtifact
    Lines = $batchLines
    Sha256 = $batchHash
  }
}
```

Compare the raw output with these exact expected receipts:

| Path | Lines | SHA-256 |
| --- | ---: | --- |
| `README.md` | 457 | `c315682a3937d42639a8d10bf7e33357376982e7f3d33fc741e3e74498408895` |
| `crates/seacad-dxf-core/src/entity_xdata_coordinate_transform.rs` | 261 | `d0f9fbb93edb9768f6053b5a6aa0edfd81ba24294b214a30e1d2df6a4dcec192` |
| `crates/seacad-dxf-core/src/entity_xdata_coordinate_transform_math.rs` | 335 | `a833296f2ce98bdfdf3e8167142a7b93f865ae44549dcb4fc11eeb32962f98e9` |
| `crates/seacad-dxf-core/src/lib.rs` | 1118 | `313ac2c4c973424aaa17a24fe30aa8906f59fb5f5cef63c6c9f21a0df847ab8c` |
| `crates/seacad-dxf-core/tests/entity_xdata_coordinate_transform_tests.rs` | 357 | `3ac9f6e138629c6e0d2fbd63277844439dd274e061b4b396564723f4c40f2e1e` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1433 | `6f53dc0f5046178928be563de1139a951082805b123ab15db082d91242cadf0b` |
| `docs/IMPLEMENTATION_PLAN.md` | 2850 | `c55f4ce87b11cbe5fab218b94da68d0f4e1b11f2e0f8c860cf0b5e610da7b4f7` |
| `docs/SUPPORT_MATRIX.md` | 2491 | `6f1c96876d8589c63a3e9fa358799cef4de9e85a30cf41bc8a2d873e5871743d` |

Any line-count or hash mismatch is a failure. Do not regenerate an expected
receipt and do not edit the artifact.

## 15. Phase 8 — Mutation audit and postflight

Run:

```powershell
git diff --check
git diff --name-only
git ls-files --others --exclude-standard
git status --short --branch
git rev-parse HEAD
```

Expected results:

- `git diff --check` exits `0`.
- `git diff --name-only` is empty.
- `git ls-files --others --exclude-standard` is empty.
- Final branch/status text is byte-for-byte identical to the initial
  branch/status text.
- Final `HEAD` is identical to initial `HEAD`.
- No project mutation occurred. Normal ignored `target` outputs are tolerated
  but must still be listed under `tolerated_ephemeral_outputs` if newly observed.

## 16. Batch acceptance criteria

The batch is `PASS` only if all of these are true:

1. Preflight succeeds and all exact commands are attempted in order unless a
   defined immediate stop condition occurs.
2. Every command has a recorded exit code, timestamps, working directory, and
   output evidence.
3. All nineteen recent tags are annotated and peel to their expected commits.
4. Every focused test target passes, including exactly 18 entity insert session
    tests, exactly 44 POINT edit tests, exactly 3 entity XDATA tests, and
    exactly 3 entity XDATA APPID-resolution tests, and exactly 3 entity XDATA
    structure tests, exactly 3 entity XDATA typed-value tests, and exactly 3
    entity XDATA point-tuple tests, exactly 3 entity XDATA layer-resolution
    tests, exactly 3 entity XDATA capacity tests, and exactly 3 entity XDATA
    coordinate-transform tests.
5. Both generated-artifact checks pass without mutation.
6. All full repository gates pass, including exactly 992 workspace tests.
7. The prohibited-API scan is empty with expected exit code `1`.
8. Documentation and audit evidence satisfies the stated mechanical checks.
9. Every artifact line count and SHA-256 matches.
10. Initial and final Git state and `HEAD` are identical, with no tracked or
    untracked project mutation.
11. There are no undeclared deviations, retries, installations, updates,
    credential prompts, or prohibited actions.
12. The complete report is written as one UTF-8 file at the exact required
    report path outside the repository, and no other report file is created.

Any unmet criterion makes the batch `FAIL`, unless an immediate stop condition
makes it `BLOCKED`.

## 17. Required report format

Write one complete UTF-8 YAML-shaped report using the schema below to this exact
path, even when the batch ends as `FAIL` or `BLOCKED`:

`D:\SeaCad\AntigravityReports\seacad-entity-xdata-coordinate-transform-verification-2026-08-04.yaml`

Do not omit commands, including commands with empty output. The report file must
contain no prose before or after the YAML-shaped report. After the file is fully
written and closed, print only its absolute path and final status to the
Antigravity chat. Chat output without the report file is not delivery.

```yaml
batch_id: seacad-entity-xdata-coordinate-transform-verification-2026-08-04
status: PASS | FAIL | BLOCKED
root: D:\SeaCad\SeaCad
report_file: D:\SeaCad\AntigravityReports\seacad-entity-xdata-coordinate-transform-verification-2026-08-04.yaml
started_at:
finished_at:
head_before:
head_after:
git_before: |
  <verbatim initial git status --short --branch>
git_after: |
  <verbatim final git status --short --branch>
checkpoint:
  tag: m14.3cs-entity-xdata-coordinate-transform
  tag_type:
  peeled_commit:
  ancestor_exit_code:
  post_checkpoint_paths:
    - <path or none>
tag_receipts:
  - tag:
    tag_type:
    peeled_commit:
commands:
  - ordinal:
    phase:
    command:
    cwd:
    started_at:
    finished_at:
    exit_code:
    stdout: |
      <raw output or empty>
    stderr: |
      <raw output or empty>
    test_summary: <exact count or not_applicable>
    result: PASS | FAIL | EXPECTED_NO_MATCH | BLOCKED
focused_test_total:
point_edit_test_count:
entity_xdata_test_count:
entity_xdata_appid_resolution_test_count:
entity_xdata_structure_test_count:
entity_xdata_value_test_count:
entity_xdata_point_tuple_test_count:
entity_xdata_layer_resolution_test_count:
entity_xdata_capacity_test_count:
entity_xdata_coordinate_transform_test_count:
workspace_test_total:
documentation_checks:
  current_checkpoint:
  required_audits_exist:
  unsupported_features_not_claimed:
hash_receipts:
  - path:
    observed_lines:
    expected_lines:
    observed_sha256:
    expected_sha256:
    result:
mutations: none | <exact list>
unexpected_untracked: none | <exact list>
tolerated_ephemeral_outputs: none | <exact list>
deviations: none | <exact list>
retries: none | <exact list with both attempts>
failures: none | <exact list>
blocker: none | <exact blocker>
final_assessment: <one literal sentence stating why PASS, FAIL, or BLOCKED>
```
