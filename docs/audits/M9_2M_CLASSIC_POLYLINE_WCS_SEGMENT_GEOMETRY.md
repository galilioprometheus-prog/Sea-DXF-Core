# M9.2m Classic POLYLINE WCS Segment Geometry

Retrieved: 2026-07-30

## Normative boundary

Autodesk [Object Coordinate Systems in DXF](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-D99F1509-E4E4-47A3-8691-92EA07DC88F5.htm)
states that planar 2D POLYLINE coordinates are in OCS, whose Z axis is the
extrusion direction and whose elevation shifts the entity plane along that
axis; 3D POLYLINE coordinates are already in WCS. Autodesk's
[Arbitrary Axis Algorithm](https://help.autodesk.com/cloudhelp/2015/ENU/AutoCAD-DXF/files/GUID-E19E5B42-0CC7-4EBA-B29F-5E1D595149EE.htm)
defines the deterministic OCS X axis, the exact `1/64` square polar-cap branch,
and the right-handed Y axis for a unit normal.

## Implementation contract

`crates/seacad-dxf-core/src/polyline_segment_wcs_geometry.rs` lazily projects
each usable M9.2l classic 2D line or arc from OCS into WCS. It robustly
normalizes finite extrusion vectors, applies the documented arbitrary-axis
basis, transforms endpoints/center/elevation, and retains the normalized WCS
normal, radius, signed sweep, and bulge. Native classic 3D WCS lines pass
through exactly without consulting extrusion. Source-geometry failures,
unavailable extrusion, zero-length extrusion, and non-finite results remain
typed per segment.

## Test evidence

`crates/seacad-dxf-core/tests/polyline_segment_wcs_geometry_tests.rs` covers
ASCII/Binary parity across all nine supported dialects, non-unit extrusion,
both arbitrary-axis branch families, default extrusion, elevation translation,
transformed straight/arc geometry, exact native 3D pass-through, source errors,
invalid and zero-length extrusion, arithmetic overflow, lookup bounds,
cancellation, source identity, and public traits.

## Non-claims

M9.2m does not claim external-engine geometric conformance, validate thickness
or planarity, select effective widths, tessellate, diagnose file conformance,
edit, write, or render.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, formatting,
generated-schema drift, workspace Clippy with warnings denied, 404 workspace
tests with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/polyline_segment_wcs_geometry.rs` | 309 | `1769c9b63869f678aeebed371d3c0d1adf77d7b06c8a6fb95d88e1be6e77bdc4` |
| `crates/seacad-dxf-core/src/lib.rs` | 380 | `5dd94d2f1fdbe211ba40ce4321009f7c5a397acccb906212c98d7b8361e35a7b` |
| `crates/seacad-dxf-core/tests/polyline_segment_wcs_geometry_tests.rs` | 374 | `952f9e5731e2f58f0b0deb40697f15d4cd00125244ee6d6bb80e9080d3923a08` |
| `docs/IMPLEMENTATION_PLAN.md` | 636 | `7bb20a4189fa6b634c6692774cadf7567ae96f1e3067a336be140982d4c699d8` |
| `docs/SUPPORT_MATRIX.md` | 537 | `e1d11f9891a043f2c7c8d536009c6fa796db5085c04aff1a1a5a0bc1947a5ccd` |
