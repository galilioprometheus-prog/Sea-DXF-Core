# M14.4ac HATCH Boundary CircularArc Edge Semantics

M14.4ac promotes every M14.4ab CircularArc numeric component to required,
domain-checked semantics. Groups 10/20 provide the exact OCS center, group 40
provides the radius, groups 50/51 remain exact start/end angles in DXF degrees,
and group 73 maps only 0 to Clockwise or 1 to Counterclockwise.

Autodesk defines the six fields in the
[Arc edge data table](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-DC5215D6-E73F-4DFF-8BE9-01CA9610FAEE.htm),
defines group codes 50 through 58 as angles in degrees in the
[group-code reference](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3F0380A5-1C15-464D-BC66-2C5F094BCFB9.htm),
and requires an arc radius greater than zero while accepting equivalent
negative angles in the
[AcDbArc method contract](https://help.autodesk.com/cloudhelp/2018/ENU/OARX-RefGuide/files/OREF-__MEMBERTYPE_Methods_AcDbArc.html).
SeaCad therefore rejects positive zero, negative zero, and every negative
radius while preserving their exact bits, and does not normalize, wrap, or
derive a sweep from either angle.

Absent fields become MissingRequiredValue without invented raw provenance.
Multiple, malformed ASCII, and non-finite Binary values retain their exact
M14.4ab typed issue and available raw provenance. Every other signed Int16
direction remains DirectionFlagOutOfDomain with its exact raw value. Edge-count
mismatch does not erase usable entries; non-CircularArc, invalid, empty, and
Polyline states publish none while retaining the complete numeric directory.

The focused suite passes 4/4 tests and workspace passes 1,205/1,205 tests. All
required dependency, format, schema, release-evidence, Clippy, test-count, and
whitespace gates pass. Production adds 350 lines: a 341-line module and nine
module/export lines. Tests add 522 lines. No dependency, license, schema,
release, corpus, CI, fixture, or protected-surface change occurs. OCS/WCS
CircularArc geometry, the other edge payload families, HATCH applicability,
CRUD/write, rendering, and `Complete` remain open.

The repository batch note and cumulative external `Status: READY` packet are
synchronized through M14.4ac at
`D:\SeaCad\AntigravityReports\seacad-m14.3dr-m14.4ac-hatch-boundary-circular-arc-edge-semantics-2026-08-10.yaml`.
No independent Antigravity PASS is claimed. This audit omits its own hash and
the self-referential active batch-note hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 261 | `6493ac09938d6f6f445efcc4ed756a5cdf9b8a81483533e89fb3ec0d3f817060` |
| `README.vi.md` | 261 | `b5079debe4abcef8bc746c746e1749e4d26905bc1b44bc83d6c706e25aa8e6ef` |
| `crates/seacad-dxf-core/src/hatch_boundary_circular_arc_edge_semantic.rs` | 341 | `470ad786f740459d2a211cde5b4f24ec769c034ff4775c7599f95117acf2964f` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,447 | `5ae8e6d836cddec9d49b4744cb8fa9473cf481d60575c9f986fd65c16cdbeb36` |
| `crates/seacad-dxf-core/tests/hatch_boundary_circular_arc_edge_semantic_tests.rs` | 522 | `7d8d9513972a599ab68c4e5ebf005cf9b32f91b3436da6246419320d3493c0a8` |
| `docs/IMPLEMENTATION_PLAN.md` | 424 | `2787702969debc9868bb7b85f62adf422531ef247ff85345c70357a83d9a8e26` |
| `docs/SUPPORT_MATRIX.md` | 3,322 | `f7baebdc819f98a1e59714c0b794a8807732cc38c78db7f5f5ee5fd4c74ece6c` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,240 | `797afb1815cea2e873b3bc7fd938d05c7f0d8edabc73c3cafbe09734b220beb7` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,663 | `7a715283d04bae9fcee17d13ba941d92ebcf59b6369982a83f0374b2252c80dc` |
