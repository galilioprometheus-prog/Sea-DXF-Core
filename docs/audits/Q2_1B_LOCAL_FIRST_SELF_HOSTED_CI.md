# Q2.1b Local-First Self-Hosted CI

## Scope

Q2.1b prevents ordinary pushes and pull requests from allocating GitHub-hosted
runners. Development commits are batched locally and the required gates run on
the final commit before one deliberate push. The ordinary CI workflow remains
available only as a manual Windows x64 diagnostic on the repository's
`seacad` self-hosted runner.

The separate manual Native Release Artifacts workflow remains unchanged and
is the only workflow that intentionally allocates all six GitHub-hosted native
platforms.

## Provisioned runner

- Official `actions/runner` version: `2.336.0`, Windows x64.
- Official package SHA-256:
  `d59123a43003e357b0805b5d0f611d0bd2f65ab67d51bd070dd4e7a0f685c162`.
- Install root: `D:\SeaCad\actions-runner`, outside the repository.
- Repository runner: `seacad-win-x64` with `self-hosted`, `Windows`, `X64`,
  `seacad`, and `local-quality` labels.
- Startup: limited interactive Scheduled Task under the `SeaFlower` account,
  so the runner receives the already provisioned Rust/MSVC toolchain.
- Workflow Cargo concurrency: eight jobs.
- Persistent target root: `D:\SeaCad\actions-runner-cache\target`, outside
  the checkout.

No registration token, runner credential, machine identifier, or private path
is committed beyond the reviewed fixed install/cache roots above.

## Execution contract

- `.github/workflows/ci.yml` has `workflow_dispatch` only.
- The manual job verifies provisioned Rust 1.97.1 and cargo-deny 0.20.2.
- Dependency, formatting, schema, release-evidence, Clippy, and workspace-test
  gates remain unchanged in substance.
- Checkout credential persistence remains disabled.
- Pushes, pull requests, and schedules allocate no runner.
- Manual six-native release packaging and receipt aggregation remain hosted,
  isolated, and bound to one exact commit.

## Verification

The first cold-cache dry-run completed locally on 2026-07-31 in 211.2 seconds
with exit 0. Dependency policy, generated-schema drift, release-evidence drift,
formatting, workspace Clippy with warnings denied, 728 workspace tests, and
`git diff --check` all passed. The new persistent cache occupied 3.10 GiB.

The registered runner reported online and idle before the dry-run. No GitHub
workflow was dispatched and no hosted minute was consumed by this checkpoint.

## Nonclaims

The self-hosted diagnostic is Windows x64 evidence only. Repeating jobs on the
same machine is not six-native evidence. Q2.1b does not satisfy the twenty-run
nightly gate, create a release, sign an artifact, or authorize Core 1.0.

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `.github/workflows/ci.yml` | 49 | `3abab332b254b8d0b31184517398f0ca76299b8ae4d71fb86cbb788085b716ab` |
| `README.md` | 83 | `2972cf482f78e144ff02009cd67e6dd0c4d4a373a3228a66c12de231c17e643a` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,717 | `fbb3fdb5216e9b437398d91340a578419ff46ac5d8907fff4c3572dd3752fa39` |
| `docs/TOOLCHAIN.md` | 264 | `a14b4b9118fd8e0bcbc2e196e7aadaaf23c20c815a23f58650ba10a93004195b` |
