# M14.4am HATCH Boundary Spline Edge Header Semantics

M14.4am promotes the five M14.4al source-stable numeric header values for each
boundary edge typed as Spline to required semantics. Degree group 94,
Rational group 73, Periodic group 74, KnotCount group 95, and
ControlPointCount group 96 must all be available; absence becomes
`MissingRequiredValue` without invented raw provenance.

Autodesk assigns all five fields in the
[HATCH Spline edge data table](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-DC5215D6-E73F-4DFF-8BE9-01CA9610FAEE.htm).
That table defines rational and periodic as 0/1 flags. The
[group-code reference](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3F0380A5-1C15-464D-BC66-2C5F094BCFB9.htm)
defines groups 70-78 as signed integer values and groups 90-99 as signed
32-bit integer values.

Rational 0 maps to NonRational and 1 maps to Rational. Periodic 0 maps to
NonPeriodic and 1 maps to Periodic. Every other signed Int16 remains a typed
out-of-domain failure with exact raw provenance. Degree and both count values
retain their exact signed Int32 wire values. No positive, non-negative, or
degree-range constraint is projected from another entity contract because the
reviewed HATCH source does not state one.

Duplicate and malformed numeric values retain the exact M14.4al issue and
available provenance. Declared/observed edge-count mismatch does not erase
otherwise available semantics. Line, CircularArc, EllipticArc, malformed or
unsupported edge types, empty paths, and Polyline paths publish no Spline
header semantic entries. Group 97 remains unclassified, and repeated knot,
control-point, weight, fit-point, and tangent payloads remain raw and
unpartitioned.

The focused suite passes 4/4 tests and the Spline header card-to-semantic chain
passes 12/12. Workspace passes 1,245/1,245 tests and all required dependency,
format, schema, release-evidence, Clippy, exact test-count, forbidden-
production-pattern, protected-surface, link, and whitespace gates pass.
Production adds 391 lines: a 380-line module and eleven module/export lines.
Tests add 359 lines. No dependency, license, schema, release, corpus, CI,
fixture, or protected-surface change occurs. Group-97 classification, repeated-
sequence partitioning, topology, OCS/WCS geometry, applicability, CRUD/write,
rendering, and `Complete` remain open.

The repository batch note and cumulative external `Status: READY` packet are
synchronized through M14.4am at
`D:\SeaCad\AntigravityReports\seacad-m14.3dr-m14.4am-hatch-boundary-spline-edge-header-semantics-2026-08-10.yaml`.
No independent Antigravity PASS is claimed. This audit omits its own hash and
the self-referential active batch-note hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 291 | `9ee0c6fd4727848de8d35ceb9f49d33e85885823f2c952c89772305ed0177112` |
| `README.vi.md` | 291 | `21a11370f0eea8af10b0f2bcd904276f8be2951443bb217ee0b1c1c07988a049` |
| `crates/seacad-dxf-core/src/hatch_boundary_spline_edge_header_semantic.rs` | 380 | `0e589551578ef23b0cdce1bbec7ef8b268a4f1ad0dac607c445180730e4e60a0` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,526 | `f013caef58f77df5ec159816ac69a04bb8063bd2daad348a4f56459cbea8b654` |
| `crates/seacad-dxf-core/tests/hatch_boundary_spline_edge_header_semantic_tests.rs` | 359 | `e5b83c71f225e748ceba841a719d366135a5a0cef3815bb4ad5d303339170d29` |
| `docs/IMPLEMENTATION_PLAN.md` | 424 | `71fd95e832737f9d4bcf1b1dd3f81359a80892e9647d3404dac7440ed82c1c2a` |
| `docs/SUPPORT_MATRIX.md` | 3,452 | `e71ab7bfd2ea7120ae3a86e8e1fd43cd00decfbc3a4704b88d3dae96978b1a72` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,375 | `f3aa20191531695451593fead0dbbdfe32bce2fb10ff16c4a5d910323fd1aaea` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,803 | `0c88483530fde42b73a922a37f2af17264c4f940aff8b848006c480e92f6923e` |
