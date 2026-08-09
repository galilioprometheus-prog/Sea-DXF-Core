# M14.4al HATCH Boundary Spline Edge Header Numerics

M14.4al selects the five M14.4ak fixed-header cards into one source-stable
numeric entry per boundary edge typed as Spline. Unique Degree group 94,
KnotCount group 95, and ControlPointCount group 96 decode in their exact signed
Int32 wire domain. Unique Rational group 73 and Periodic group 74 decode in
their exact signed Int16 wire domain.

Autodesk assigns those fields in the
[Spline edge data table](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-DC5215D6-E73F-4DFF-8BE9-01CA9610FAEE.htm),
and defines groups 70-78 as integer values and groups 90-99 as 32-bit integer
values in the
[group-code reference](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3F0380A5-1C15-464D-BC66-2C5F094BCFB9.htm).

Every Explicit value retains exact `entity.hatch` field and raw provenance.
Absent cards remain Absent without defaults; Multiple cards retain their count
without selecting a winner. Malformed ASCII values remain InvalidAsciiNumber
with raw provenance. Every signed wire value remains exact, including negative
degree/count values and rational/periodic values outside a Boolean domain;
requiredness and domain enforcement are deliberately deferred.

Declared/observed edge-count mismatch does not erase otherwise available
entries. Line, CircularArc, EllipticArc, malformed or unsupported edge types,
empty paths, and Polyline paths publish no Spline numeric entries. Group 97 is
still unclassified, and repeated knot, control-point, weight, fit-point, and
tangent payloads remain raw and unpartitioned.

The focused suite passes 4/4 tests, the Spline header card-to-numeric chain
passes 8/8, and workspace passes 1,241/1,241 tests. All required dependency,
format, schema, release-evidence, Clippy, exact test-count,
forbidden-production-pattern, protected-surface, link, and whitespace gates
pass. Production adds 378 lines: a 371-line module and seven module/export
lines. Tests add 335 lines. No dependency, license, schema, release, corpus,
CI, fixture, or protected-surface change occurs. Requiredness,
degree/count/flag domains, group-97 classification, repeated-sequence
partitioning, topology, OCS/WCS geometry, applicability, CRUD/write, rendering,
and `Complete` remain open.

The repository batch note and cumulative external `Status: READY` packet are
synchronized through M14.4al at
`D:\SeaCad\AntigravityReports\seacad-m14.3dr-m14.4al-hatch-boundary-spline-edge-header-numerics-2026-08-10.yaml`.
No independent Antigravity PASS is claimed. This audit omits its own hash and
the self-referential active batch-note hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 288 | `2581e276319340d89463dd16b4d710322abb859013fc36a92d62ae234758e644` |
| `README.vi.md` | 288 | `4aecaabd9354e1499c1501d1604fa8ef9df02fa7df150e56e48cf78f16288dab` |
| `crates/seacad-dxf-core/src/hatch_boundary_spline_edge_header_numeric.rs` | 371 | `f47d87ff8dfb2003866eba5c88da5263de0909bcd5aaa6172e9bec80c84b348e` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,515 | `9fad05122e488fbea8b2c775b8a980adb381b5253877ae8df83ed0d7600ef2b2` |
| `crates/seacad-dxf-core/tests/hatch_boundary_spline_edge_header_numeric_tests.rs` | 335 | `dd58c08dd569e3cb344eb557494149d4f2087613702d779347284955bb44a181` |
| `docs/IMPLEMENTATION_PLAN.md` | 424 | `0909f637edb40a2b09d649ee55217418fbd2a18b3aba8910da2f32589a0cdf11` |
| `docs/SUPPORT_MATRIX.md` | 3,439 | `a25acb8b26704e2d7156e73c9df7236dd0e1b1b0943e434490848d66295d2f2a` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,362 | `72a3ecec71eb7a662675cea8465366df8a2ce42da596482d3e598c8fa18b5f25` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,789 | `8aa6f4414f677f339125ac258beab92ed799f8160e0595571dace129b885b5d2` |
