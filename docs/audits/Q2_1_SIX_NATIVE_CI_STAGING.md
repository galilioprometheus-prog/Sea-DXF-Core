# Q2.1 six-native CI staging receipt

Date: 2026-07-28

## Scope

Q2.1 stages native SeaCad evidence across Linux, Windows, and macOS on both
x64 and ARM64. It changes CI scheduling and project-status documentation only.
No Rust source, public API, DXF behavior, schema, CLI contract, dependency
manifest, dependency policy, or `Cargo.lock` entry changes.

The existing push and pull-request quality matrix remains unchanged:

- Linux x64 on `ubuntu-24.04`;
- Windows x64 on `windows-2025`;
- macOS ARM64 on `macos-15`.

Q2.1 adds native schema generation, workspace checking, and core library smoke
tests on every push and pull request for:

- Linux ARM64 on `ubuntu-24.04-arm`;
- Windows ARM64 on `windows-11-arm`;
- macOS x64 on `macos-15-intel`.

The daily schedule at `02:43 UTC` and manual dispatch run the full formatting,
schema, Clippy, and workspace-test gate on the three supplemental platforms;
the existing matrix supplies the same full gate on the other three. The
supplemental push/pull-request checks remain staged until twenty consecutive
nightly six-platform runs pass. Q2.2, not Q2.1, owns the offline corpus
manifest and receipt harness.

## Platform evidence

GitHub documents the six selected hosted runner labels at:

<https://docs.github.com/en/actions/reference/runners/github-hosted-runners>

The Linux ARM64 and Windows ARM64 images are public-preview runners. A runner
outage or preview failure remains a failed evidence run; it is not converted
to a passing Core 1.0 receipt.

Rust lists all six compilation targets as Tier 1 with host tools:

<https://doc.rust-lang.org/rustc/platform-support.html>

Q2.1 runs every job natively. It does not substitute cross-compilation for a
native execution receipt.

## Workflow security and determinism

- Workflow permissions remain limited to `contents: read`.
- Baseline jobs use `actions/checkout` v6.0.2 pinned to commit
  `de0fac2e4500dabe0009e67214ff5f5447ce83dd`.
- Supplemental jobs use `actions/checkout` v4.2.2 pinned to commit
  `11bd71901bbe5b1630ceea73d27597364c9af683` for its Node20 runner
  compatibility.
- Persisted checkout credentials remain disabled in every job.
- Rust remains pinned to 1.97.1; no floating toolchain or runner alias is used.
- Matrix jobs use `fail-fast: false`, so one platform cannot hide the remaining
  platform outcomes.
- Every job retains a 20-minute timeout.

## Local verification

The reviewed Windows x64 host reported `cargo-deny 0.20.2`. The required local
gates passed before the initial commit and again after the compatibility
follow-up:

| Command | Result |
| --- | --- |
| `cargo deny --locked check` | passed: advisories, bans, licenses, sources |
| `cargo +1.97.1 fmt --all -- --check` | passed |
| `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check` | passed |
| `cargo +1.97.1 clippy --workspace --all-targets -- -D warnings` | passed |
| `cargo +1.97.1 test --workspace` | passed: 239 tests, 0 failed |
| `git diff --check` | passed |

No local YAML parser or separately approved `actionlint` installation was
available. GitHub workflow acceptance and all six native outcomes therefore
remain mandatory post-push evidence.

## Post-push compatibility evidence

Approved commit `ca1456a2c9439184fcf01aab43a68c37bbbe67ff` triggered CI run
<https://github.com/seaflower205/SeaCad/actions/runs/30352355247> and Dependency
Policy run <https://github.com/seaflower205/SeaCad/actions/runs/30352355266>.
Dependency Policy and all three baseline jobs passed.

On both the original attempt and one failed-job rerun, Linux ARM64, Windows
ARM64, and macOS x64 reached their matching hosted runner but failed during job
setup before repository checkout. Every annotation reported that the runner
could not resolve the pinned v6 checkout action. GitHub API evidence confirmed
that the v6 tag and SHA exist, repository Actions policy allows public actions,
and the three jobs received the intended runner labels. No SeaCad command ran
on the failed jobs.

The v6 action manifest uses Node24. The independently verified v4.2.2 tag points
to signed commit `11bd71901bbe5b1630ceea73d27597364c9af683`, uses Node20, and
retains the MIT license. The Q2.1 follow-up therefore changes only supplemental
checkout steps to that exact v4 pin. Baseline and dependency-policy jobs retain
v6. A subsequent six-platform run must pass before Q2.1 can be tagged.

## Diff and dependency boundary

Before adding the initial receipt, Q2.1 changed five CI/documentation files
with 131 insertions and 21 deletions. No file under `crates/` or `schema/`
changed. `Cargo.toml`, `Cargo.lock`, and `deny.toml` remain unchanged.

The post-push compatibility follow-up changes the workflow, this receipt,
`docs/TOOLCHAIN.md`, and `THIRD_PARTY_NOTICES.md`; it changes no runtime or
schema file.

`Cargo.lock` retains SHA-256
`f48f459ed5a7c7b9fb9d184d78099c5c6b31b260b3d13df632dbf539ea07ad70`.

## Artifact SHA-256

| Artifact | SHA-256 |
| --- | --- |
| `.github/workflows/ci.yml` | `799de07ff5aeac5837577a8ccb46ee4d0f6ba56e2170642053fe413e4399c8db` |
| `README.md` | `7345913e6318d8dc93e8210843cfe745efd08f88787a3cd126f4243891f4ab0b` |
| `docs/IMPLEMENTATION_PLAN.md` | `fbf77a7d4df38359a43caa8f659d69479bee782b8ca96f212c05df9febdd096a` |
| `docs/SUPPORT_MATRIX.md` | `9854a60a0662c64dbc4c5d826f74d1e313adcc1dd1263cbaf96b4a2f30e802e2` |
| `docs/TOOLCHAIN.md` | `cddbb961e34479e660894ad9b25953207f66bd20a7497e38ef680c56f52773cb` |
| `THIRD_PARTY_NOTICES.md` | `983c0deda6aab40fe98c26085bff20f595c04e2f4a856b194cf3f050acd41d4d` |
| locally built `cargo-deny.exe` 0.20.2 | `379f7dd526ff77e001b6381d81e01593addace433c4be6a73e286d774371a2a9` |

The initial commit and failed remote run are recorded above. No annotated tag
exists; successful six-platform GitHub evidence remains mandatory first.
