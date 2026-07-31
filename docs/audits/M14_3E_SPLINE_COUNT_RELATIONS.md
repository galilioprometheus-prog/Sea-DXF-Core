# M14.3e SPLINE Count Relations

## Scope

M14.3e compares each unique nonnegative declared knot, control-point, and
fit-point count with the number of retained group-40, group-10, or group-11
anchor occurrences in the same SPLINE record.

## Contract

- Three stable count-relation entries are exposed per retained SPLINE record.
- Nonnegative declarations are classified as `Matched` or `Mismatched`.
- Negative, absent, invalid, and duplicate declarations remain distinct.
- Every explicit or invalid declaration retains its M14.3a value evidence.
- Observed counts use exact M14.3b member cardinality and do not depend on
  numeric lexical validity.
- Source identity, cancellation, record/kind/ordinal lookup, and ASCII/Binary
  parity remain explicit.

## Coverage and size

The 3-test focused suite covers all nine dialects in ASCII and Binary, matched
knot/control-point relations, a mismatched fit-point relation, negative,
absent, invalid, and duplicate declarations, cancellation, lookup misses, and
public traits.

Production/test modules are 296/304 lines, below 500. No user-facing/i18n
string, dependency, manifest, or lockfile changes.

## Nonclaims

Anchor occurrence count does not prove that complete coordinate tuples exist.
This checkpoint does not validate knot or point member values, group coordinate
components, validate curve topology/geometry, process HELIX, edit, or write.

## Verification

All required local gates passed on 2026-07-31: dependency policy,
generated-schema drift, release-evidence drift, formatting, workspace Clippy
with warnings denied, 728 workspace tests with zero failures or ignored tests,
production forbidden-macro scanning, and `git diff --check`. The focused suite
passed 3/3 tests. The first combined workspace-test command reached its local
120-second wrapper limit; the isolated workspace test rerun completed with
exit 0.

GitHub Actions remains blocked by the account budget/payment limit. This local
checkpoint creates no additional cloud run and makes no cloud/six-host claim.
No corpus-private directory exists at any of the three authorized release-gate
locations, so the corpus gate was not run and no corpus claim is made.

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 81 | `49afef29d1a2d4299738788583a28c53f14d4fa3aaf0e25b3db5ab9cd8f28f8a` |
| `crates/seacad-dxf-core/src/lib.rs` | 839 | `4cdd0d2e9a79b1a0d4faa5d4840a0de6c4dafdc987a32d1fd81ae9e46921c81b` |
| `crates/seacad-dxf-core/src/spline_count_relation.rs` | 296 | `1617110212ad6c27c6ba1f44d0c526a639f78a001ab8a46ce34f3c800116bb2d` |
| `crates/seacad-dxf-core/tests/spline_count_relation_tests.rs` | 304 | `486b59c899f1bfa8e5f79ef14b2301711913fb3c93e9c2bda870cfafa05e8914` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 341 | `13271b9d3766212ec149063f1188452782d9d9cd888d301dd400ef83af02be07` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,708 | `43148bde850777ae926ad3bc5e926b70e78736ff244e99bbdfd109bfcf250912` |
| `docs/SUPPORT_MATRIX.md` | 1,402 | `c654f72acb93a749d6216c39e77dd1d24c8a7d0099317f7211f9fa43e50d7de4` |
