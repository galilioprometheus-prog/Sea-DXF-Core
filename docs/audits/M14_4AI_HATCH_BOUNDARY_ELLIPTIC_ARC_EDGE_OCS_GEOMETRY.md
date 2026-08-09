# M14.4ai HATCH Boundary EllipticArc Edge OCS Geometry

M14.4ai publishes one exact-value OCS EllipticArc segment for every M14.4ah
entry whose eight required scalar semantics and reviewed major-axis relation
are usable. The segment retains the exact OCS center, relative major-axis
endpoint vector, minor-to-major ratio, start/end angles in DXF degrees, and
reviewed Clockwise or Counterclockwise direction.

Autodesk defines those fields in the
[Ellipse edge data table](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-DC5215D6-E73F-4DFF-8BE9-01CA9610FAEE.htm).
The official
[AcDbEllipse methods contract](https://help.autodesk.com/cloudhelp/2018/ENU/OARX-RefGuide/files/OREF-__MEMBERTYPE_Methods_AcDbEllipse.html)
defines the ratio and major-axis domains enforced by M14.4ah. This checkpoint
performs no arithmetic. Center and relative-axis coordinates, ratio,
unnormalized negative or large angles, signed zero, and direction pass through
bit-exactly. It derives no minor axis, sweep, endpoint, trigonometric value, or
WCS coordinate.

If any scalar semantic is unavailable, geometry returns an exact
eight-component mask while the complete M14.4ah semantic and M14.4ag numeric
issue/provenance chain remains resolvable. A degenerate or overflowed
major-axis squared magnitude remains a separate typed geometry issue. Declared
and observed edge-count mismatch does not erase otherwise usable geometry.
Line, CircularArc, Spline, malformed or unsupported edge types, empty paths,
and Polyline paths publish no EllipticArc OCS entries.

The focused suite passes 4/4 tests and the EllipticArc card-to-OCS chain passes
16/16. Workspace passes 1,229/1,229 tests. All required dependency, format,
schema, release-evidence, Clippy, exact test-count, forbidden-production-pattern,
protected-surface, link, and whitespace gates pass. Production adds 376 lines:
the 370-line OCS module and six module/export lines. Tests add 428 lines. No
dependency, license, schema, release, corpus, CI, fixture, or protected-surface
change occurs. Angle wrapping, signed sweep, endpoint or minor-axis derivation,
OCS-to-WCS projection, Spline edge payloads, HATCH applicability, CRUD/write,
rendering, and `Complete` remain open.

The repository batch note and cumulative external `Status: READY` packet are
synchronized through M14.4ai at
`D:\SeaCad\AntigravityReports\seacad-m14.3dr-m14.4ai-hatch-boundary-elliptic-arc-edge-ocs-geometry-2026-08-10.yaml`.
No independent Antigravity PASS is claimed. This audit omits its own hash and
the self-referential active batch-note hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 279 | `4068b79e7bc3a97308148d71858d3235a56567ef380bc67ed3a410afd0fa89a2` |
| `README.vi.md` | 279 | `1f4840ad160f38d84ff3e55e2a99bd03b5845ea78d4dbe8dd1166e370110e84e` |
| `crates/seacad-dxf-core/src/hatch_boundary_elliptic_arc_edge_geometry.rs` | 370 | `9132896ab8d224138f64c97f2af508f50077b00652091219b84440b1c34f5b67` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,495 | `e9e382c8deae6d286a6414c1ed16fce206543825e4999c26df11aabbcba2c90e` |
| `crates/seacad-dxf-core/tests/hatch_boundary_elliptic_arc_edge_geometry_tests.rs` | 428 | `0035376706fa201d64f3d7f17e68df820d0c59b697018d1565ed9eb9a89f665e` |
| `docs/IMPLEMENTATION_PLAN.md` | 424 | `7822154fccaa27218107cb815dd56897f09303c543706fe939bc5d51624a5930` |
| `docs/SUPPORT_MATRIX.md` | 3,399 | `98332dfbf40bf1c886a9819bca299f209a582b6038f4be1739b18ad4fc80db82` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,324 | `dcb7000031c3866daabcc7fc5e44651b8fcb870f0cc7dd31cce9e4602ac7b3c7` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,749 | `fbff9864817a852795a2773bff9766bd484433608d2c6909eefeb5f4454c5e2b` |
