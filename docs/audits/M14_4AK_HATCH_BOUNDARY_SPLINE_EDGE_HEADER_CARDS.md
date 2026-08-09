# M14.4ak HATCH Boundary Spline Edge Header Cards

M14.4ak publishes five ordered cardinality cards for every M14.4u boundary
edge typed as Spline: Degree group 94, Rational group 73, Periodic group 74,
KnotCount group 95, and ControlPointCount group 96. Each card retains every
matching M14.4t payload field in exact source order and reports Absent, Unique,
or Multiple independently; no value is selected or decoded.

Autodesk assigns those five fixed header fields in the
[Spline edge data table](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-DC5215D6-E73F-4DFF-8BE9-01CA9610FAEE.htm).
Group 97 is deliberately not classified here because the final edge payload
can also retain the path-level source-boundary-object count under that group
code. Knot, control-point, weight, fit-point, and tangent sequences remain raw
until a grammar-aware partition can distinguish their repeated payloads.

Declared/observed edge-count mismatch does not erase otherwise available
cards. Line, CircularArc, EllipticArc, malformed or unsupported edge types,
empty paths, and Polyline paths publish no Spline header cards while retaining
the full lower edge-type/grouping directory. Every lookup is bounded,
cancellation is cooperative, source identity is exact, family role typing
prevents interchange, and Debug output does not disclose values.

The focused suite passes 4/4 tests, all four HATCH edge-card family suites pass
16/16, and workspace passes 1,237/1,237 tests. All required dependency, format,
schema, release-evidence, Clippy, exact test-count, forbidden-production-pattern,
protected-surface, link, and whitespace gates pass. Production adds 90 lines:
an 83-line module and seven module/export lines. Tests add 337 lines. No
dependency, license, schema, release, corpus, CI, fixture, or protected-surface
change occurs. Numeric decoding, requiredness, degree/count/flag domains,
group-97 classification, repeated-sequence partitioning, topology, OCS/WCS
geometry, applicability, CRUD/write, rendering, and `Complete` remain open.

The repository batch note and cumulative external `Status: READY` packet are
synchronized through M14.4ak at
`D:\SeaCad\AntigravityReports\seacad-m14.3dr-m14.4ak-hatch-boundary-spline-edge-header-cards-2026-08-10.yaml`.
No independent Antigravity PASS is claimed. This audit omits its own hash and
the self-referential active batch-note hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 285 | `6a7d3d33eb4a72f58ac6e6e1214cb0852a7a6600309d91b15a2d272a3c269735` |
| `README.vi.md` | 285 | `f786f515ac847b7c0ed4e047fd0840bddffb135973a8a6676ef904d1ea85e9c9` |
| `crates/seacad-dxf-core/src/hatch_boundary_spline_edge_header_card.rs` | 83 | `62c3043001336a9ae58bf447b8c2c03db87f9c8788dda604554c0f1d0a71063b` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,508 | `4977d2572dc812f534874375168df6aeb1bc97ff28316662862bcb2dd8ffc8d3` |
| `crates/seacad-dxf-core/tests/hatch_boundary_spline_edge_header_card_tests.rs` | 337 | `0b63b7d2476b5f3af8d1ca13f3b7f7889a824ab391cc0ca43ac5719e4c50423a` |
| `docs/IMPLEMENTATION_PLAN.md` | 424 | `6a8c230680ca6906cf01eea196443583d0f1e23068b4fc6d588cb80a2e13d3b0` |
| `docs/SUPPORT_MATRIX.md` | 3,425 | `1cedf62c81afcf0df5cee4b18f26c6b88181424b5d1b822431d51e846904877c` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,349 | `ac4ca007b85574e1fcff6b7661216fcd49ccdeacd4c8119a1982d2b74910241f` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,776 | `9cfc1211777fe828008be36a94809f24f30e027215442a6b0bf8e1f9d56a4a99` |
