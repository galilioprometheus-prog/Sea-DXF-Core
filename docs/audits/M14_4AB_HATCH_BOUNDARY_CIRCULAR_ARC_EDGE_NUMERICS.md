# M14.4ab HATCH Boundary CircularArc Edge Numerics

M14.4ab selects every M14.4aa CircularArc card into one source-stable numeric
entry. Unique groups 10/20/40/50/51 decode as finite binary64 values and unique
group 73 decodes as an exact signed Int16. This checkpoint deliberately does
not require fields, validate positive radius, normalize angles, or restrict the
direction flag to 0/1.

Autodesk assigns center, radius, start/end angle, and counterclockwise flag in
the
[Arc edge data table](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-DC5215D6-E73F-4DFF-8BE9-01CA9610FAEE.htm).

Every Explicit value retains exact `entity.hatch` field and raw provenance.
Absent cards remain Absent without defaults; Multiple cards retain their count
without selecting a winner. Malformed ASCII doubles and integers remain
InvalidAsciiNumber with raw provenance, while NaN and infinities remain
NonFiniteDouble with their exact bits. Edge-count mismatch does not erase a
usable entry; non-CircularArc, invalid, empty, and Polyline states publish none.

The focused suite passes 4/4 tests; card and Line-numeric regressions each pass
4/4. Workspace passes 1,201/1,201 tests and all gates pass. Production adds 398
lines: a 390-line module and eight module/export lines. Tests add 377 lines. No
dependency, license, schema, release, corpus, CI, fixture, or protected-surface
change occurs. Requiredness and domain semantics, OCS/WCS geometry, other edge
payloads, HATCH applicability, CRUD/write, rendering, and `Complete` remain
open.

The repository batch note and cumulative external `Status: READY` packet are
synchronized through M14.4ab at
`D:\SeaCad\AntigravityReports\seacad-m14.3dr-m14.4ab-hatch-boundary-circular-arc-edge-numerics-2026-08-10.yaml`.
No independent Antigravity PASS is claimed. This audit omits its own hash and
the self-referential active batch-note hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 258 | `fe464a7f74a90dbaca1a6dc2804595cc9459c063e36c3dea07b05a3d1a5d293b` |
| `README.vi.md` | 258 | `cf3dfa1cfc058d18cac013d3c96f019aa10a67510686742468926c7724f7960c` |
| `crates/seacad-dxf-core/src/hatch_boundary_circular_arc_edge_numeric.rs` | 390 | `c6a4a220d203816b04b538ba571423a4e2b338f9ddae00854bd17e4a1d51bbfa` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,438 | `c66c3ee21508fa28672c7329768612373d11426341d6077a286e7cda5c8c468c` |
| `crates/seacad-dxf-core/tests/hatch_boundary_circular_arc_edge_numeric_tests.rs` | 377 | `c93905c601cf84e30b7393b6358fc37647afd451cfcbe2624a48bbf3ee7b972c` |
| `docs/IMPLEMENTATION_PLAN.md` | 424 | `bf2055c8fe5fef55e2f66beeaa94b8906a9ff1387f2930b0ebe3b57675ff356e` |
| `docs/SUPPORT_MATRIX.md` | 3,307 | `3f2b1fa4e62d5245f97b1f9a72800aab9fb286bf1e95a9a90fdebdf3cbf9e2b4` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,226 | `8116e4a621bb72c20cf95763a08c01e87da7597a77dbec1298945c67597bb842` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,648 | `4830b572b193ac73b33ce87ce53a3b0a32a16fae0acf19a64f7787e26eec8a3e` |
