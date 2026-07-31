# M14.3c SPLINE Flag Semantics

## Scope and contract

M14.3c selects SPLINE group 70 only when its M14.3b card is unique. Absence,
duplicate occurrences, and invalid ASCII numbers remain separate states. A
valid signed value exposes Autodesk's closed, periodic, rational, planar, and
linear bits while retaining the exact signed value and every unknown bit.

The legacy behavioral notes record real values such as 1064 and warn that
rejecting bits above the documented low five discarded valid source geometry.
The regression therefore verifies that 1064 remains usable as planar and
retains 1056 unknown bits. No legacy implementation or fixture was copied.

## Coverage and size

The focused 4-test suite covers all nine dialects in ASCII and Binary, every
documented flag helper, exact raw values, extended unknown bits, absent,
invalid, and unique states, card/evidence composition, cancellation, lookup,
and public traits. Duplicate group-70 state is inherited directly from the
already tested M14.3b card contract.

The production/test modules are 434/426 lines, below 500. No user-facing/i18n
string, dependency, manifest, or lockfile changes.

## Nonclaims

This checkpoint does not validate relationships among flags, select other
scalars, apply defaults, reconcile counts, group points, validate or construct
a curve, process HELIX, edit, or write.

## Verification

All required local gates passed on 2026-07-31: dependency policy,
generated-schema drift, release-evidence drift, formatting, workspace Clippy
with warnings denied, 722 workspace tests with zero failures or ignored tests,
production forbidden-macro scanning, and `git diff --check`. The focused suite
passed 4/4 tests.

GitHub Actions remains blocked by the account budget/payment limit. This local
checkpoint creates no additional cloud run and makes no cloud/six-host claim.

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 81 | `97d0ea4ca861f871f1e0d78dbc739cb6da85706addd9aa14ce1e76277e4f4ab7` |
| `crates/seacad-dxf-core/src/lib.rs` | 830 | `eafdc41afcde11802988bc94697d0d5fd67fd0703e466dbc07ead66d0ab72b8e` |
| `crates/seacad-dxf-core/src/spline_card.rs` | 434 | `899bf43cdab90cd5966e8c3929b8aa928eee4538c15a323e5ef1a4f96ec04e33` |
| `crates/seacad-dxf-core/tests/spline_evidence_tests.rs` | 426 | `60ce5807f00bc1838e6ae9c7cb61aa95f32b69f299d3aef5f67f05051316b9c4` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 329 | `38e1a63c078d01cf984b48d2ca0d2babbd618d2f3bf18631adcfa8fe0c624726` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,694 | `769c3842b0eb6060f135d35ca7bc058583bd6cd9ed6728f9d8b28ebadc7c036f` |
| `docs/SUPPORT_MATRIX.md` | 1,388 | `a803baed14880320bd675073edc50fbfd740e61ac42e65da5c86b1bb173c55e7` |
