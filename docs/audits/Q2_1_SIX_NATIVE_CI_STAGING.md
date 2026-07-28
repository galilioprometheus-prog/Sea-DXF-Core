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
- Every checkout uses `actions/checkout` v6.0.2 pinned to commit
  `de0fac2e4500dabe000e67214ff5f5447ce83dd`.
- Persisted checkout credentials remain disabled in every job.
- Rust remains pinned to 1.97.1; no floating toolchain or runner alias is used.
- Matrix jobs use `fail-fast: false`, so one platform cannot hide the remaining
  platform outcomes.
- Every job retains a 20-minute timeout.

## Local verification

The reviewed Windows x64 host reported `cargo-deny 0.20.2`. The required local
gates passed:

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
remain mandatory post-push evidence; they are not claimed by this local
receipt.

## Diff and dependency boundary

Before adding this receipt, Q2.1 changed five CI/documentation files with 131
insertions and 21 deletions. No file under `crates/` or `schema/` changed.
`Cargo.toml`, `Cargo.lock`, and `deny.toml` remain unchanged.

`Cargo.lock` retains SHA-256
`f48f459ed5a7c7b9fb9d184d78099c5c6b31b260b3d13df632dbf539ea07ad70`.

## Artifact SHA-256

| Artifact | SHA-256 |
| --- | --- |
| `.github/workflows/ci.yml` | `5691df3a91167c6d41287251361d8a5fa615b5638b015088aa8a6483d0a3a4d9` |
| `README.md` | `7345913e6318d8dc93e8210843cfe745efd08f88787a3cd126f4243891f4ab0b` |
| `docs/IMPLEMENTATION_PLAN.md` | `fbf77a7d4df38359a43caa8f659d69479bee782b8ca96f212c05df9febdd096a` |
| `docs/SUPPORT_MATRIX.md` | `9854a60a0662c64dbc4c5d826f74d1e313adcc1dd1263cbaf96b4a2f30e802e2` |
| `docs/TOOLCHAIN.md` | `35886eb43425aeab04deeb661e68afc652a38f4b9e5bb5d97b55f51ae51abc38` |
| locally built `cargo-deny.exe` 0.20.2 | `379f7dd526ff77e001b6381d81e01593addace433c4be6a73e286d774371a2a9` |

No commit, remote workflow run, or annotated tag is claimed by this receipt.
Those actions require checkpoint approval and successful GitHub evidence.
