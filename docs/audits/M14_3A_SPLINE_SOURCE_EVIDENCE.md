# M14.3a SPLINE Source Evidence

## Scope

M14.3a begins the curve family with exact numeric evidence for uppercase
`SPLINE` records in completely indexed BLOCKS and ENTITIES sections.

The normative field list is Autodesk's SPLINE DXF reference:
<https://help.autodesk.com/cloudhelp/2026/CSY/AutoCAD-DXF/files/GUID-E1F884F8-AA90-4864-A215-3182D47A9C74.htm>.
The legacy SeaCad tree was used only to confirm risk ordering: real files may
carry undocumented higher group-70 bits, so this checkpoint retains the exact
integer instead of rejecting unknown flags. No legacy parser or fixture was
copied.

## Contract

- Five group-70-through-74 fields remain exact signed-16-bit evidence.
- Nineteen tolerance, tangent, knot, control-point, fit-point, and normal roles
  remain exact binary64 evidence.
- Repeated knot/control/fit values and duplicate singleton candidates preserve
  source order; invalid ASCII values preserve typed lexical failures.
- Group-102 application content cannot impersonate a SPLINE field.
- Exact case-sensitive markers and complete BLOCKS/ENTITIES membership are
  required; other sections and near-match markers are ignored.
- Source identity, cancellation, raw record/group lookup, and public type
  bounds remain explicit across ASCII and Binary.

## Coverage and size

The 4-test focused suite covers all nine dialects in ASCII and Binary, every
role, repeated values, integer/binary64 wire separation, exact negative-zero
bits, invalid syntax/range, duplicate fields, application-group decoys, empty
records, section/marker filtering, cancellation, lookup misses, and public
traits.

The production module is 388 lines and the integration test is 288 lines, both
below 500. `lib.rs` changes only by the module declaration and re-exports. No
user-facing/i18n string, dependency, manifest, or lockfile changes.

## Nonclaims

This checkpoint does not select values, apply tolerances or version defaults,
interpret flags, reconcile declared/observed counts, group point components,
validate knots or curve topology, construct SPLINE geometry, parse HELIX
subclass fields, edit, or write.

## Verification

All required local gates passed on 2026-07-31: dependency policy,
generated-schema drift, release-evidence drift, formatting, workspace Clippy
with warnings denied, 722 workspace tests with zero failures or ignored tests,
production forbidden-macro scanning, and `git diff --check`. The focused suite
passed 4/4 tests.

GitHub Actions remains externally blocked before its first step by the account
budget/payment limit. This checkpoint is retained locally for a later batched
push, so it has not created another cloud run and makes no cloud or six-host
success claim.

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 81 | `82edf7c6c30a1ae53c1e937355a6797e6e73df04b5809ff01007935da7d87ed0` |
| `crates/seacad-dxf-core/src/lib.rs` | 825 | `5e626ee774e6661dc8cc4fc427768fe1b0ce723827b980d10bbff3c59432a36c` |
| `crates/seacad-dxf-core/src/spline_evidence.rs` | 388 | `9149e35563c7e54bc41dfd4f08f1aae45e58a2eced5bd0fb16d5a18b4a6713a7` |
| `crates/seacad-dxf-core/tests/spline_evidence_tests.rs` | 288 | `d267eb29fcc83909f3890f8509e5f06d12cc9cd68fee8f79be4b9c577badba5e` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 317 | `7317b3204cc0d74a24bdcfac1a79e47c67b599a740e55b24ca2f94f132192f29` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,682 | `fee8014127ccb1e690ddbef92a3df9507d02b51b1b42e2d2999691f313e8689e` |
| `docs/SUPPORT_MATRIX.md` | 1,376 | `01d364ca6b5cda77e878dc8463e735c31ab9fc9c722ac5fde883bd8cce22ba4c` |
