# Antigravity CLI Mechanical Verification Batch

Status: `READY`

Batch ID: `seacad-point-edit-verification-2026-08-03`

Repository root: `D:\SeaCad\SeaCad`

Target code checkpoint: `M14.3bx — POINT UCS X-axis angle set/update`

Target checkpoint commit: `bab60370cdf2feb1e436fe63aff28cc905a2cb45`

Target annotated tag: `m14.3bx-point-ucs-x-axis-angle-set`

Prepared: `2026-08-03` (`Asia/Saigon`)

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
  than `bab60370cdf2feb1e436fe63aff28cc905a2cb45`.
- The target checkpoint is not an ancestor of current `HEAD`.
- The only paths changed after the target checkpoint are not exactly
  `AGENTS.md` and `docs/ANTIGRAVITY_MECHANICAL_BATCH.md`.
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
git cat-file -t m14.3bx-point-ucs-x-axis-angle-set
git rev-list -n 1 m14.3bx-point-ucs-x-axis-angle-set
git merge-base --is-ancestor bab60370cdf2feb1e436fe63aff28cc905a2cb45 HEAD
git diff --name-only m14.3bx-point-ucs-x-axis-angle-set..HEAD
```

Expected evidence:

- `Get-Location` resolves exactly to `D:\SeaCad\SeaCad`.
- Initial `git status --short --branch` reports a clean worktree.
- `git diff --check` exits `0` with no error output.
- `git cat-file -t` prints `tag`, proving the checkpoint is annotated.
- `git rev-list -n 1` prints
  `bab60370cdf2feb1e436fe63aff28cc905a2cb45`.
- `git merge-base --is-ancestor` exits `0`.
- `git diff --name-only` prints exactly these two paths and no others:
  `AGENTS.md` and `docs/ANTIGRAVITY_MECHANICAL_BATCH.md`.

Record the initial branch/status text and `HEAD` verbatim for postflight
comparison. The workflow documentation commit is intentionally newer than the
target code checkpoint, so current `HEAD` is not expected to equal the target
checkpoint commit.

## 8. Phase 1 — Recent checkpoint lineage and annotated tags

For each tag below, run `git cat-file -t <tag>` and then
`git rev-list -n 1 <tag>`, preserving the listed order:

```powershell
git cat-file -t m14.3bt-point-extrusion-replacement
git rev-list -n 1 m14.3bt-point-extrusion-replacement
git cat-file -t m14.3bu-point-extrusion-insertion
git rev-list -n 1 m14.3bu-point-extrusion-insertion
git cat-file -t m14.3bv-point-extrusion-partial-completion
git rev-list -n 1 m14.3bv-point-extrusion-partial-completion
git cat-file -t m14.3bw-point-extrusion-reset
git rev-list -n 1 m14.3bw-point-extrusion-reset
git cat-file -t m14.3bx-point-ucs-x-axis-angle-set
git rev-list -n 1 m14.3bx-point-ucs-x-axis-angle-set
```

Every `git cat-file -t` must print `tag`. Expected peeled commits:

| Tag | Expected commit |
| --- | --- |
| `m14.3bt-point-extrusion-replacement` | `876d01df56b05634a4201c4d5c727827f9e192a5` |
| `m14.3bu-point-extrusion-insertion` | `a3bbcbbf7d0341b7c9acde2ef37489df13a21c71` |
| `m14.3bv-point-extrusion-partial-completion` | `93dc000432aaf973285912bcc5b8d4f392bea37a` |
| `m14.3bw-point-extrusion-reset` | `4c83bad585b9598ff626e69c093cc217c7780ada` |
| `m14.3bx-point-ucs-x-axis-angle-set` | `bab60370cdf2feb1e436fe63aff28cc905a2cb45` |

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
```

