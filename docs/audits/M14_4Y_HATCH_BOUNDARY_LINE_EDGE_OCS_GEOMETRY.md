# M14.4y HATCH Boundary Line Edge OCS Geometry

M14.4y publishes one exact OCS Line segment per M14.4x coordinate entry when
both endpoints are usable. Start and end coordinates pass through without
derived arithmetic, preserving every signed-zero bit; identical endpoints
remain a valid zero-length Line. Unavailable endpoints remain typed as
`EndpointsUnavailable` with the exact four-component mask, and callers can
still resolve every lower-layer coordinate issue and provenance. Declared/
observed edge-count mismatch does not erase otherwise usable geometry. Non-Line,
invalid, and Polyline paths publish no Line geometry entries.

Autodesk documents Line-edge groups 10/20 as the OCS start point and groups
11/21 as the OCS endpoint in the
[Boundary Path Data table](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-DC5215D6-E73F-4DFF-8BE9-01CA9610FAEE.htm).

The focused suite passes 4/4 tests and workspace passes 1,189/1,189 tests. All
gates pass. No dependency, license, schema, release, corpus, or CI surface
changes. Production adds 227 lines: a 222-line module and five module/export
lines. OCS-to-WCS projection, circular/elliptic/spline edge payloads and
geometry, HATCH applicability, CRUD/write, rendering, and `Complete` remain
open.

The repository batch note and one cumulative external `Status: READY` packet
are synchronized through M14.4y at
`D:\SeaCad\AntigravityReports\seacad-m14.3dr-m14.4y-hatch-boundary-line-edge-ocs-geometry-2026-08-09.yaml`.
No independent Antigravity PASS is claimed. This audit omits its own hash and
the self-referential active batch-note hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 250 | `7928e6f46731aae2a82065a02658d0907a27d24b06006f8b1d0c251fc9189e31` |
| `README.vi.md` | 249 | `4d765e0b8a4e0dffd92e4ee13daa8c369efabb78a04a271db5018463a29bb8c0` |
| `crates/seacad-dxf-core/src/hatch_boundary_line_edge_geometry.rs` | 222 | `ab50f11f6ef9092c8eea01612d95bf5cb205f92429eb6547087eeb891dbf41fd` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,417 | `f9445515982d2463576d3cdbe155f7b2f1d4ba7933c1c1719db3631f59948ef6` |
| `crates/seacad-dxf-core/tests/hatch_boundary_line_edge_geometry_tests.rs` | 335 | `4ab1f4f7bf6e053a516035fbc7f2d9d233a06419561162ca09b2754c2a0be6eb` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `1dafe0584d3896f2d5e25c09e8c2fda7cda0bac6f23ca20e6f7513d25605c3a1` |
| `docs/SUPPORT_MATRIX.md` | 3,272 | `23a91697859fc54bd7f365e825daa7cfd0bdcaf116bdc7a99d05f961d8380973` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,177 | `05a60362d01eea0e0e3d25bd4b96ad0e105979616a7a380744f7b23a9b6690e5` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,597 | `d8158014a93dec364db8552593c7a4c2b6c3f8a7a0afef89faad6812f431c24e` |
