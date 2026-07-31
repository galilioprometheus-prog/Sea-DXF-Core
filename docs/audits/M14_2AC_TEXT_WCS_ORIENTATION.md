# M14.2ac TEXT WCS Orientation

## Scope

M14.2ac projects classic TEXT rotation and documented text-generation mirror
bits into WCS glyph axes on the normalized extrusion plane. It composes with
the reviewed OCS-to-WCS placement anchor instead of introducing a parallel
entity parser or coordinate path.

## Normative basis

- Autodesk's TEXT reference defines the first and second alignment points in
  OCS, group 50 as optional rotation with a zero default, group 71 bit 2 as
  backward/mirrored in X, bit 4 as upside-down/mirrored in Y, and extrusion as
  optional with a `(0, 0, 1)` default:
  <https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-62E5383D-8A14-47B4-BFC4-35824CAE8363.htm>
- Autodesk's DXF group-code reference specifies degree storage for group
  50-58 angles in DXF files:
  <https://help.autodesk.com/cloudhelp/2027/ENU/OARX-RefGuide/files/OARX-RefGuide-DXF_Group_Codes.html>
- Autodesk's arbitrary-axis algorithm defines the orthonormal basis generated
  from an extrusion direction:
  <https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-E19E5B42-0CC7-4EBA-B29F-5E1D595149EE.htm>

## Contract

- Absent rotation defaults to zero degrees and preserves the arbitrary-axis
  basis before generation flags are applied.
- Group 71 bit 2 reverses the rotated glyph x axis; bit 4 reverses the rotated
  glyph y axis. Their combination reverses both.
- Unknown generation bits remain bit-exact and do not suppress the two
  documented effects.
- Output axes and normal remain finite, unit length, and mutually orthogonal;
  one mirror bit intentionally changes handedness.
- The selected WCS placement anchor and every lower layout/numeric/raw layer
  remain reachable.
- Invalid or duplicate rotation/flags/extrusion, non-finite Binary values,
  zero extrusion, and basis failure remain typed.

## Coverage and size

Tests cover every supported dialect in ASCII and Binary framing for defaults,
90-degree rotation, backward with an unknown bit, upside-down, both mirror
bits, and a non-polar normal. Focused failures cover invalid and duplicate
rotation/flags, invalid and zero extrusion, Binary NaN/infinity, cancellation,
family scoping, source identity, raw lookup, and public traits. The complete
SHAPE WCS orientation suite runs as a shared-helper regression.

The production orientation module is 311 lines, its focused test module is 429
lines, and the shared projection helper is 175 lines. All remain below the
500-line changed-module ceiling. No dependency or localized UI string is added.

## Nonclaims

This checkpoint does not claim style resolution, text-height/width/oblique
geometry, fit/aligned metric adjustment, glyph outlines, editing, or writing.

## Verification

All required gates passed on 2026-07-31: dependency policy, generated-schema
drift, release-evidence drift, formatting, workspace Clippy with warnings
denied, 703 workspace tests with zero failures or ignored tests, production
forbidden-macro scanning, and `git diff --check`. The focused TEXT and shared
SHAPE orientation suites passed 8/8 tests. No dependency manifest or lockfile
changed.

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 79 | `78b2de69f5d56f5bcd637752405880dcda23e0144dbaed8da75d8c6f42768ad7` |
| `crates/seacad-dxf-core/src/lib.rs` | 775 | `fb30ce0eb6a1da02e8978458b72d470da8902d5c26515858f0b417a5aa56911a` |
| `crates/seacad-dxf-core/src/shape_wcs_orientation.rs` | 278 | `cc783cf49ac9e2e69fdc82dadd235c668b9e5dcbb8724aa7727c29fb20b848fc` |
| `crates/seacad-dxf-core/src/text_symbol_ocs_projection.rs` | 175 | `97bbfd90d6d308b9f95942f8cb75908520aa720001c94dc34e5f623ecfc335b2` |
| `crates/seacad-dxf-core/src/text_wcs_orientation.rs` | 311 | `ceb4555ddb377238f2c922e590b810278c3c154e74d71f65f983f88c0ea88116` |
| `crates/seacad-dxf-core/tests/text_wcs_orientation_tests.rs` | 429 | `4c102b07ccc70e23ab54d9fdf1bf25c5440740a2027314e95a7bce48ef3053f4` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 267 | `50dac4ec9c58ba246cfd70d446a0bfcd91216585b919b8ef98d5796b8c8e9f4d` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,621 | `dd82c23ad744026fb3a602593388f35bfb0a0cad54f176c010214e17f93f6b94` |
| `docs/SUPPORT_MATRIX.md` | 1,317 | `4f1ed781adc20b3439325dca034bea070f13bc504219c2665add00a7362c3ec0` |
