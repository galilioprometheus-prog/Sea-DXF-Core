# M14.4aj HATCH Boundary EllipticArc Edge WCS Geometry

M14.4aj joins every usable M14.4ai OCS EllipticArc segment to its exact HATCH
subclass elevation and explicit or defaulted extrusion. The shared M14.4s
arbitrary-axis basis projects the OCS center as a point with elevation and the
relative major-axis endpoint as a translation-free vector. Each result exposes
a finite WCS center, WCS major-axis vector, and normalized normal.

The WCS result preserves the exact source minor/major ratio, unnormalized
start/end DXF degrees, and Clockwise or Counterclockwise direction. Its complete
M14.4ai OCS geometry and elevation/extrusion receipts remain available. Derived
WCS center, vector, and normal bits are runtime geometry rather than raw-source
or cross-platform canonical evidence. No minor axis, endpoint, angle wrapping,
signed sweep, or trigonometric value is derived.

Failure precedence is source OCS geometry, elevation, extrusion, then
non-finite derived transformation. The center uses the HATCH elevation while
the relative major-axis vector receives no translation. Declared/observed edge-
count mismatch does not erase otherwise usable geometry. Line, CircularArc,
Spline, malformed or unsupported edge types, empty paths, and Polyline paths
publish no EllipticArc WCS entries.

Autodesk documents HATCH elevation groups 10/20/30 and extrusion groups
210/220/230 in the
[HATCH entity data](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-C6C71CED-CE0F-4184-82A5-07AD6241F15B.htm),
and documents EllipticArc center, relative major axis, ratio, angles, and
direction in the
[Boundary Path Data table](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-DC5215D6-E73F-4DFF-8BE9-01CA9610FAEE.htm).

The focused suite passes 4/4 tests, the EllipticArc card-to-WCS chain passes
20/20, and workspace passes 1,233/1,233 tests. All required dependency, format,
schema, release-evidence, Clippy, exact test-count, forbidden-production-pattern,
protected-surface, link, and whitespace gates pass. Production adds 347 lines:
the 310-line WCS module, 13 exact-owner resolver lines, 18 shared-basis vector-
projection lines, and six module/export lines. Tests add 501 lines. No
dependency, license, schema, release, corpus, CI, fixture, or protected-surface
change occurs. Minor-axis, endpoint, sweep, and trigonometric derivation,
Spline edge payloads, HATCH applicability, CRUD/write, rendering, and
`Complete` remain open.

The repository batch note and cumulative external `Status: READY` packet are
synchronized through M14.4aj at
`D:\SeaCad\AntigravityReports\seacad-m14.3dr-m14.4aj-hatch-boundary-elliptic-arc-edge-wcs-geometry-2026-08-10.yaml`.
No independent Antigravity PASS is claimed. This audit omits its own hash and
the self-referential active batch-note hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 282 | `8bfc760b9314b9b3124ea0559f08ab7745d07319c4303da02116f515833a42e2` |
| `README.vi.md` | 282 | `f46e3ba6a63fc80d98b7cca74b439bc1a35fbed5eb5f9f909827d8aabceaa0c7` |
| `crates/seacad-dxf-core/src/hatch_boundary_elliptic_arc_edge_geometry.rs` | 383 | `cef74a5dcad96095286bcaf36f1424449302ff54961fa777bd63c003fa6a094f` |
| `crates/seacad-dxf-core/src/hatch_boundary_elliptic_arc_edge_wcs_geometry.rs` | 310 | `bfd4d558aa5bc9165f6d44fe1b5b1c3181ab8e60332587ed0af46386a92eadd5` |
| `crates/seacad-dxf-core/src/hatch_polyline_wcs_geometry.rs` | 468 | `f10f1ed0f06a9d89c91e0d9365f412e9ed4fef38e47d7460954098f82bba5748` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,501 | `73864fd2e31aeaf90fcc90f51e9aee01f9a5a12769a985fc406f020493568c34` |
| `crates/seacad-dxf-core/tests/hatch_boundary_elliptic_arc_edge_wcs_geometry_tests.rs` | 501 | `4f530c49ac31af73400cb074c67840725771cd2d3c2b0308e842ae7749321ff4` |
| `docs/IMPLEMENTATION_PLAN.md` | 424 | `427335d17bf9e416f5586a0f9c18a7797fc4b94d0eb10a08ad4bad0290ee074c` |
| `docs/SUPPORT_MATRIX.md` | 3,413 | `e2d2b80e3a0dc256997084bc3a1baf27b4b9b416ae92f960dc36ec18259779ba` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,337 | `6686059aadf34ddf8a522ff506751726fded76014aeeb140e0e8a7336dc4e364` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,763 | `a94eb79ff6af91a71c8a3f383a5191ac31263d48637da572e481d54f7c465ec0` |
