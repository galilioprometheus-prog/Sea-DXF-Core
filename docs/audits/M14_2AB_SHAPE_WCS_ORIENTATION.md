# M14.2ab SHAPE WCS Orientation

## Scope

M14.2ab projects a SHAPE entity's optional rotation onto its normalized
extrusion plane. It exposes finite orthonormal WCS x/y axes and the plane
normal without transforming the SHAPE insertion point, which the DXF contract
already defines in WCS.

## Normative basis

- Autodesk's SHAPE entity reference defines groups 10/20/30 as a WCS insertion
  point, group 50 as an optional rotation angle with a zero default, and groups
  210/220/230 as an optional extrusion direction with a `(0, 0, 1)` default:
  <https://help.autodesk.com/cloudhelp/2018/ENU/AutoCAD-DXF/files/GUID-0988D755-9AAB-4D6C-8E26-EC636F507F2C.htm>
- Autodesk's DXF group-code reference defines group 50-58 values as angles in
  degrees for file input and output:
  <https://help.autodesk.com/cloudhelp/2027/ENU/OARX-RefGuide/files/OARX-RefGuide-DXF_Group_Codes.html>
- Autodesk's arbitrary-axis algorithm defines the orthonormal basis generated
  from an extrusion direction:
  <https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-E19E5B42-0CC7-4EBA-B29F-5E1D595149EE.htm>

## Contract

- Absent rotation is zero degrees and preserves the arbitrary-axis x/y basis.
- Finite rotation is converted from degrees to radians and rotates the basis
  x/y axes within the normalized extrusion plane.
- Successful output contains finite, mutually orthogonal unit x/y/normal axes
  with canonical positive zero.
- The WCS insertion point is never transformed by orientation projection.
- Missing, invalid, duplicate, non-finite, zero-extrusion, and basis failures
  remain typed, while lower semantic/layout/raw evidence stays reachable.

## Coverage and size

Tests cover all nine supported dialects in both ASCII and Binary framing for
the default, 90-degree, non-polar-normal, and negative-angle/negative-normal
cases. They also cover orthonormality, invalid and duplicate scalar evidence,
Binary NaN/infinity, zero extrusion, cancellation, family scoping, raw lookup,
source identity, and public semantic traits.

The changed production modules remain below 300 lines, and the focused test
module remains below 400 lines. No new dependency or duplicated locale catalog
is introduced.

## Nonclaims

This checkpoint does not claim SHAPE definition resolution, style lookup,
size, relative x scale, oblique angle, thickness, glyph geometry, editing, or
writing.

## Verification

All required gates passed on 2026-07-31: dependency policy, generated-schema
drift, release-evidence drift, formatting, workspace Clippy with warnings
denied, 699 workspace tests with zero failures or ignored tests, production
forbidden-macro scanning, and `git diff --check`. The focused SHAPE
orientation suite passed 4/4 tests. No dependency manifest or lockfile changed.

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 79 | `2c2e7b921d01a021d76eecefe840cd8ce697b8b69e8d764d899b67abc7b60019` |
| `crates/seacad-dxf-core/src/lib.rs` | 770 | `acec9b5b15f6cd8b1fb5bd22fffa58fc07a97d51fca3691892b35cfcb453397b` |
| `crates/seacad-dxf-core/src/text_symbol_ocs_projection.rs` | 165 | `dc8625863f590a5dd2df2292a7a1f5ed1d1795bf01d7501f94b82d3955d32cc3` |
| `crates/seacad-dxf-core/src/shape_wcs_orientation.rs` | 278 | `c790e8eaaa82924c760722e03eb405f6aa90938a75aa77c74c9d1904f262cf10` |
| `crates/seacad-dxf-core/tests/shape_wcs_orientation_tests.rs` | 381 | `3566ee261ec4ef66e3db18bf8383d34df8cbbc1c81664ae69860eebe13b3f8d7` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 260 | `de040df1ac54a4e13e0b9e2c7bacdb70c3d9986f77a31b181b9076f6c39f186c` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,613 | `e07ce8df795d6d7717784bdf310c4fd2cc4e7bbaeea98fccc8970bb0ae5e2e64` |
| `docs/SUPPORT_MATRIX.md` | 1,309 | `15f32c1fd5415b0291aa47238d093eae6f8b30bf3d66c3ed9edcc9aa94e2bf8d` |
