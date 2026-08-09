# M14.4ae HATCH Boundary CircularArc Edge WCS Geometry

M14.4ae joins every M14.4ad OCS CircularArc segment to its exact HATCH
subclass elevation and explicit or defaulted extrusion. The shared M14.4s
arbitrary-axis basis projects the OCS center into WCS and publishes finite WCS
X/Y axes plus the normalized normal. The basis retains Autodesk's exact `1/64`
polar-cap branch.

The WCS result preserves the exact source radius, start/end DXF degrees, and
Clockwise or Counterclockwise direction. It does not normalize angles, derive a
signed sweep, evaluate trigonometric endpoints, or reinterpret equal angles.
Derived WCS center and basis bits are finite runtime geometry, not raw-source or
cross-platform canonical evidence.

Failure precedence is source OCS geometry, elevation, extrusion, then
non-finite derived transformation. Every entry retains its complete source
geometry directory and exact elevation/extrusion receipts. Declared/observed
edge-count mismatch does not erase otherwise usable geometry. Non-CircularArc,
invalid, empty, and Polyline states publish no WCS CircularArc entries.

Autodesk documents HATCH elevation groups 10/20/30 and extrusion groups
210/220/230 in the
[HATCH entity data](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-C6C71CED-CE0F-4184-82A5-07AD6241F15B.htm),
and documents CircularArc center, radius, angles, and direction in the
[Boundary Path Data table](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-DC5215D6-E73F-4DFF-8BE9-01CA9610FAEE.htm).

The focused suite passes 4/4 tests and workspace passes 1,213/1,213 tests. All
required dependency, format, schema, release-evidence, Clippy, exact test-count,
forbidden-production-pattern, and whitespace gates pass. Production adds 342
lines: a 315-line WCS module, 13 exact-owner resolver lines, eight shared-basis
axis accessor lines, and six module/export lines. Tests add 483 lines. No
dependency, license, schema, release, corpus, CI, fixture, or protected-surface
change occurs. Angle wrapping, signed sweep, endpoint derivation, the other
edge payload families, HATCH applicability, CRUD/write, rendering, and
`Complete` remain open.

The repository batch note and cumulative external `Status: READY` packet are
synchronized through M14.4ae at
`D:\SeaCad\AntigravityReports\seacad-m14.3dr-m14.4ae-hatch-boundary-circular-arc-edge-wcs-geometry-2026-08-10.yaml`.
No independent Antigravity PASS is claimed. This audit omits its own hash and
the self-referential active batch-note hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 267 | `dab50c56dfb316a6472e47b1bf9ca0c8862de7cdd7000623e8653ddcba905c72` |
| `README.vi.md` | 267 | `407eb082f1035ea39a2cc0238892792e8154273036ca6854101eac6120e9cc69` |
| `crates/seacad-dxf-core/src/hatch_boundary_circular_arc_edge_geometry.rs` | 322 | `916a0518c88c2c09e1fa824d253b701ebd77b52a34ac79772ec67206ca957b13` |
| `crates/seacad-dxf-core/src/hatch_boundary_circular_arc_edge_wcs_geometry.rs` | 315 | `9b9dcacdf5987719f05a500d73918c7dc72ba56dc2144ab6ac938135289572be` |
| `crates/seacad-dxf-core/src/hatch_polyline_wcs_geometry.rs` | 450 | `d0f0f4ca229cde1dcfc0fd159fe728805461069865e7df00a31fd32a827ab4a2` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,459 | `40e9d5bd3b5b041e21b42f5ec49cf45e21106ad6fbbc4e1121ced2acb5875871` |
| `crates/seacad-dxf-core/tests/hatch_boundary_circular_arc_edge_wcs_geometry_tests.rs` | 483 | `9bdfd04070ebd2002cc80476e100077bb6d67e78b9d0d1692205db64a3c044cf` |
| `docs/IMPLEMENTATION_PLAN.md` | 424 | `480759a6c908073aac76b31fbed8ea8fc0b749f6e48e3eee79bac85926c8b684` |
| `docs/SUPPORT_MATRIX.md` | 3,346 | `46b9be5d74f4fbef0e53cedbb98e3bf7411b4573abef86891d16427ab3616199` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,264 | `d7d636e09afc5736d21a28598a1d20390d64137a980a72d50db3819dbb0b2e12` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,689 | `6959a05ed3bf14e0c7e2464a363afa1fd052df7f24507dcd9baa0be8b4b06614` |
