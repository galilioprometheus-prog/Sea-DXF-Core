# M14.3b SPLINE Cardinality

## Scope

M14.3b adds one fixed cardinality card for each of the 24 M14.3a SPLINE roles
on every retained record. Compact members point back to the original evidence
directory; no numeric value or raw span is copied.

## Contract

- Cards use one public stable role order and expose `Absent`, `Unique`, or
  duplicate-preserving `Multiple` state.
- Cardinality depends only on occurrences, not numeric lexical validity.
- Repeated knots and coordinate components retain every source-order member.
- Card, raw-record, role, member, and evidence lookup failures return `None`.
- Source identity, cancellation, ASCII/Binary parity, and public type bounds
  remain explicit.

## Coverage and size

The focused 4-test suite covers all nine dialects in ASCII and Binary, all 24
cards, four repeated roles, absent roles, invalid unique values, duplicate
values, application-group exclusion inherited from M14.3a, member-to-evidence
resolution, lookup misses, cancellation, and public traits.

The production module is 337 lines and the shared integration test is 398
lines, both below 500. No user-facing/i18n string, dependency, manifest, or
lockfile changes.

## Nonclaims

This checkpoint does not select values, apply defaults, interpret flags,
compare counts, group coordinates, validate knots or topology, construct
SPLINE geometry, process HELIX, edit, or write.

## Verification

All required local gates passed on 2026-07-31: dependency policy,
generated-schema drift, release-evidence drift, formatting, workspace Clippy
with warnings denied, 722 workspace tests with zero failures or ignored tests,
production forbidden-macro scanning, and `git diff --check`. The focused suite
passed 4/4 tests.

GitHub Actions remains unavailable due to the account payment/budget limit.
This checkpoint is local for a later batched push and has not created a cloud
run; no cloud or six-host success is claimed.

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 81 | `b8562853065e1b6f3cecd9b3f2e775a059d7e16702e6a4678e69ebfa7500bc0c` |
| `crates/seacad-dxf-core/src/lib.rs` | 830 | `c54b0c34fb5591687750b6a50839e46458369ba088105d9b0bfdd5f94cd352c2` |
| `crates/seacad-dxf-core/src/spline_card.rs` | 337 | `a43fd4a0098f71ce8853d2dfdb4aad00db6a0c8389e1868df4e48407deb34bc7` |
| `crates/seacad-dxf-core/tests/spline_evidence_tests.rs` | 398 | `b89d9c1659654556444628e3cbe4e35bd78b5f76f086761fd358f51a8346ca6a` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 323 | `aea69aa62538b985f380e8c70bcab54b8fe19dc6e46aed410cc3f90c4d2b1284` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,688 | `4954561fb5ba9cfca6f65ee9c5bbd62bf24f32c443a43c51a752fe882943300c` |
| `docs/SUPPORT_MATRIX.md` | 1,382 | `613dd3b9d0abda90d1056d7badc9faee3a7837f1f41b5948d88f62ffe18b817d` |
