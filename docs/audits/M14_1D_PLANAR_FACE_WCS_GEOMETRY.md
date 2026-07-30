# M14.1d Planar-Face WCS Geometry

Retrieved: 2026-07-31

## Normative boundary

Autodesk's [3DFACE
reference](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-747865D5-51F0-45F2-BEFE-9572DBC5B151.htm)
defines its four corners in WCS and specifies that an absent fourth corner
inherits the third. Autodesk's [SOLID
reference](https://help.autodesk.com/cloudhelp/2018/ENU/AutoCAD-DXF/files/GUID-E0C5F04E-D0C5-48F5-AC09-32733E8848F2.htm)
and [TRACE
reference](https://help.autodesk.com/cloudhelp/2018/ENU/AutoCAD-DXF/files/GUID-EA6FBCA8-1AD6-4FB2-B149-770313E93511.htm)
define their corners in OCS, with zero thickness and (0,0,1) extrusion
defaults. Autodesk's [arbitrary-axis
algorithm](https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-E19E5B42-0CC7-4EBA-B29F-5E1D595149EE.htm)
defines the OCS basis used to project SOLID and TRACE corners into WCS.

## Legacy evidence boundary

`D:\SeaCad\cad_2026-07-23_source` was consulted only as a read-only behavioral
oracle. Its normalized-view evidence identifies 3DFACE as WCS, SOLID/TRACE as
OCS, SOLID/TRACE trailing-corner order, and zero extrusion as compatibility
risks. No legacy parser or geometry implementation was copied, translated,
vendored, or linked. Autodesk documentation remains normative.

## Implementation contract

`planar_face_wcs_geometry.rs` exposes a lazy WCS geometry directory derived
from the M14.1c typed semantic directory:

- 3DFACE keeps its documented WCS source-corner order;
- SOLID and TRACE reorder source corners to perimeter order
  `[first, second, fourth, third]`;
- SOLID and TRACE use the crate's shared arbitrary-axis implementation to
  project OCS coordinates into WCS;
- the result retains a normalized WCS normal and finite thickness; and
- positive zero is canonicalized for deterministic geometry values.

Every source corner, thickness, and extrusion value must first be available
and finite. Zero extrusion and non-finite derived geometry remain explicit,
typed failures. Construction owns the semantic directory, checks source
identity, observes cancellation, and supports bounded source-record lookup.

## Test evidence

`planar_face_wcs_geometry_tests.rs` covers ASCII/Binary parity across all nine
AC1009--AC1032 dialects; 3DFACE WCS order; SOLID/TRACE perimeter reordering;
nontrivial OCS-to-WCS projection; normalized normals; positive-zero thickness;
missing corners; zero extrusion; invalid or non-finite thickness, extrusion,
and coordinates; source identity; lookup bounds; cancellation; and public
`Copy`/`Send`/`Sync` bounds.

## Non-claims

M14.1d does not interpret 3DFACE invisible-edge bits, reject or erase unknown
edge bits, apply thickness to construct surfaces, assemble mesh topology,
expand block references, edit, write, render, or tessellate. It does not claim
typed semantic completion for all currently recognized DXF entity families.

## Required checkpoint gates

All required gates passed on 2026-07-31: dependency policy, generated-schema
drift, release-evidence drift, formatting, workspace Clippy with warnings
denied, 615 workspace tests with zero failures/ignored tests, and
`git diff --check`. No dependency manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 78 | `567a75725f38bb87311d61b8e61e253b233486338fc8c4c3d6fa799a641fef42` |
| `crates/seacad-dxf-core/src/lib.rs` | 649 | `93a9522f245bf13d08b3a4239c4ee681828ee24126f42582b84a854a18a49c78` |
| `crates/seacad-dxf-core/src/planar_face_wcs_geometry.rs` | 278 | `90e8bebb29885db4e51cc8621b2165b91410d09151f31257fac50c603b3e6648` |
| `crates/seacad-dxf-core/tests/planar_face_wcs_geometry_tests.rs` | 336 | `87f8b6f609753def8a70fbbe4dbccf3513e8ab47c62893a5da77a5e3cb2254c2` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 99 | `ae3ff615190410d0d8f6138637dd7d8acf37b4d812b18a7a94f4e9a3371fead5` |
| `docs/IMPLEMENTATION_PLAN.md` | 1380 | `8a931a567092193fe706e8a32a8a9acb47f31b7dd9f999fd813b1d33f1f0fa3e` |
| `docs/SUPPORT_MATRIX.md` | 1102 | `cfb8830176f1f523e8734285210733ffe376fa858924fa1ce92a5f21aa70ef00` |
