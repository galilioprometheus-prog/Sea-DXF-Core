# M14.3cs Entity XDATA Coordinate Transform

Retrieved: 2026-08-04

## Normative and behavioral basis

Autodesk assigns separate transformation behavior to generic XDATA 3D groups:
group 1010 is a plain point; group 1011 is a world-space position transformed
with its entity; group 1012 is a world-space displacement not moved with its
entity; and group 1013 is a world direction not moved or scaled:

<https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-A2A628B0-3699-4740-A215-C560E7242F63.htm>

AutoCAD 2027 Core Console X.60.0.0 was used as a behavioral oracle. MOVE by
`(10,20,30)` changed only 1011 from `(1,2,3)` to `(11,22,33)`. SCALE by two
about the origin changed 1011 and 1012 to `(2,4,6)` while 1010 and 1013 stayed
unchanged. SCALE by two about `(10,0,0)` changed 1011 to `(-8,4,6)` and 1012
to `(2,4,6)`. ROTATE 90 degrees about Z changed 1011, 1012, and 1013 to
`(-2,1,3)`. MIRROR across the YZ plane changed those same roles to
`(-1,2,3)`. Group 1010 stayed unchanged in every operation.

## Contract

`DxfEntityXDataCoordinateTransform` retains three private coordinated channels:
an affine position matrix, a linear displacement matrix, and a rotation/mirror
direction matrix. Validated factories create translations, uniform scales about
a base point, axis rotations about a base point, and mirror-plane reflections;
`then` composes the channels in operation order. Non-finite inputs, zero scale,
zero axes/normals, and non-finite composed matrices fail typed.

`DxfEntityXDataCoordinateTransformDirectory` emits one source-bound entry for
every M14.3cp tuple. Complete typed finite tuples publish exact original and
transformed values. Group 1010 bypasses every transform, 1011 uses the position
channel, 1012 uses the displacement channel, and 1013 uses the direction
channel. Partial tuples, invalid component values, and non-finite derived
results remain typed unavailable states; no partial result is published.

## Verification boundary

The focused suite pairs ASCII and Binary fixtures for AC1009, AC1012, AC1014,
AC1015, AC1018, AC1021, AC1024, AC1027, and AC1032. It covers all four tuple
roles, the composed scale-about-base/rotation/translation/mirror oracle,
partial and invalid tuples, derived overflow, every factory rejection,
composition overflow, source identity, cancellation, lookup bounds, metadata
bounds, and non-disclosing debug output.

This checkpoint does not interpret application-specific payloads, resolve or
remap group-1005 handles, write transformed values, clone XDATA, or advance any
entity to `Complete`.

## Gate receipts

The focused coordinate-transform suite passed 3/3 tests and the full workspace
passed all 992 listed tests across 187 targets. Generated schema and release-
evidence checks, `cargo deny --locked check`, formatting, workspace Clippy with
warnings denied, the production forbidden-construct scan, and
`git diff --check` all passed. No manifest, lockfile, dependency, generated
schema, locale source, committed fixture, or external corpus changed.

Production adds 607 lines: a 261-line source-bound projection directory, a
335-line validated affine math/configuration module, and 11 root registration/
export lines. This is above the preferred 200--500-line aggregate micro-
milestone range, while each implementation file remains below 400 lines; the
split keeps transformation algebra independent from raw-source projection and
avoids a partially usable role contract. The focused target adds 357 lines.
This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 457 | `c315682a3937d42639a8d10bf7e33357376982e7f3d33fc741e3e74498408895` |
| `crates/seacad-dxf-core/src/entity_xdata_coordinate_transform.rs` | 261 | `d0f9fbb93edb9768f6053b5a6aa0edfd81ba24294b214a30e1d2df6a4dcec192` |
| `crates/seacad-dxf-core/src/entity_xdata_coordinate_transform_math.rs` | 335 | `a833296f2ce98bdfdf3e8167142a7b93f865ae44549dcb4fc11eeb32962f98e9` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,118 | `313ac2c4c973424aaa17a24fe30aa8906f59fb5f5cef63c6c9f21a0df847ab8c` |
| `crates/seacad-dxf-core/tests/entity_xdata_coordinate_transform_tests.rs` | 357 | `3ac9f6e138629c6e0d2fbd63277844439dd274e061b4b396564723f4c40f2e1e` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,433 | `6f53dc0f5046178928be563de1139a951082805b123ab15db082d91242cadf0b` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,850 | `c55f4ce87b11cbe5fab218b94da68d0f4e1b11f2e0f8c860cf0b5e610da7b4f7` |
| `docs/SUPPORT_MATRIX.md` | 2,491 | `6f1c96876d8589c63a3e9fa358799cef4de9e85a30cf41bc8a2d873e5871743d` |
