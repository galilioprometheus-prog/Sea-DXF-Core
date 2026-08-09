# M14.4ah HATCH Boundary EllipticArc Edge Semantics

M14.4ah promotes all eight M14.4ag EllipticArc numeric components to required,
domain-checked semantics. Missing values become source-anchored
MissingRequiredValue failures. Duplicate, malformed ASCII, and non-finite
Binary values retain their exact numeric issue and available raw provenance.

Autodesk's
[Ellipse edge data table](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-DC5215D6-E73F-4DFF-8BE9-01CA9610FAEE.htm)
defines the center, relative major-axis endpoint, minor/major ratio, start/end
angles, and direction flag. The official
[AcDbEllipse methods contract](https://help.autodesk.com/cloudhelp/2018/ENU/OARX-RefGuide/files/OREF-__MEMBERTYPE_Methods_AcDbEllipse.html)
requires radius ratio `1e-6..=1.0` and major-axis dot product greater than
`1e-12`. The two-component HATCH OCS major-axis vector therefore distinguishes
usable, unavailable-component, degenerate, and non-finite-derived-magnitude
states. Direction group 73 maps only 0 to Clockwise and 1 to Counterclockwise.
Start/end angles remain exact DXF degrees without normalization or geometry.

Declared/observed edge-count mismatch does not erase otherwise available
semantics. Line, CircularArc, Spline, malformed/unsupported edge types, empty
edge paths, and Polyline paths publish no EllipticArc semantic entries while
retaining the complete numeric/card/type/grouping chain. Every lookup is
bounded, cancellation is cooperative, source identity is exact, signed zero
and domain-failure bits remain exact, and Debug output does not expose raw
values.

The focused semantic suite passes 4/4 and the EllipticArc card-to-semantic chain
passes 12/12. Workspace passes 1,225/1,225 tests and all gates pass. Production
adds 465 lines: the 455-line semantic module and ten module/export lines. Tests
add 541 lines. No dependency, license, schema, release, corpus, CI, fixture, or
protected-surface change occurs. OCS/WCS geometry, Spline edge payloads, HATCH
applicability, CRUD/write, rendering, and `Complete` remain open.

The repository batch note and cumulative external `Status: READY` packet are
synchronized through M14.4ah at
`D:\SeaCad\AntigravityReports\seacad-m14.3dr-m14.4ah-hatch-boundary-elliptic-arc-edge-semantics-2026-08-10.yaml`.
No independent Antigravity PASS is claimed. This audit omits its own hash and
the self-referential active batch-note hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 276 | `fe87d5dfef9e8bff2514091f6c0ac38218a05e2396e134107b7898e8722ba9e8` |
| `README.vi.md` | 276 | `bb2ec4f7377c80c8f51242af8b284ea00048da65589da0b4e93ab6565967df41` |
| `crates/seacad-dxf-core/src/hatch_boundary_elliptic_arc_edge_semantic.rs` | 455 | `26b734025548c9ef4403d029c4d7a27ac4c074ada98e79604d89aad9daae2f28` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,489 | `eaa51d5229a7239833cfb030872fff8717003be5e481af2f43c3ebb1f52d3130` |
| `crates/seacad-dxf-core/tests/hatch_boundary_elliptic_arc_edge_semantic_tests.rs` | 541 | `d98f8b58c9c98e04e3944c3340597920ccf543d567a27babbb08c7aa7d757167` |
| `docs/IMPLEMENTATION_PLAN.md` | 424 | `8b256ed523fee5112da74bb82b15ea62bfabc2997e1fbe7a31f9c3e99a21a790` |
| `docs/SUPPORT_MATRIX.md` | 3,386 | `bd60b39b058b53e605e4284dbaecbd6cf84916323fa35cf0c792955fb6b1526e` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,311 | `41e7d82e49fe109921909614844794ad39ff72958aa007827c9377d8f602b7ed` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,736 | `4e58adb4048f37f1d188892ffa7d95b9df00a06077cd7aaeb5cd4e6664cdeb94` |