The `point_edit_session_tests` target must report exactly `31 passed; 0 failed`.
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
- The workspace test command reports exactly `941 passed; 0 failed` across its
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
rg -n "M14\.3b[t-x]|POINT extrusion|UCS X-axis angle|SetUcsXAxisAngle|ResetExtrusion" README.md docs
Get-Item docs/audits/M14_3BT_POINT_EXTRUSION_REPLACEMENT.md,docs/audits/M14_3BU_POINT_EXTRUSION_INSERTION.md,docs/audits/M14_3BV_POINT_EXTRUSION_PARTIAL_COMPLETION.md,docs/audits/M14_3BW_POINT_EXTRUSION_RESET.md,docs/audits/M14_3BX_POINT_UCS_X_AXIS_ANGLE_SET.md | Select-Object FullName,Length
```

Mechanically verify and report whether the output establishes all of the
following, without editing or reinterpreting the documents:

- The current documented completed POINT edit checkpoint is M14.3bx.
- Audit files exist for M14.3bt, M14.3bu, M14.3bv, M14.3bw, and M14.3bx.
- The documentation does not claim completed support for UCS angle reset,
  mixed insert/update batches, clone, delete, or complete entity editing.

If the evidence is ambiguous, record `FAIL` for this phase and quote the
ambiguous lines. Do not decide how the text should be changed.

## 14. Phase 7 — Checkpoint artifact receipts

Run the following exact PowerShell block:

```powershell
$batchArtifacts = @(
  'README.md',
  'crates/seacad-dxf-core/src/entity_edit_session.rs',
  'crates/seacad-dxf-core/src/entity_edit_verification.rs',
  'crates/seacad-dxf-core/src/point_edit.rs',
  'crates/seacad-dxf-core/tests/point_edit_session_tests.rs',
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
| `README.md` | 334 | `9a40fecf2d71f25c744b705ec8b372150a675de38c5ae95f09eceed9daf054f8` |
| `crates/seacad-dxf-core/src/entity_edit_session.rs` | 1580 | `9d9de5d3741270e37cf966a1785d999aad0e035ec4e097b5012661c301e73921` |
| `crates/seacad-dxf-core/src/entity_edit_verification.rs` | 1143 | `107fb722031559f6ebedebe64e084c044957018df266402518ff68416390ab20` |
| `crates/seacad-dxf-core/src/point_edit.rs` | 1022 | `c9da631c0fdda830206feb1ca233b52b9fff7b24819738e2b8d7320d4e8963b4` |
| `crates/seacad-dxf-core/tests/point_edit_session_tests.rs` | 2452 | `2cea4733a461674bdbb58ef51fe58233610897b2028e8357fb30ef64a0020392` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1184 | `bb6d5a54e9d70a55d2cfc427d7f6916809b44148cca56913f1741345cd71aa2e` |
| `docs/IMPLEMENTATION_PLAN.md` | 2613 | `6c592fb42ad8e07e5dd22f66cf5437de8305cdf596bdff9c67239b3389db31e5` |
| `docs/SUPPORT_MATRIX.md` | 2266 | `48f643175aa8fe4b52469c3136fc2fd005c1642e597eb33d6af303a1ad5a497c` |

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
3. All five recent tags are annotated and peel to their expected commits.
4. Every focused test target passes, including exactly 31 POINT edit tests.
5. Both generated-artifact checks pass without mutation.
6. All full repository gates pass, including exactly 941 workspace tests.
7. The prohibited-API scan is empty with expected exit code `1`.
8. Documentation and audit evidence satisfies the stated mechanical checks.
9. Every artifact line count and SHA-256 matches.
10. Initial and final Git state and `HEAD` are identical, with no tracked or
    untracked project mutation.
11. There are no undeclared deviations, retries, installations, updates,
    credential prompts, or prohibited actions.

Any unmet criterion makes the batch `FAIL`, unless an immediate stop condition
makes it `BLOCKED`.

## 17. Required report format

Return only one YAML-shaped report block using this schema. Do not add prose
before or after it. Do not omit commands, including commands with empty output.

```yaml
batch_id: seacad-point-edit-verification-2026-08-03
status: PASS | FAIL | BLOCKED
root: D:\SeaCad\SeaCad
started_at:
finished_at:
head_before:
head_after:
git_before: |
  <verbatim initial git status --short --branch>
git_after: |
  <verbatim final git status --short --branch>
checkpoint:
  tag: m14.3bx-point-ucs-x-axis-angle-set
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
