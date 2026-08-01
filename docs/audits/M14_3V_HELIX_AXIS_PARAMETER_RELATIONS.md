# M14.3v HELIX Axis and Parameter Relations

## Scope

M14.3v composes the M14.3t scalar and M14.3u WCS-vector projections into one
typed relation entry per HELIX. It derives public geometric relationships but
does not compose the embedded SPLINE or publish analytic HELIX geometry.

## Evidence boundary

Autodesk's DXF table defines axis base, start point, axis vector, radius,
turns, turn height, handedness, and constraint type:

`https://help.autodesk.com/cloudhelp/2020/ENU/AutoCAD-DXF/files/GUID-76DB3ABF-3C8C-47D1-8AFB-72942D9AE1FF.htm`

Autodesk's managed Helix properties state that the axis vector is
perpendicular to the line joining the start and axis points, that base radius
is the distance between those points, and that turn height is the distance
between turns:

`https://help.autodesk.com/cloudhelp/2020/ENU/OARX-ManagedRefGuide/files/OARX-ManagedRefGuide-__MEMBERTYPE_Properties_Autodesk_AutoCAD_DatabaseServices_Helix.html`

The HELIX command documents a maximum of 500 turns, while the helix concept
page explicitly permits height zero for a flat 2D spiral:

`https://help.autodesk.com/cloudhelp/2020/ENG/AutoCAD-Core/files/GUID-C6FE985E-8978-4D11-8490-D81CFB323CDD.htm`

`https://help.autodesk.com/cloudhelp/2022/ENU/AutoCAD-Core/files/GUID-D887B1ED-2084-48B3-92AA-56E4A93CC766.htm`

The constraint reference confirms that height, turns, and turn height are the
three coupled editable properties. M14.3v observes their stored relationship;
it does not emulate constraint-driven edits:

`https://help.autodesk.com/view/ACD/2027/ENU/?caas=caas%2Fdocumentation%2FACD%2F2014%2FENU%2Ffiles%2FGUID-7A19A40A-5808-4D4C-9C4D-2E304BBC7DC9-htm.html`

Read-only legacy and public-repository fixtures were used only as behavioral
oracles. They exposed two risks: the legacy validator rejects zero turn height
despite Autodesk's flat-helix rule, and public files show that DXF group 40
must not be silently relabeled as the derived base radius.

| Read-only oracle | SHA-256 |
|---|---|
| `D:\SeaCad\tham khảo\New folder\cad_2026-07-23_source\crates\cad-dxf-lossless\src\semantic_helix_validation.rs` | `ab894332b09034c0197ac1bf381e36c86722f8a1ff3d8d27c68eb0d6de674cd0` |
| `D:\SeaCad\tham khảo\New folder\cad_2026-07-23_source\scratch\dxf_fixture_queue_augmented_input_20260716_b\OSGeo_gdal\autotest\ogr\data\dxf\additional-entities.dxf` | `04dfba0307cf30e1f22eb4c6bdaf06780aaa8ede14f5c40c836ddcba6002788a` |
| `D:\SeaCad\tham khảo\New folder\cad_2026-07-23_source\scratch\dxf_fixture_queue_augmented_input_20260716_b\LibreDWG_libredwg\test\test-data\2007\Helix.dxf` | `0370919383dc58d0b3d0c3c75e9993fb58cb4bf76083fa44a205864ede42c3b1` |

No oracle code or fixture is copied, translated, vendored, linked, or used at
runtime. The M14.3v fixtures are independently authored synthetic inputs.

## Contract

- Each retained HELIX has one stable relation entry linked to its exact record.
- Complete finite axis-base, start, and axis triples produce the exact radial
  difference. A nonzero finite axis is normalized, and finite base-radius and
  orthogonality residual values are published.
- Exact perpendicularity means residual equals floating-point zero. No hidden
  epsilon converts a residual into conformance.
- Missing or invalid input yields `Unavailable`; a zero axis remains distinct;
  derived non-finite radial, length, radius, residual, or height arithmetic is
  typed without publishing partial results.
- Group-40 radius remains the documented stored radius and is only classified
  as negative or nonnegative. The separately derived point distance is named
  `derived_base_radius`.
- Turns are classified as nonpositive, within the documented command limit,
  or above 500. This is an observational state, not a writer policy.
- Axial height is derived as `turns * turn_height`. Exact zero is a flat state,
  not an error.
- Construction is source-identity checked, cancellable, allocation-fallible,
  bounded by the prior directories, and adds no dependency.

## Nonclaims

M14.3v does not choose a tolerance, relabel group 40 as base or top radius,
emulate constrained-property edits, compose embedded SPLINE data, construct
analytic HELIX geometry, CRUD, write, review applicability, or advance HELIX
to `Complete`.

## Verification

The focused HELIX relation suite passes 3/3 tests and the workspace passes all
790 tests. The first workspace invocation exceeded the terminal's 120-second
capture limit and closed its output pipe; the identical command was rerun with
a longer capture limit and exited zero. Schema and release-evidence checks,
`cargo deny --locked check`, format, workspace Clippy with warnings denied,
workspace tests, the production forbidden-macro scan, and `git diff --check`
all pass. The checkpoint adds 392 physical production lines: 386 in the HELIX
relation module and six module/export lines. This audit intentionally omits its
own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 119 | `3f343cba8536bacc7876e1fa387883d6564c9f02f11d1641d271e8ac8d7e7ebf` |
| `crates/seacad-dxf-core/src/lib.rs` | 927 | `eba2d7ddd237a76f578825f915ca6c7208f668fc18a99e51f4c049b7a20dfffe` |
| `crates/seacad-dxf-core/src/helix_relation.rs` | 386 | `cafa942959fe6a65d011602bdc073cfdf33d652a3fcc8419b01055a35dfdeb7b` |
| `crates/seacad-dxf-core/tests/helix_relation_tests.rs` | 379 | `4857e568fa5d79162ad766ab89ae45364e819c77619387324a318df0ae141b0a` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 523 | `9a227c75cabb91a66c12d9da785f125a54518688901d663a127346e448bd3dab` |
| `docs/IMPLEMENTATION_PLAN.md` | 1937 | `a564dda8893a8719a67e39a0897d90e1ec02cbd5ecc24f5fa5de97ded345162d` |
| `docs/SUPPORT_MATRIX.md` | 1585 | `c2547e58a9e93f77c066abc202c2a6c76cb89c6fa51a5732f734549f32b8921f` |
