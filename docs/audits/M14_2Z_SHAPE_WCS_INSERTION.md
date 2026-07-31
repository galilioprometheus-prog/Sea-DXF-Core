# M14.2z SHAPE WCS Insertion

## Scope

M14.2z transforms the required classic SHAPE insertion from OCS to WCS and
deduplicates the finite text-symbol OCS projection path.

## Normative basis

Autodesk's SHAPE reference defines required insertion coordinates in OCS and
the optional extrusion direction with default `(0,0,1)`:
<https://help.autodesk.com/cloudhelp/2025/ENU/AutoCAD-DXF/files/GUID-0988D755-9AAB-4D6C-8E26-EC636F507F2C.htm>.

Autodesk's object-coordinate-system and arbitrary-axis references define
OCS-to-WCS placement and the normalized basis:
<https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-D99F1509-E4E4-47A3-8691-92EA07DC88F5.htm>,
<https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-E19E5B42-0CC7-4EBA-B29F-5E1D595149EE.htm>.

No parser, fixture, or implementation text was copied from the legacy project.

## Contract

- Required groups 10/20/30 form the SHAPE OCS insertion.
- Groups 210/220/230 use the M14.2c explicit/defaulted/invalid scalar contract.
- Successful projection retains a finite WCS point, normalized WCS normal,
  canonical positive zero, and best available raw provenance.
- Missing, invalid, or duplicate insertion/extrusion values, non-finite Binary
  values, zero extrusion, basis failure, and transformed overflow remain
  distinct typed failures.
- The complete M14.2c SHAPE scalar semantics remains reachable.

## Duplication and size

M14.2z extracts a 129-line crate-private `text_symbol_ocs_projection` helper
used by both TEXT and SHAPE. This reduces `text_wcs_anchor.rs` from 309 to 271
lines while preserving its public API and all four regression tests. The new
SHAPE production module is 280 lines and its integration test is 357 lines.
Every behavior module remains below 500 lines.

## Coverage

All nine supported dialects have ASCII/Binary parity for identity, non-polar,
and negative-polar extrusion. Focused cases cover missing and duplicate
insertion, invalid extrusion, zero extrusion, Binary NaN/infinity, derived
overflow, cancellation, source identity, family scope, raw-record lookup, and
public trait bounds. The existing TEXT WCS suite runs alongside the SHAPE
suite to verify the shared-helper refactor.

## Explicit nonclaims

M14.2z does not resolve shape definitions or styles, apply rotation, width,
oblique, size or thickness geometry, produce glyph outlines, edit, or write.

## Verification

All required gates passed on 2026-07-31: dependency policy, generated-schema
drift, release-evidence drift, formatting, workspace Clippy with warnings
denied, 695 workspace tests with zero failures or ignored tests, changed
production forbidden-macro scanning, and `git diff --check`. The focused SHAPE
and TEXT regression suites passed 8/8 tests. No dependency manifest or
lockfile changed.

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 79 | `6c2c88ca80044d694b7f173dc62f33691bcbc45b2eaed6bee8dad325bd5905bb` |
| `crates/seacad-dxf-core/src/lib.rs` | 765 | `9d65bbbe2963a0471f08aaa50b10117cfcd6059f97d9d33c5b74a56959f722b0` |
| `crates/seacad-dxf-core/src/text_symbol_ocs_projection.rs` | 129 | `ad5e5dd3ced1a1aab9848281996fab96705995d78daccb442cdb77b0f68018ba` |
| `crates/seacad-dxf-core/src/text_wcs_anchor.rs` | 271 | `d27c9a1e2efe771c458d1276b40ae344a27d20aa55cc54c98f6b0e93bb354699` |
| `crates/seacad-dxf-core/src/shape_wcs_insertion.rs` | 280 | `4c03a06128e977d5149724cecdbbabf8109ffd8e202bc790e27ed271d32677bb` |
| `crates/seacad-dxf-core/tests/shape_wcs_insertion_tests.rs` | 357 | `4afae0011a463efee0cacd8ddff494b95bc812f2743d1cc35fe8906cdfc29f21` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 252 | `b49c711959c0fd54e6d6da3098e8391368adc219b1fe96f93569929c1d0a2183` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,603 | `5d4199450880c0e0673648d59995bb684c6a9655baa240c4f192839e17f36f55` |
| `docs/SUPPORT_MATRIX.md` | 1,299 | `93e0349996e359c6ded0f6bb42d2de3e0057f99e4741c268fc7ee2da9069fc3d` |
