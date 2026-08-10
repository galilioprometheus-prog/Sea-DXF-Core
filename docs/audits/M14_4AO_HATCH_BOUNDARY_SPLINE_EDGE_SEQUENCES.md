# M14.4ao HATCH Boundary Spline Edge Sequences

M14.4ao partitions the exact M14.4an `edge_data` slice for every boundary edge
typed as Spline into ordered grammar phases. It does not decode a numeric value,
assemble a point tuple, compare a declared count, apply a weight default, or
derive geometry.

Autodesk's
[Boundary Path Data table](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-DC5215D6-E73F-4DFF-8BE9-01CA9610FAEE.htm)
orders the fixed groups 94/73/74/95/96 before repeated knot group 40,
control-point groups 10/20 with optional weight group 42, optional fit-data
count group 97, repeated fit-point groups 11/21, and optional start/end tangent
groups 12/22 and 13/23. M14.4an has already removed the later path-level group
97 and group-330 references, so this checkpoint classifies only Spline edge
data.

Each available sequence publishes exact source ranges for its header, knot,
control-point/weight, fit-point, start-tangent, and end-tangent phases plus the
exact optional fit-data-count field. Empty phases remain explicit zero-length
ranges at their grammar boundary. Header duplicates and incomplete point or
tangent components remain retained raw evidence for later cardinality/tuple
checkpoints rather than being selected or rejected here.

Fit-point or tangent fields before a fit-data count become
`FitDataCountAbsent`; a second edge-level group 97 becomes
`DuplicateFitDataCount`; a field that returns to an earlier grammar phase
becomes `FieldOutOfOrder`; and every unrecognized edge-data group becomes
`UnexpectedField`. An unavailable M14.4an payload partition remains the exact
`EdgePayloadUnavailable` issue. All failures retain the complete lower
partition, edge-type, and edge-count evidence.

The focused suite passes 4/4 tests and the edge grouping-to-sequence chain
passes 16/16. Workspace passes 1,253/1,253 tests and all required dependency,
format, schema, release-evidence, Clippy, exact test-count, forbidden-production-
pattern, protected-surface, link, and whitespace gates pass. Production adds
454 lines and deletes one: a 447-line module, six module/export lines, and one
crate-private shared slice helper visibility change. Tests add 413 lines. No
dependency, license, schema, release, corpus, CI, fixture, or protected-surface
change occurs. Point tuple/cardinality construction, count decoding and count
relations, weight defaults, numeric values, topology beyond phase order,
geometry, applicability, CRUD/write, rendering, and `Complete` remain open.

The repository batch note and cumulative external `Status: READY` packet are
synchronized through M14.4ao at
`D:\SeaCad\AntigravityReports\seacad-m14.3dr-m14.4ao-hatch-boundary-spline-edge-sequences-2026-08-10.yaml`.
No independent Antigravity PASS is claimed. This audit omits its own hash and
the self-referential active batch-note hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 298 | `444b971562352d79d1bd833689ee892f33287b31f6709ef69ee7de81dfd0d5f9` |
| `README.vi.md` | 297 | `07e7fce2e2ca1614f6e2908ea12bb67f157df437b201072fb3c6d4b0f0fd2885` |
| `crates/seacad-dxf-core/src/hatch_boundary_edge_payload_partition.rs` | 300 | `ea29fd20453d30d1381a4260509bf4cec068630168732c2e6cdc9fefea711029` |
| `crates/seacad-dxf-core/src/hatch_boundary_spline_edge_sequence.rs` | 447 | `71ec1ac816a1d962a7c2c372b70f0555fc93848c81b61ca9b32c2a86031b1a5a` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,537 | `60f0be1e04ba745451f9aa2879d6673de2d57f15e4eac3656c214ffef01bdb86` |
| `crates/seacad-dxf-core/tests/hatch_boundary_spline_edge_sequence_tests.rs` | 413 | `206d469e5ea8dc2d4bd16a42101b654d9fb31de7d23332dc2e628637ec71619f` |
| `docs/IMPLEMENTATION_PLAN.md` | 424 | `17393c66af67f6a0ea5633dd2e7078f773de5e646ccfe081d6f3d69637de2795` |
| `docs/SUPPORT_MATRIX.md` | 3,478 | `7bee4a1cb17df0c081271d1e18465b42ad295a850974a03c1340675cdd737346` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,400 | `e74ce960778669e245d066da4a6ddbf552442d5bfdc7b40fbf77a3fdcc21e93b` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,830 | `9e49c9cb36fd07881c58ad0c12278504b036f780ea657d5c1f53e81ba9c1ffea` |
