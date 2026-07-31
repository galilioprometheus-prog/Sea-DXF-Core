# M14.2aa SHAPE WCS Insertion Correction

## Correction

M14.2aa corrects M14.2z's coordinate-system classification. Autodesk defines
SHAPE groups 10/20/30 as an insertion point in WCS, not OCS:
<https://help.autodesk.com/cloudhelp/2018/ENU/AutoCAD-DXF/files/GUID-0988D755-9AAB-4D6C-8E26-EC636F507F2C.htm>.

The extrusion direction remains optional with default `(0,0,1)`. Autodesk's
arbitrary-axis algorithm remains relevant to the shape plane and normalized
normal, but it must not be applied to a point already expressed in WCS:
<https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-E19E5B42-0CC7-4EBA-B29F-5E1D595149EE.htm>.

## Corrected contract

- Required groups 10/20/30 retain their exact finite `DxfDouble` WCS values.
- Extrusion groups 210/220/230 are independently normalized into the WCS
  normal using the shared arbitrary-axis implementation.
- No OCS basis is multiplied into the SHAPE insertion point.
- A maximum finite WCS point remains usable even with a non-axis-aligned
  extrusion, proving that no inapplicable transform or derived overflow occurs.
- Missing, invalid, duplicate, and non-finite insertion evidence remains
  typed; invalid, duplicate, non-finite, or zero extrusion and basis failure
  also remain typed with best available provenance.
- The TEXT path remains unchanged: its alignment points are OCS and continue
  through the shared point transform.

## Refactor and coverage

The shared helper now exposes extrusion normalization separately from finite
OCS-point transformation. SHAPE calls only the former; TEXT calls both.
All nine dialects retain ASCII/Binary parity for exact WCS points and identity,
non-polar, and negative-polar normals. Focused tests cover missing/duplicate
insertion, invalid/zero extrusion, Binary NaN/infinity, maximum finite WCS
coordinates, cancellation, source identity, family scope, lookup, and public
traits. The complete TEXT WCS suite runs as a regression.

## Explicit nonclaims

M14.2aa does not resolve SHAPE definitions or styles, apply rotation, width,
oblique, size or thickness geometry, produce glyph outlines, edit, or write.

## Verification

All required gates passed on 2026-07-31: dependency policy, generated-schema
drift, release-evidence drift, formatting, workspace Clippy with warnings
denied, 695 workspace tests with zero failures or ignored tests, changed
production forbidden-macro scanning, and `git diff --check`. The corrected
SHAPE and unchanged TEXT regression suites passed 8/8 tests. No dependency
manifest or lockfile changed.

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 79 | `782e4440c0dc0fcb7b4301de1e9a798402202b45fd129846ac08f8a820dfb1c1` |
| `crates/seacad-dxf-core/src/shape_wcs_insertion.rs` | 290 | `f879aaf87af2274d04a90850dc2b7e8210f6dbfb0bc24d65d4caf1d294f15881` |
| `crates/seacad-dxf-core/src/text_symbol_ocs_projection.rs` | 159 | `b4d218a614a9190d3d552733ab4e019887904bed07047236a86704119a5e874b` |
| `crates/seacad-dxf-core/tests/shape_wcs_insertion_tests.rs` | 362 | `69699c2de1d3e6312ea382c74af56c1d8a576c382735a706cc600282c2a3e568` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 253 | `80813ca6f4d819217533aca5501a70ef12228a38fc906ced89e1e70e2b7c14eb` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,605 | `20793bbd02d8fbc1ed4abbcb74e2e76cbbcc0d77fb663418f58bc5cfd9ea55e1` |
| `docs/SUPPORT_MATRIX.md` | 1,301 | `bfff75fc4ceb2f5359847f54d339e7280bb82ee2685a2300f0bdc0dbcd729c47` |
| `docs/audits/M14_2Z_SHAPE_WCS_INSERTION.md` | 79 | `41546bf81c7d982148e549f0db2d2be3ce16eaa1d8258b9e530427b9e3b43ae7` |
