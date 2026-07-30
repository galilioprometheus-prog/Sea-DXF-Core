# M9.2r Classic POLYLINE Polyface WCS Geometry

Retrieved: 2026-07-30

## Normative boundary

Autodesk [VERTEX (DXF)](https://help.autodesk.com/cloudhelp/2020/ENU/AutoCAD-DXF/files/GUID-0741E831-599E-4CBF-91E1-8ADBCFD6556D.htm)
defines groups `10/20/30` as a WCS location for 3D vertices, states that a
polyface coordinate VERTEX has flag bits `128|64`, and says its location groups
supply the mesh coordinate. It separately says face-definition VERTEX location
groups are irrelevant. Autodesk [PFACE](https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-Core/files/GUID-4B0667EF-D8E3-4BD2-A3EE-06CFF164A0A6.htm)
defines polyface meshes as 3D and permits point-, line-, and polygon-like faces.

## Implementation contract

`crates/seacad-dxf-core/src/polyline_polyface_geometry.rs` assembles each
resolved M9.2q corner's referenced coordinate VERTEX components into an exact
WCS point. Every point retains its resolved corner, signed source index, edge
visibility, and coordinate identity. Face-resolution failures and unusable
coordinate components retain typed zero-point face states; component semantic
states remain explicit.

## Test evidence

`crates/seacad-dxf-core/tests/polyline_polyface_geometry_tests.rs` covers
ASCII/Binary parity across all nine dialects, exact WCS bits, odd ordering,
coordinate identity and edge visibility retention, component-level failure,
upstream face-resolution failure, lookup bounds, cancellation, source identity,
and public traits.

## Non-claims

M9.2r does not consult irrelevant face-record locations, derive edges, validate
degeneracy, winding, planarity, manifoldness, or normals, triangulate, edit,
write, or render.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, formatting,
generated-schema drift, workspace Clippy with warnings denied, 420 workspace
tests with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/polyline_polyface_geometry.rs` | 346 | `f286b7e74240d3d4adcceea1f7fa25abf7800eb5522999893fcffcbe842907eb` |
| `crates/seacad-dxf-core/src/lib.rs` | 414 | `7f93f0c2d7377db4767b39cdd03e02b0e6642c2665f04264c27ca252234d4704` |
| `crates/seacad-dxf-core/tests/polyline_polyface_geometry_tests.rs` | 258 | `1327e26d0d2d705177185ed888161c9635541b6608fe03f9d91ffe413f31b726` |
| `docs/IMPLEMENTATION_PLAN.md` | 695 | `dc11734aad7aab8c7d47a19e81a59041ef16f81fbd69a2dda06dad46ed0d2d49` |
| `docs/SUPPORT_MATRIX.md` | 586 | `3351deee9ce104f3fa98a7d61c2f45bf332cf08ec0620a620315269c451da8e6` |
