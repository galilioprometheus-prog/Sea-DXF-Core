# M14.4ad HATCH Boundary CircularArc Edge OCS Geometry

M14.4ad publishes one exact-value OCS CircularArc segment for every M14.4ac
entry whose six required semantics are usable. The segment retains the exact
OCS center, strictly positive radius, start/end angles in DXF degrees, and
reviewed Clockwise or Counterclockwise direction.

Autodesk defines these values in the
[Arc edge data table](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-DC5215D6-E73F-4DFF-8BE9-01CA9610FAEE.htm)
and defines group codes 50 through 58 as angles in degrees in the
[group-code reference](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3F0380A5-1C15-464D-BC66-2C5F094BCFB9.htm).
This checkpoint performs no arithmetic: negative, large, equal, and signed-zero
values pass through bit-exactly. Equal start/end angles do not invent either a
zero sweep or a full turn, and no endpoint, wrapping, or trigonometric value is
derived.

If any semantic value is unavailable, geometry returns an exact six-component
mask while the complete M14.4ac semantic and M14.4ab numeric issue/provenance
chain remains resolvable. Count mismatch does not erase usable geometry;
non-CircularArc, invalid, empty, and Polyline states publish none.

The focused suite passes 4/4 tests and workspace passes 1,209/1,209 tests. All
required dependency, format, schema, release-evidence, Clippy, test-count, and
whitespace gates pass. Production adds 315 lines: a 309-line module and six
module/export lines. Tests add 390 lines. No dependency, license, schema,
release, corpus, CI, fixture, or protected-surface change occurs. Angle
wrapping, signed sweep, endpoint derivation, OCS-to-WCS projection, the other
edge payload families, HATCH applicability, CRUD/write, rendering, and
`Complete` remain open.

The repository batch note and cumulative external `Status: READY` packet are
synchronized through M14.4ad at
`D:\SeaCad\AntigravityReports\seacad-m14.3dr-m14.4ad-hatch-boundary-circular-arc-edge-ocs-geometry-2026-08-10.yaml`.
No independent Antigravity PASS is claimed. This audit omits its own hash and
the self-referential active batch-note hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 264 | `a95cf74fc3436ef08cb864d008a6a48bd36a8d05e9fe1bd40ba76b23c768598d` |
| `README.vi.md` | 264 | `bcc11605e2b74ab51efedccff6c346a7ac93c802945782f4fa29ced1e6a5c8bf` |
| `crates/seacad-dxf-core/src/hatch_boundary_circular_arc_edge_geometry.rs` | 309 | `51cf5416ed9867fb725eb72dfafa406166d58f15cc8ce0e10f1beccb0aca9fe6` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,453 | `6cfe268554cc58597e09a414b5dfbc5b568807e6c2e234c235c4a52de6ddb401` |
| `crates/seacad-dxf-core/tests/hatch_boundary_circular_arc_edge_geometry_tests.rs` | 390 | `eeeafc4f3db6397400ae604d52f0beb389e83db0d77673096ddf1ec95ea66970` |
| `docs/IMPLEMENTATION_PLAN.md` | 424 | `e505b36ba89d15cd3c9aee0a88705db1f0f390a67e754ce30a72e24051d1ddfd` |
| `docs/SUPPORT_MATRIX.md` | 3,334 | `983057b96e46d90be13187c45ce826bf71360529a19f1f4e3dadebf423c07281` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,252 | `ca69e900b2bce2abed9a7d6d596a23a0f4952cf12bb97881a5310dbd4b64aa23` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,676 | `df04c898f02ba3ba4888d5ac27410fbee597017b8642f4c3a27ddce2de139b3d` |
