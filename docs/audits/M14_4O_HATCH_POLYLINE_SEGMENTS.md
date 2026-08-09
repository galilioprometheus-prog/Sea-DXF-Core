# M14.4o HATCH Polyline Segments

M14.4o builds fail-closed open/closed segment topology for HATCH Polyline paths.
Matched group-93 counts admit adjacent segments and optional last-to-first
closure; mismatches, Edges, and invalid headers publish no segments. Endpoint
ordinals resolve exact M14.4n coordinate entries even when coordinates are
unusable. No line or bulge-arc geometry is inferred.

Autodesk documents group 73 closure and group 93 vertex count:
`https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-DC5215D6-E73F-4DFF-8BE9-01CA9610FAEE.htm`

The focused suite passes 4/4 tests and workspace passes 1,149/1,149 tests. All
gates pass. No dependency/license/schema/release/corpus/CI change. Production
adds 420 lines: 413 module and seven exports. Segment shape, bulge arc
construction, OCS/WCS transformation, applicability, CRUD/write, and `Complete`
remain open. This audit omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 225 | `b0e3829291f7c2dc375cf970dae20fabc0aa6d9446234c079c924cf64a1ecc57` |
| `README.vi.md` | 223 | `5b15ec5265985f81d05ef5235d22346098b6c37f13bbf77a62c67ed8a1058dda` |
| `crates/seacad-dxf-core/src/hatch_polyline_segment.rs` | 413 | `5d54ec4f661282fb0c0f3fc6c1a73beb2e18ccd4df0397fc7e2acbd4d841cc22` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,358 | `2c3c9857bedca7b9e72d7c15ef04f7983ce11171cc3fb6ebb8f80821473fcd48` |
| `crates/seacad-dxf-core/tests/hatch_polyline_segment_tests.rs` | 311 | `e64d78976c0dd1707370711431e4e62c54e479ca9e1488bea4a1189f742aa253` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `84c7dbc195749177d67c38de83de1857a85de64c81ac4232e3a1067ffc1a77e6` |
| `docs/SUPPORT_MATRIX.md` | 3,159 | `611ab3caab6fec61755c6f9fa10dbfaa42d8b441e08307a52aecb6be56b2933f` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,072 | `9cb008ceace45e06352accb13125d2085e207cd244f75d4c0126bc112d83846e` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,490 | `aed6bf4fd23be46a913779e2ea4cbb51f9166b30de3b00da48565d932f067eb6` |
