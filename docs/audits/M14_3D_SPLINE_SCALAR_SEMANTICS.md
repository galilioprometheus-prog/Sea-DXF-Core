# M14.3d SPLINE Scalar Semantics

## Scope

M14.3d selects the unique degree, knot/control/fit counts, and knot/control/fit
tolerances through M14.3b cards. Missing integer scalars remain absent. Missing
tolerances use Autodesk's documented defaults of `1e-7`, `1e-7`, and `1e-10`.

## Contract

- Seven stable entries are exposed per retained SPLINE record.
- `Absent`, `Defaulted`, `Multiple`, `Invalid`, and `Explicit` remain distinct.
- Explicit and invalid states retain the original M14.3a value evidence.
- No degree/count default is invented and no undocumented range is enforced.
- Wire-domain invariants remain fail-closed internally.
- Source identity, cancellation, record/role/ordinal lookup, and ASCII/Binary
  parity remain explicit.

## Coverage and size

The 3-test focused suite covers all nine dialects in ASCII and Binary, every
scalar, exact explicit values, all three defaults, missing counts, duplicate
degree/tolerance values, invalid integer/double syntax and range, negative
explicit counts without guessed validation, cancellation, lookup misses, and
public traits.

Production/test modules are 267/317 lines, below 500. No user-facing/i18n
string, dependency, manifest, or lockfile changes.

## Nonclaims

This checkpoint does not validate scalar domains, reconcile declared and
observed counts, group coordinate components, validate knots/topology,
construct SPLINE geometry, process HELIX, edit, or write.

## Verification

All required local gates passed on 2026-07-31: dependency policy,
generated-schema drift, release-evidence drift, formatting, workspace Clippy
with warnings denied, 725 workspace tests with zero failures or ignored tests,
production forbidden-macro scanning, and `git diff --check`. The focused suite
passed 3/3 tests.

GitHub Actions remains blocked by the account budget/payment limit. This local
checkpoint creates no additional cloud run and makes no cloud/six-host claim.

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 81 | `8d1253dd6df0bf23b875f571c8da23c3fc7f1e0a871a506f003220619f14b837` |
| `crates/seacad-dxf-core/src/lib.rs` | 834 | `55cffa51bfb9a3eba3023a28d75bd9b97b4c6ea652aa5e32bb4a6ac6a781245f` |
| `crates/seacad-dxf-core/src/spline_scalar_semantic.rs` | 267 | `7dcbb10ea2942c8d62ee2dd1359c459b1ced666131c3b08bd122bf1fcedef0fb` |
| `crates/seacad-dxf-core/tests/spline_scalar_semantic_tests.rs` | 317 | `6a77a660d39d4742d67bc0d22262221bd5e809061b553aa21948642055067a69` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 335 | `06e698c19f07839af56550d3888dffe14ab3c9537ca187553f4280d01fd6d6c9` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,701 | `d8ace92aacb6da09b4b2c5e6b03f1dbab09bcbb868d07d90c32082254ce18dc4` |
| `docs/SUPPORT_MATRIX.md` | 1,395 | `7ea93d194194b2e424f94ca7a8fe627449851c97bc5fd7db50e5a2bf80a0e9d8` |
