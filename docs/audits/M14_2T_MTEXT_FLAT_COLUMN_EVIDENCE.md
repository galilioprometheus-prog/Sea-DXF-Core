# M14.2t MTEXT Flat Column Evidence

## Scope

M14.2t adds bounded evidence for Autodesk's direct MTEXT column groups 75, 76,
78, 79, 48, and 49. It deliberately preserves group 50 as
`RotationOrColumnHeight` because Autodesk also assigns that code to rotation.

## Behavior

- Only an unambiguous direct column field creates a flat-column entry.
- All direct column fields and every coexisting group 50 retain source order,
  typed numeric results, raw groups, and source identity.
- A lone group 50 remains orientation evidence and creates no column entry.
- The existing text evidence boundary prevents fields after exact group 101
  `Embedded Object` from leaking into the direct slice.
- No record-order heuristic silently converts rotation into column height.

## Coverage and size

ASCII/Binary parity covers AC1009 through AC1032. Tests cover source order,
invalid numerics, cancellation, lookup, public traits, rotation-only records,
and the embedded boundary. Production and test modules remain below 500 lines.

## Explicit nonclaims

M14.2t does not unify flat scalars with Embedded/XDATA storage, disambiguate
group 50, derive layout geometry, or edit/write MTEXT columns.

## Verification

All required gates passed on 2026-07-31: dependency policy, generated-schema
drift, release-evidence drift, formatting, workspace Clippy with warnings
denied, 680 workspace tests with zero failures or ignored tests, changed
production forbidden-macro scanning, and `git diff --check`. Three focused
flat-column tests also passed independently. No dependency manifest or
lockfile changed.

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 79 | `f62247bc59081199deb8deb462a4ebd3e29ed91cfb371f8b4d8d5d14d34aefe2` |
| `crates/seacad-dxf-core/src/lib.rs` | 740 | `9d6945e0472051e15028197c49adbbe2c6508bad4c4f82c68ebe6e07c596d476` |
| `crates/seacad-dxf-core/src/mtext_flat_column_evidence.rs` | 248 | `e8241483f6d8b376360acecb30e214394a9a0aa1a263b97fcc8007b8c30a154c` |
| `crates/seacad-dxf-core/tests/mtext_flat_column_evidence_tests.rs` | 242 | `23c71b32e1eb8af9b5660564965f6a9d780a08278ec0745719278d325318d061` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 214 | `e0532e89202364937d50adfe1781317a5ec2bcf92b14687e36260e2bbaa1a75c` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,559 | `5595f330cd6dc5dfd5820102c7faaaff686ed9395e46dc98b465be900008bcc2` |
| `docs/SUPPORT_MATRIX.md` | 1,253 | `c353d939d02caad36b548b0957b06eb2e315ebbe97bedf20849619a38710a4fd` |
