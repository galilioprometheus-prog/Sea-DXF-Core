# DXF-STRUCT-R1b HATCH Geometry Resolvers

DXF-STRUCT-R1b removes ownership-graph knowledge from both HATCH WCS
constructors. Boundary Line-edge WCS previously traversed coordinate, numeric,
card, edge-type, and edge directories. Polyline WCS previously traversed line,
shape, and segment directories. Each constructor now asks its immediate source-
geometry directory for the subclass owning one local entry ordinal.

The two crate-private resolvers first select an entry from their own directory,
then derive its subclass from the same retained exact path evidence used before
this refactor. They store no copied owner identifier and add no field. Public
APIs, type layout, raw bytes, provenance, typed issue precedence, debug
redaction, cancellation, support claims, and dependencies remain unchanged.

The two focused WCS suites pass 8/8 tests and workspace passes 1,193/1,193
tests. All gates pass. Production changes four files, adds 28 lines, deletes 17,
and therefore adds 11 net lines while deleting two upper-layer dependency paths.
No fixture, schema, release, corpus, CI, or support-matrix change occurs.

The repository batch note and cumulative external `Status: READY` packet are
synchronized through DXF-STRUCT-R1b at
`D:\SeaCad\AntigravityReports\seacad-m14.3dr-dxf-struct-r1b-hatch-geometry-resolvers-2026-08-09.yaml`.
No independent Antigravity PASS is claimed. This audit omits its own hash and
the self-referential active batch-note hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/hatch_boundary_line_edge_geometry.rs` | 202 | `e8f4a0a495d02a44a0097a101d459fa277c4ae18ab8c4b363b593b3057a65793` |
| `crates/seacad-dxf-core/src/hatch_boundary_line_edge_wcs_geometry.rs` | 267 | `305bc6eaa1d7bc2e478d1003bedb3fc85ffffd5d8dfd7769b7394707db489400` |
| `crates/seacad-dxf-core/src/hatch_polyline_segment_geometry.rs` | 343 | `07fe2d014d3bcbab1cd2eb94a1e8b4467552b701809b58ef6183f63078dce76a` |
| `crates/seacad-dxf-core/src/hatch_polyline_wcs_geometry.rs` | 442 | `0570fba5a2a0229a9c95f866aa5e56ce2c7b33e8024c9bcb60d113ed828a323c` |
| `docs/ARCHITECTURE.md` | 204 | `4fdcda1179cb34a1dc9b3f188a553899d0ccd114e558dbce2ad330e3db5ec570` |
| `docs/IMPLEMENTATION_PLAN.md` | 424 | `0341807fb22f9fa331874c188d6fc5c562dc7def9c9a9e8388ca8f0b6c8a17bf` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,204 | `d0c00b2f32353da529290ac8385aeb83a211a19ae02a69072fdb6b212db21fd2` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,625 | `a6aa4473c0cd7d36a549224967bce25225e81fda84ec863a3c767ff1ef49217b` |
