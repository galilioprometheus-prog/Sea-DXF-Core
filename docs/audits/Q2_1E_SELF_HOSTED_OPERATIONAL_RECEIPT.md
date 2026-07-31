# Q2.1e Self-Hosted Operational Receipt

## Evidence

Manual workflow run `30640753470` completed successfully on 2026-07-31:

<https://github.com/seaflower205/SeaCad/actions/runs/30640753470>

- Workflow: `CI (self-hosted manual)`.
- Event: `workflow_dispatch`.
- Commit: `9e7c47a2af2a80f79c59f51ef151f62090572c91`.
- Job: `Windows x64 local quality`, ID `91189779259`.
- Job duration: 3 minutes 48 seconds.
- Conclusion: `success`.

## Passed steps

Setup, credential-free checkout, provisioned Rust/cargo-deny verification,
dependency policy, formatting, generated schemas, release evidence, workspace
Clippy with warnings denied, all 728 workspace tests, and checkout cleanup each
completed successfully.

The repository runner `seacad-win-x64` returned online and idle after the job.
No automatic push run preceded the manual dispatch.

## Cost boundary

The job used the repository's `self-hosted`, `Windows`, `X64`, `seacad` runner.
It did not allocate a GitHub-hosted runner. The six-native release-artifact
workflow remains separate and manual-only.

## Nonclaims

This receipt proves operational Windows x64 self-hosted quality execution. It
does not count toward the six-native nightly sequence, provide a six-native
artifact receipt, close private corpus or signature gates, or authorize Core
1.0.

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `docs/IMPLEMENTATION_PLAN.md` | 1,722 | `401d94852b1c8b63ec8a4d873d02008de7b417ebbd31a0323b2fba46945dbf0b` |
| `docs/TOOLCHAIN.md` | 276 | `1b0ac028fe7b5d4c5d8ed634ff37d57d115164566ce8400fe96c71e427005c68` |
