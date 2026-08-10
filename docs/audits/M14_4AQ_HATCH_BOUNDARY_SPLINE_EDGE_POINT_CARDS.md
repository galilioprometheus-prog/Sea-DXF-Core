# M14.4aq HATCH Boundary Spline Edge Point Cards

M14.4aq publishes fixed component-card evidence over every available M14.4ap
HATCH boundary Spline point tuple. It does not decode a numeric value, select
a required component, apply the documented weight default, compare a declared
count, classify knots or tangents, or derive topology or geometry.

Autodesk's
[Boundary Path Data table](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-DC5215D6-E73F-4DFF-8BE9-01CA9610FAEE.htm)
defines control points as group-10/20 pairs with optional group-42 weights and
fit points as group-11/21 pairs. M14.4ap already retains each X anchor, every
following same-tuple member, and exact Y/weight counts. This checkpoint adds
the independent fixed-card contract needed before value selection.

Every control-point tuple receives exactly two ordered cards: Y then Weight.
Every fit-point tuple receives exactly one Y card and no Weight card. Each card
retains every matching lower tuple member in source order and reports Absent,
Unique, or Multiple with the exact occurrence count. No duplicate winner is
chosen, and the X anchor remains reachable through the retained tuple.

An available Spline sequence with no tuples publishes an explicit empty card
slice. An unavailable M14.4ap sequence publishes no cards while its typed
lower-layer issue remains reachable through the retained point-tuple directory.
Line, CircularArc, EllipticArc, malformed, empty, and Polyline edge states
publish no point-card entry. Declared/observed edge-count mismatch does not
erase otherwise available cards.

The test-first compile failed only because the new public API did not yet
exist. The focused suite passes 4/4 tests and the edge grouping-to-point-card
chain passes 24/24. Workspace passes 1,261/1,261 tests and all required
dependency, format, schema, release-evidence, Clippy, exact test-count,
forbidden-production-pattern, protected-surface, link, and whitespace gates
pass.

Production adds 267 lines: a 262-line module and five module/export lines.
Tests add 450 lines. No dependency, license, schema, release, corpus, CI,
fixture, or protected-surface change occurs. Numeric point/weight semantics,
required-component selection, default weight 1, knot/header/fit-count
relations, tangent cardinality, topology, OCS/WCS geometry, applicability,
CRUD/write, rendering, and `Complete` remain open.

The repository batch note and cumulative external `Status: READY` packet are
synchronized through M14.4aq at
`D:\SeaCad\AntigravityReports\seacad-m14.3dr-m14.4aq-hatch-boundary-spline-edge-point-cards-2026-08-10.yaml`.
No independent Antigravity PASS is claimed. This audit omits its own hash and
the self-referential active batch-note hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 305 | `3ad68634c32902ecc1970ce7c4fcb6cbf6d060b96c4ecbf4326a5e73e3944850` |
| `README.vi.md` | 304 | `31cbd2274b78e8a2f4078408c4f845a290864764673fc6ffb20e8bd101a74484` |
| `crates/seacad-dxf-core/src/hatch_boundary_spline_edge_point_card.rs` | 262 | `db0e0b15d8a861d962fb83fdfe586033fce328344883b7fca6e84caf5d328633` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,550 | `f644c848d2110454698cef63d163aa4493e8bbd3b8ca9cdf45ff6ddf129b268b` |
| `crates/seacad-dxf-core/tests/hatch_boundary_spline_edge_point_card_tests.rs` | 450 | `57af3852779a21fab604dff20c0ea6289926d76b066fd00d5f9c7c7afdf596de` |
| `docs/IMPLEMENTATION_PLAN.md` | 424 | `cb3ce5768513f562f1b4a402b70936627f0bb462416feb64c64d8427c2861a62` |
| `docs/SUPPORT_MATRIX.md` | 3,502 | `2e24e87c2c692e7f8ac35955e0b889c45a905916e16b3c78701e43d96d41a5c9` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,425 | `ffa71b52ea3cc816f0dbb9ec43836b76177d9d667806139c8d0e64fe0accaa6b` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,855 | `fbe7c8aa4296b426afbe3ebf8f052bed94a53c05553f29bddef3255223d13c85` |
