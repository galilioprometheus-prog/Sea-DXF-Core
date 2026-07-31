# M14.2y TEXT WCS Placement Anchor

## Scope

M14.2y transforms the selected M14.2x TEXT placement anchor from OCS to WCS
without changing authoritative source bytes.

## Normative basis

Autodesk's TEXT DXF reference defines the alignment points in OCS and the
optional extrusion direction:
<https://help.autodesk.com/cloudhelp/2016/ENU/AutoCAD-DXF/files/GUID-62E5383D-8A14-47B4-BFC4-35824CAE8363.htm>.

Autodesk's object-coordinate-system and arbitrary-axis references define
OCS-to-WCS placement, extrusion normalization, and the exact `1/64` polar-cap
branch:
<https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-D99F1509-E4E4-47A3-8691-92EA07DC88F5.htm>,
<https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-E19E5B42-0CC7-4EBA-B29F-5E1D595149EE.htm>.

The implementation reuses the internal OCS basis established by M10.1l; it
does not copy that algorithm or any legacy parser source.

## Contract

- A usable M14.2x first/second OCS anchor is multiplied by the normalized
  arbitrary-axis basis selected from groups 210/220/230.
- The result retains the selected anchor kind, finite WCS point, normalized
  WCS normal, and canonical positive zero.
- Explicit/defaulted/invalid state and best available raw provenance propagate
  through the source-bound semantic value.
- Invalid or unavailable OCS anchors and extrusion components, non-finite
  Binary inputs, zero extrusion, non-finite basis derivation, and transformed
  overflow remain distinct typed failures.
- The complete M14.2x layout and scalar evidence remains reachable.

## Coverage and size

All nine supported dialects have ASCII/Binary parity for identity, non-polar,
negative polar, and second-alignment projection. Focused cases cover missing
placement, invalid and duplicate extrusion, unsupported justification, zero
extrusion, Binary NaN/infinity, derived overflow, cancellation, source
identity, family scope, raw-record lookup, and public trait bounds.

The new production module is 309 lines and its integration test is 422 lines;
the existing `lib.rs` receives only module and re-export glue. Both behavior
files remain below the repository's 500-line review threshold.

## Explicit nonclaims

M14.2y does not apply TEXT rotation, width factor, oblique or generation flags,
resolve text styles, measure content, derive glyph geometry, edit, or write.

## Verification

All required gates passed on 2026-07-31: dependency policy, generated-schema
drift, release-evidence drift, formatting, workspace Clippy with warnings
denied, 691 workspace tests with zero failures or ignored tests, changed
production forbidden-macro scanning, and `git diff --check`. The focused WCS
anchor suite passed 4/4 tests. No dependency manifest or lockfile changed.

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 79 | `2aa974a27242d9b2ca51823c98455afb59fc67d83c971ca9bd12f01dbbd3f964` |
| `crates/seacad-dxf-core/src/lib.rs` | 759 | `eea6f2977971a698c3c100e3d7c7b3b60ef17e708b40bf99aa150f47f5e2c13d` |
| `crates/seacad-dxf-core/src/text_wcs_anchor.rs` | 309 | `356cfc9ec731751d9d40d0ebe25c316a88b06383ba5486c127628bc86d23e3fd` |
| `crates/seacad-dxf-core/tests/text_wcs_anchor_tests.rs` | 422 | `ff1fc2e1132300cc3feca6f54101fe6805dccb53f3cc40ef80efdc9a59e494d8` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 245 | `8dc9c20b1197fcaeb976dfdcdef15c72108c489cb1466cac95d8e64df602f5ed` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,595 | `3f43029a51363a5bacbafccf00e1bb733c7f39a2aad395ecd8112174e83b225d` |
| `docs/SUPPORT_MATRIX.md` | 1,291 | `ec5f5bac62deb563ea3e16477bf6b2c4e7f6626d740c5a8800534d504b902f89` |
