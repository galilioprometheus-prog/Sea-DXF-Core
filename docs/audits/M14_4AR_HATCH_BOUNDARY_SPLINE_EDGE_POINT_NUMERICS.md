# M14.4ar HATCH Boundary Spline Edge Point Numerics

M14.4ar selects and decodes numeric components for every M14.4ap HATCH boundary
Spline point tuple. It does not require a Y component, apply the documented
weight default, compare declared counts, classify knots or tangents, validate
NURBS topology, or derive OCS/WCS geometry.

Autodesk's
[Boundary Path Data table](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-DC5215D6-E73F-4DFF-8BE9-01CA9610FAEE.htm)
defines control and fit data as OCS 2D points and group 42 as an optional
control-point weight. The M14.4ap X anchor is already unique by construction;
M14.4aq supplies independent Y and Weight cardinality without winner selection.

Each tuple receives one numeric entry. Its X anchor decodes directly, Y selects
only a Unique card, and control-point Weight selects only a Unique card. Fit
points publish no Weight value because that role is inapplicable. Finite values
become Explicit with exact field/raw provenance and signed-zero bits. Absent
cards remain Absent, Multiple cards become a typed issue without invented raw
provenance, malformed ASCII numbers retain their exact lexical issue, and non-
finite Binary doubles retain their exact payload and raw provenance.

Available Spline sequences with no tuples publish empty numeric slices. An
unavailable M14.4ap sequence publishes no entries while its lower issue remains
reachable through the retained card/tuple chain. Non-Spline edges publish no
numeric entry, and declared/observed edge-count mismatch does not erase usable
point entries. All nine Core dialects retain ASCII/Binary parity.

The test-first compile failed only because the new public API did not yet
exist. The focused suite passes 4/4 tests and the edge grouping-to-point-
numeric chain passes 28/28. Workspace passes 1,265/1,265 tests and all required
dependency, format, schema, release-evidence, Clippy, exact test-count,
forbidden-production-pattern, protected-surface, link, and whitespace gates
pass.

Production adds 361 lines: a 355-line module and six module/export lines. Tests
add 453 lines. No dependency, license, schema, release, corpus, CI, fixture, or
protected-surface change occurs. Required point semantics, default weight 1,
knot/header/fit-count relations, tangent cardinality, topology, OCS/WCS
geometry, applicability, CRUD/write, rendering, and `Complete` remain open.

The repository batch note and cumulative external `Status: READY` packet are
synchronized through M14.4ar at
`D:\SeaCad\AntigravityReports\seacad-m14.3dr-m14.4ar-hatch-boundary-spline-edge-point-numerics-2026-08-10.yaml`.
No independent Antigravity PASS is claimed. This audit omits its own hash and
the self-referential active batch-note hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 308 | `dbd6d6bd567e8656c5d3c9be6d52ff4141341395f09bb19bdbef51bce663b1bd` |
| `README.vi.md` | 307 | `198d4308cd8c14dcd06763cb05989fed8a2b8093562d5e38bf6cfaf65588811d` |
| `crates/seacad-dxf-core/src/hatch_boundary_spline_edge_point_numeric.rs` | 355 | `17eb53aa27e5051fc5b48e4eea8b8f0eb49c3ecc10f307c167fc51022b800eab` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,556 | `6de77f8bc0593eaa61d3e2de107881dcb155764017323a0dfa435b05bf18b2e1` |
| `crates/seacad-dxf-core/tests/hatch_boundary_spline_edge_point_numeric_tests.rs` | 453 | `1b7b1c51069a8f8af1e4452c1ff9522e15a980d69e5620fcb45ff3d1a961da4a` |
| `docs/IMPLEMENTATION_PLAN.md` | 424 | `24b3e57276de00b8600ce8ae38fb1e79909a2dfb3b3db94697adeb28c3a9db25` |
| `docs/SUPPORT_MATRIX.md` | 3,514 | `89234566a71ed03c8a70e8f1b21f25338117264fd06835c3ee6fce12917c10f8` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,438 | `bd474d11bd5531e5981b6d53aa8343292370a7d38b4daf14a2dadb8d07e27b5d` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,868 | `1170d76af14c368988868e9de9a94be22fff024da32d59bc6ec4d07a488ec9d3` |
