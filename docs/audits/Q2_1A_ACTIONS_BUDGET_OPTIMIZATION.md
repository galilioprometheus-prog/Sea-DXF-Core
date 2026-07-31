# Q2.1a GitHub Actions Budget Optimization

## Reason

GitHub rejected recent jobs before their first step because the account's
Actions payment or spending limit was exhausted. The previous configuration
started seven jobs for every source push: three full quality jobs, three native
smoke jobs, and one dependency-policy job. It also scheduled two daily
workflows.

## Cost-aware execution contract

- A non-Markdown push to `main` or pull request starts one Linux x64 quality
  job.
- That job includes the pinned cargo-deny policy action, formatting, generated
  schema and release-evidence checks, workspace Clippy, and workspace tests.
- Markdown-only changes do not allocate a runner.
- Scheduled CI and scheduled standalone dependency-policy runs are removed.
- Manual CI dispatch runs Linux x64 plus the other five reviewed native hosts,
  each with the full format/schema/evidence/Clippy/test gate.
- The standalone dependency-policy workflow remains manual for targeted use.
- Existing per-ref concurrency cancels a superseded run before it consumes more
  budget.
- Native release artifact assembly remains a distinct manual-only workflow.

Normal source-push fan-out is therefore reduced from seven jobs to one. Full
six-platform evidence remains available but requires an explicit dispatch.

## Evidence boundary

This changes CI scheduling only. It does not change Rust code, dependencies,
DXF support, artifacts, or an existing release claim. Twenty consecutive
six-platform receipts remain open evidence. GitHub cannot validate the updated
workflow until Actions billing is available; no cloud success is claimed.

## Verification

All local gates passed on 2026-07-31: dependency policy, generated-schema
drift, release-evidence drift, formatting, workspace Clippy with warnings
denied, 718 workspace tests with zero failures or ignored tests, production
forbidden-macro scanning, and `git diff --check`. Workflow action revisions
remain pinned to already reviewed exact commit SHAs; no new action or package
was introduced.

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `.github/workflows/ci.yml` | 88 | `53c88954dc2fb0f5ac180d1c8e0fff387429012e90c6eb84ef26ab2594cfaba2` |
| `.github/workflows/dependency-policy.yml` | 28 | `35f5c094553d603e606d7590b8094332fd2dd381b4b904d6ae018683bb2f4ca6` |
| `README.md` | 81 | `ba0f5277059d8afe12424ca42c1437caeac1a6441df844f34a09e844a826c034` |
| `docs/DEPENDENCY_POLICY.md` | 54 | `d9c80a683903f9b7d5d9fe952b6ac2fd69d8cbfbc024e236d1ff29dcf910d2f8` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,675 | `0ba30c64560169a6a0b02b92452607ac236c0ba91ba1d7e2d0dfbd0040ababf9` |
| `docs/TOOLCHAIN.md` | 253 | `1fc25495ea12317e02b6fab7de7a13d936b7078db176e5e472bdb3b22e02843b` |
