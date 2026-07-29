# M9.2l Classic POLYLINE Segment Geometry

Retrieved: 2026-07-30

## Normative boundary

Autodesk [POLYLINE (DXF)](https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-ABF6B778-BE20-4B49-9B58-A94E64CEFFF3.htm)
places classic 2D elevation in the parent POLYLINE's OCS dummy point and 3D
locations in WCS. [VERTEX (DXF)](https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-0741E831-599E-4CBF-91E1-8ADBCFD6556D.htm)
defines bulge as the tangent of one quarter of the signed included angle.
Autodesk's [AcDb2dVertex position contract](https://help.autodesk.com/cloudhelp/2018/ENU/OARX-RefGuide/files/OREF-AcDb2dVertex__position.html)
states that 2D vertex Z is kept in the owner and obtained from the polyline
elevation; its [AcDb2dVertex guide](https://help.autodesk.com/cloudhelp/2025/JPN/OARX-DevGuide/files/GUID-0A297115-E5D9-4699-BBB8-DA0185D2D946.htm)
further states that a stored vertex Z is otherwise ignored. Autodesk's
[3DPOLY contract](https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-Core/files/GUID-10E0EDAB-BF4C-442C-93DA-E516F6DEAA7B.htm)
states that 3D polylines are sequences of straight segments, may be
non-coplanar, and cannot contain arc segments.

## Implementation contract

`crates/seacad-dxf-core/src/polyline_segment_geometry.rs` lazily derives one
typed geometry result for each M9.2j segment. Two-dimensional geometry retains
VERTEX X/Y and parent elevation in OCS, preserving zero-bulge lines and
deriving finite circular center, radius, and signed sweep from nonzero bulge.
Three-dimensional geometry retains exact WCS endpoint tuples and permits only
straight segments. Unavailable endpoint/elevation/bulge values, contradictory
nonzero 3D bulge, degenerate arc chords, and non-finite derivation fail typed.

## Test evidence

`crates/seacad-dxf-core/tests/polyline_segment_geometry_tests.rs` covers
ASCII/Binary parity across all nine supported dialects, straight 2D geometry,
positive arc geometry, ignored stored 2D VERTEX Z, parent elevation, exact 3D
WCS lines, every typed failure, lookup bounds, cancellation, source identity,
and public traits.

## Non-claims

M9.2l does not transform OCS to WCS, validate extrusion, select effective
widths, tessellate, diagnose file conformance, edit, write, or render.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, formatting,
generated-schema drift, workspace Clippy with warnings denied, 401 workspace
tests with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/polyline_segment_geometry.rs` | 303 | `cc1a2d0eb0b2d882b2c6464514390093a66b71ace0ae10249afd1534f6e45ee5` |
| `crates/seacad-dxf-core/src/lib.rs` | 374 | `9e1308180fda32dc11d31a8c7136c7788bef0848d47db37f5fdce74f9dd56a90` |
| `crates/seacad-dxf-core/tests/polyline_segment_geometry_tests.rs` | 343 | `b71ef1e7633c7ccc9e28b568c18d9864b6237e9cf5ef2afeaffcc29a0e347c96` |
| `docs/IMPLEMENTATION_PLAN.md` | 624 | `14a124783c7ebd0152cff8576f3812f3d6030a89138fe6667045292775de5880` |
| `docs/SUPPORT_MATRIX.md` | 527 | `41814dd309002e0201e8dcdf83d73df57837def23b45cdab63721b2607d61ced` |
