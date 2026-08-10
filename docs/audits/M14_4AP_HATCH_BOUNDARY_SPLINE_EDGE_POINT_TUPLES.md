# M14.4ap HATCH Boundary Spline Edge Point Tuples

M14.4ap groups the exact M14.4ao control-point and fit-point phase fields into
source-order point tuples for every available HATCH boundary Spline edge. It
does not decode a numeric value, select a required component, apply the weight
default, compare a declared count, interpret tangents, or derive geometry.

Autodesk's
[Boundary Path Data table](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-DC5215D6-E73F-4DFF-8BE9-01CA9610FAEE.htm)
defines control points as repeated group-10/20 OCS points with optional group-42
weights and fit data as repeated group-11/21 OCS points. The documented default
weight is 1, but this checkpoint retains absence rather than inventing that
value.

Every group 10 starts one control-point tuple and owns following groups 20/42
until the next group 10. Every group 11 starts one fit-point tuple and owns
following group 21 fields until the next group 11. The tuple retains its exact
X anchor, all members in source order, point index, Y count, and weight count.
Y or weight fields before the first same-kind X anchor remain exact typed
orphans. Duplicate Y/weight members are retained rather than selected.

Each M14.4ao Spline sequence receives one point-grouping entry. Available empty
control or fit phases publish explicit zero tuple/orphan counts. An unavailable
sequence retains its exact `SequenceUnavailable` lower issue, while Line,
CircularArc, EllipticArc, malformed, empty, and Polyline edge states publish no
point entry. Declared/observed edge-count mismatch does not erase available
tuples.

The test-first compile failed only because the new public API did not yet
exist. The focused suite passes 4/4 tests and the edge grouping-to-point-tuple
chain passes 20/20. Workspace passes 1,257/1,257 tests and all required
dependency, format, schema, release-evidence, Clippy, exact test-count,
forbidden-production-pattern, protected-surface, link, and whitespace gates
pass.

Production adds 586 lines: a 578-line module and eight module/export lines.
Tests add 524 lines. No dependency, license, schema, release, corpus, CI,
fixture, or protected-surface change occurs. Numeric point/weight semantics,
required component cardinality, default weight 1, knot/header/fit-count
relations, tangent cards, topology, OCS/WCS geometry, applicability,
CRUD/write, rendering, and `Complete` remain open.

The repository batch note and cumulative external `Status: READY` packet are
synchronized through M14.4ap at
`D:\SeaCad\AntigravityReports\seacad-m14.3dr-m14.4ap-hatch-boundary-spline-edge-point-tuples-2026-08-10.yaml`.
No independent Antigravity PASS is claimed. This audit omits its own hash and
the self-referential active batch-note hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 302 | `10fe610fbab03655079ef904239f12da348861ffffb8da4e51866586efb60048` |
| `README.vi.md` | 301 | `cbd9f4e795c03c4b4bf7cf4c8cd97f5e81baf3cac5511ccebcc96122e7a0a983` |
| `crates/seacad-dxf-core/src/hatch_boundary_spline_edge_point_tuple.rs` | 578 | `1f19a80625a70137d38ef166a7b654758f156bffb18bc0c9f8d70c2a4129e21d` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,545 | `9c7dde2fa36d238c8e33cfd814aa1ca6b2168d326c3fffcfb3cdf70ccc8fb342` |
| `crates/seacad-dxf-core/tests/hatch_boundary_spline_edge_point_tuple_tests.rs` | 524 | `d2f376a8df4b2da462d55c4d34966848551172012e5cfb881ba4d9a2ce78b380` |
| `docs/IMPLEMENTATION_PLAN.md` | 424 | `779f353ee0ff6bd1a21af5524e54f2b704b5c594e8c161ca1e021c7694e7cf58` |
| `docs/SUPPORT_MATRIX.md` | 3,490 | `3745c3bb43d6dec1cb1c255f1cf86acb3473b243cf3caa54a7894f56a40eb95c` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,412 | `1b8907455b511fd94d91e14cc4a8ed99c314a6b01de65bfad1cd688d6e8722a5` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,842 | `80628bb160f3777b2258225425542bde966ad62ff23e418575ec6ec1171a46ba` |
