# M14.4w HATCH Boundary Line Edge Numerics

M14.4w selects the four M14.4v cardinality cards into one source-stable numeric
entry per grouped edge typed as Line. Unique finite doubles become Explicit
semantic values with exact field and raw provenance, including bit-exact signed
zero. Absent cards remain Absent because required-coordinate semantics are a
later checkpoint. Multiple cards retain their occurrence count without a
selected winner. Malformed ASCII and non-finite Binary payloads remain typed
InvalidAsciiNumber and NonFiniteDouble issues; a unique raw value retains its
exact provenance. Count mismatch does not erase usable entries. Non-Line and
invalid typed edges publish no Line numeric entry while the complete M14.4v
card/type/grouping chain remains available.

Autodesk documents the four Line edge fields as OCS start point groups 10/20
and OCS endpoint groups 11/21 in the
[Boundary Path Data table](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-DC5215D6-E73F-4DFF-8BE9-01CA9610FAEE.htm).

The focused suite passes 4/4 tests and workspace passes 1,181/1,181 tests. All
gates pass. No dependency, license, schema,
release, corpus, or CI surface changes. Production adds 347 lines: a 341-line
module and six module/export lines. Required OCS endpoint tuples, OCS/WCS Line
geometry, circular/elliptic/spline edge payloads, HATCH applicability,
CRUD/write, rendering, and `Complete` remain open.

The repository batch note and one cumulative external `Status: READY` packet
are synchronized through M14.4w at
`D:\SeaCad\AntigravityReports\seacad-m14.3dr-m14.4w-hatch-boundary-line-edge-numerics-2026-08-09.yaml`.
No independent Antigravity PASS is claimed. This audit omits its own hash and
the self-referential active batch-note hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 244 | `126971ebbb55e76f03ff723a28eeb1ffc0f1d979d740fa385f5fa21ad29714f4` |
| `README.vi.md` | 243 | `9f041e1d4d1a4f108a356d1725bafd8bd8b703ad3635c22fde376e581e77e076` |
| `crates/seacad-dxf-core/src/hatch_boundary_line_edge_numeric.rs` | 341 | `6f780706baa7721c7ef42f3f23a917660ffa9d5c7f1357d3b75d7ac80df4569a` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,404 | `4539e89ae4887fc5f428849dadc2c4087f9bd7ea9b61827929cddb0cfe60e93e` |
| `crates/seacad-dxf-core/tests/hatch_boundary_line_edge_numeric_tests.rs` | 316 | `3388531ea007f147c620805e09256feec8ab68a4ad6ee5c67126a4805c61c215` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `6a6f860b50bb8748adcc72a9878d4a9f9e148d66585aa532b4c41185e5f30152` |
| `docs/SUPPORT_MATRIX.md` | 3,248 | `76ae4fa8565e71301014f1333e8450b956be9c629eb45f4aaa22a6f8073b9c73` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,155 | `78768c2d2668c9314126d69968e5461af2d80ba25bdfdc53f23659ae8157f73a` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,574 | `97a92ddc87363c12dddc905e6e81123b0c1e739a550ad6fa1cf6b63d87a9ba58` |
