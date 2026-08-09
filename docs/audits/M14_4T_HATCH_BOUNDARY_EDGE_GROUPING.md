# M14.4t HATCH Boundary Edge Grouping

M14.4t selects one required non-negative group-93 edge count for every HATCH
boundary path classified as Edges and groups each subsequent group-72 field as
an exact source-anchored edge marker. Every marker retains a conservative raw
slice through the next marker or path end. Declared and observed counts remain
explicitly Matched or Mismatched, and zero-edge paths are valid. Invalid path
flags, count cardinality/value, or marker ordering publish no edge entries.

Autodesk documents the count, edge marker, four edge kinds, and path trailer in
the [HATCH group-code table](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-C6C71CED-CE0F-4184-82A5-07AD6241F15B.htm).

The focused suite passes 4/4 tests and workspace passes 1,169/1,169 tests. All
gates pass. No dependency, license, schema, release, corpus, or CI surface
changes. Production adds 507 lines: a 501-line module and six module/export
lines. Edge-type values and line/circular/elliptic/spline payload semantics are
not decoded. HATCH entity applicability remains `NotYetReviewed` because the
reviewed Autodesk sources do not directly prove a version floor. Geometry,
CRUD/write, rendering, and `Complete` remain open. This audit omits its own
hash.

One cumulative Antigravity review packet is prepared as `Status: READY` through
M14.4t at
`D:\SeaCad\AntigravityReports\seacad-m14.3dr-m14.4t-hatch-boundary-edge-grouping-2026-08-09.yaml`.
It supersedes the M14.4s packet; no independent Antigravity PASS is claimed at
this checkpoint.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 235 | `2ded960ad6606f0f21a3eaab781ae468cc037aac5c2ff25cf9f9aa2def3fa0cf` |
| `README.vi.md` | 234 | `2f55719570d2c08c410ecf8853c80967649d0c227e7f1705d4b79a8448e4c807` |
| `crates/seacad-dxf-core/src/hatch_boundary_edge.rs` | 501 | `4ed1194477ca74b9e9e7db0a225eb88ce6618a41ecc978c2593471ee30f8155e` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,386 | `294dd75e1947132850b0728814e31e01e1e81a4c34744ca98b9122aa1f98c9ea` |
| `crates/seacad-dxf-core/tests/hatch_boundary_edge_tests.rs` | 382 | `37bb0dcdc6d22822abf8bc6f6f1daa02546781f8dc824b002ddaa98e034920d6` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `000a177102e60f285b532ced4d4aec547bd1f16cf46401755e03c994e41a8779` |
| `docs/SUPPORT_MATRIX.md` | 3,214 | `69f307f408ef507f464ce84ccf1bd502e5b837d021e2af08df71e6a3d56679de` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,123 | `544b033056ed2eddff4ffeb6a53edf277e739b98fe8694c38bc0e165fc025ea4` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,541 | `630a69a182a8973fe17b9dd7f30b67e8d7d453385bb4aa1dc3fc9058b176ffba` |
