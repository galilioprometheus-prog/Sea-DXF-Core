# M14.2f TEXT Layout Codes

## Scope

M14.2f projects M14.2c TEXT integer scalars into:

- exact generation-flag bits with backward, upside-down, and unknown-bit
  helpers;
- all six documented horizontal justification values;
- all four documented vertical justification values;
- typed first-versus-second alignment-point applicability.

The projection preserves source/default/absent/invalid states and exact raw
provenance. It does not normalize unknown generation bits or guess unsupported
justification codes.

## Normative basis

Autodesk's DXF TEXT entity reference defines group 71 generation flags, group
72 horizontal justification codes, group 73 vertical justification codes, and
the rule that the second alignment point controls placement when either
justification code is nonzero:
<https://help.autodesk.com/cloudhelp/2016/ENU/AutoCAD-DXF/files/GUID-62E5383D-8A14-47B4-BFC4-35824CAE8363.htm>.

## Clean-room oracle boundary

`D:\SeaCad\cad_2026-07-23_source` was inspected read-only as behavioral
evidence. Its legacy TEXT diagnostics highlighted unsupported justification
codes and unknown generation-bit combinations as compatibility cases. No
implementation, parser, type, fixture bytes, test code, dependency, or source
text was copied, translated, linked, vendored, or imported. Autodesk
documentation and current SeaCad scalar provenance remain normative.

## Behavior locked

- All AC1009-AC1032 dialects have ASCII/Binary classification parity.
- Horizontal codes `0..=5` map to left, center, right, aligned, middle, and
  fit.
- Vertical codes `0..=3` map to baseline, bottom, middle, and top.
- Unsupported justification codes are typed-invalid with exact raw
  provenance.
- Missing source values use the M14.2c documented zero defaults before
  classification.
- Invalid and duplicate source scalars remain typed-invalid and preserve the
  underlying issue.
- Generation flags retain the exact signed 16-bit source pattern; bits 2 and 4
  have documented helpers and all other bits remain visible.
- A nonzero valid horizontal or vertical code requires the second alignment
  point; left/baseline uses the first.
- Cancellation, family scope, raw-record bounds, source identity, and public
  `Copy`/`Send`/`Sync` contracts are covered.

## Explicit nonclaims

M14.2f does not validate height/width/angle ranges, verify second-point
component availability, select a coordinate tuple, transform OCS/WCS, decode
text, resolve styles, derive glyph geometry, edit, or write.

## Verification

All required gates passed on 2026-07-31: dependency policy, generated-schema
drift, release-evidence drift, formatting, workspace Clippy with warnings
denied, 637 workspace tests with zero failures/ignored tests, prohibited
production-macro scanning, and `git diff --check`. No dependency manifest or
lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 63 | `dc9eab7fee2f647b5e66314841bc6e5d2a5ba2d0acfc728fe1c06ba684f0b8e8` |
| `crates/seacad-dxf-core/src/lib.rs` | 680 | `39a4a695f8056eb470f04523c2af35f796a60d3b670e535b74cbf21e40de4511` |
| `crates/seacad-dxf-core/src/text_layout.rs` | 263 | `dc54a6466f30b1d592382647268e873d6221ab61a5195a0144784c08ce977bed` |
| `crates/seacad-dxf-core/tests/text_layout_tests.rs` | 338 | `236408900768081d341b5839e7d2dbbe19bac74df3ae74ae8d1e1d06a64da3e6` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 113 | `06379f04608a2960fc5fc3021d60062c462e7626f712c4491cd00c966ebfca5b` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,446 | `1ac8b3930cd737eefa4d1563e7d645e59a0996c3fa4caf89d71a2599c8a7a79c` |
| `docs/SUPPORT_MATRIX.md` | 1,143 | `2c99389bb86e35ab193831def3a5b6f7e136ca132894111f13636b698c487c39` |
