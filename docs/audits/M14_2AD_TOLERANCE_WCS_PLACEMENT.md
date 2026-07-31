# M14.2ad TOLERANCE WCS Placement

## Scope

M14.2ad composes TOLERANCE's required insertion and x-axis direction with its
optional extrusion into one source-anchored WCS placement semantic. It reuses
the reviewed scalar directory and shared extrusion normalizer rather than
introducing another parser or coordinate path.

## Normative basis

Autodesk's TOLERANCE reference defines groups 10/20/30 as the insertion point
in WCS, groups 11/21/31 as the x-axis direction vector in WCS, and groups
210/220/230 as optional extrusion with a `(0, 0, 1)` default:
<https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-ADFCED35-B312-4996-B4C1-61C53757B3FD.htm>.

Autodesk's arbitrary-axis algorithm is used only to validate and normalize the
extrusion direction:
<https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-E19E5B42-0CC7-4EBA-B29F-5E1D595149EE.htm>.

## Contract

- Required insertion and x-axis components remain exact `DxfDouble` WCS
  values, including signed zero and maximum finite magnitudes.
- The x-axis must be finite and nonzero, but its magnitude is preserved.
- Extrusion is independently normalized into a finite WCS normal.
- No OCS transform is applied to either WCS vector. No y-axis, orthogonality,
  glyph box, or display metric is invented.
- Missing, invalid, duplicate, non-finite, zero x-axis/extrusion, and basis
  failures remain typed with best available provenance.
- The complete underlying TOLERANCE numeric/card/raw evidence remains
  reachable.

## Coverage and size

Tests cover all nine supported dialects in both ASCII and Binary framing for
default, non-polar, and negative-polar normals. Exact signed zero, an unscaled
x-axis, and a maximum finite x-axis component are frozen. Focused failures
cover missing, invalid, duplicate, zero, NaN, and infinity inputs,
cancellation, source identity, family scoping, raw lookup, and public traits.

The production module is 332 lines and its focused test module is 414 lines,
both below the 500-line changed-module ceiling. No dependency or localized UI
string is added.

## Nonclaims

This checkpoint does not resolve DIMSTYLE records, interpret the tolerance
display string, construct a y-axis or glyph box, render glyphs, edit, or write.

## Verification

All required gates passed on 2026-07-31: dependency policy, generated-schema
drift, release-evidence drift, formatting, workspace Clippy with warnings
denied, 707 workspace tests with zero failures or ignored tests, production
forbidden-macro scanning, and `git diff --check`. The focused TOLERANCE WCS
placement suite passed 4/4 tests. No dependency manifest or lockfile changed.

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 79 | `aa6d04c75255156c1f5e7854c32d45dd4b691f3d9ca5e2dd028c4dfe3a8ea0d7` |
| `crates/seacad-dxf-core/src/lib.rs` | 781 | `14b52a2b90c6fb6a0d7b5c07f757ff4dd2adca796de2377dde02eb37697a7506` |
| `crates/seacad-dxf-core/src/tolerance_wcs_placement.rs` | 332 | `94b374bf6da14082264ff4411679b541d1db7dbd12b695160e116dd6ecfd2cf0` |
| `crates/seacad-dxf-core/tests/tolerance_wcs_placement_tests.rs` | 414 | `6335382cdc2f610301377ed068eddfc63ab0c6ff17cb21398905bbc6fb5f957d` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 273 | `fb89ed509d86514b000a44bfe0713f604e90926740f80e51b7879395495fc405` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,628 | `dbace14e5e24410a9a6863ea8f777de0f2cd78a89a73859fd1d8d209f6642b99` |
| `docs/SUPPORT_MATRIX.md` | 1,324 | `6d14d0a0ce7c049a23d1bafdea45504fa9fd177e7fbddd2ae6f9dd093e3e6c41` |
