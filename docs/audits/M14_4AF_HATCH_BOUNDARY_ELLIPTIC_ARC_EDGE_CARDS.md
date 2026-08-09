# M14.4af HATCH Boundary EllipticArc Edge Cards

M14.4af publishes eight ordered cardinality cards for every M14.4u boundary
edge typed as EllipticArc: OCS CenterX group 10, CenterY group 20, relative
major-axis EndpointX group 11 and EndpointY group 21, MinorToMajorRatio group
40, StartAngle group 50, EndAngle group 51, and Counterclockwise group 73. Each
card retains every exact M14.4t payload field and reports Absent, Unique, or
Multiple independently; duplicates remain ordered evidence and no value is
selected.

Autodesk specifies those eight fields in the
[Ellipse edge data table](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-DC5215D6-E73F-4DFF-8BE9-01CA9610FAEE.htm).

Declared/observed edge-count mismatch does not erase otherwise available
cards. Line, CircularArc, Spline, malformed/unsupported edge types, empty edge
paths, and Polyline paths publish no EllipticArc cards while retaining the full
shared card/type/grouping chain. Every lookup is bounded, cancellation is
cooperative, source identity is exact, family role typing remains distinct, and
Debug output does not expose raw values.

The focused suite passes 4/4 tests and all three Line/CircularArc/EllipticArc
card regressions pass 12/12. Workspace passes 1,217/1,217 tests and all gates
pass. Production adds 97 lines: a 90-line family module and seven module/export
lines. Tests add 341 lines. No dependency, license, schema, release, corpus, CI,
fixture, or protected-surface change occurs. Numeric selection, requiredness,
vector/ratio/angle/direction semantics, OCS/WCS geometry, Spline edge payloads,
HATCH applicability, CRUD/write, rendering, and `Complete` remain open.

The repository batch note and cumulative external `Status: READY` packet are
synchronized through M14.4af at
`D:\SeaCad\AntigravityReports\seacad-m14.3dr-m14.4af-hatch-boundary-elliptic-arc-edge-cards-2026-08-10.yaml`.
No independent Antigravity PASS is claimed. This audit omits its own hash and
the self-referential active batch-note hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 270 | `c8756e5325028f990de6f90f90437c609a5f4d3ecdcbb6418a21c83cae20a0dc` |
| `README.vi.md` | 270 | `3f27722bf8d4bc2e8c29f8f1101f41948170dde6a6cfecb41eae44c5630842c2` |
| `crates/seacad-dxf-core/src/hatch_boundary_elliptic_arc_edge_card.rs` | 90 | `e70ccc6a358e6876f381510adf04049e3fb661260d5d05bafc9b6bddd197bae0` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,471 | `bbd4012829e4ea099d29d10c0aafecec71f32014f7dd177384c26f599830c022` |
| `crates/seacad-dxf-core/tests/hatch_boundary_elliptic_arc_edge_card_tests.rs` | 341 | `2d745c3cd12b1a76251b97d67a1cf745e0766cae13841b5fb85a24da52463cd4` |
| `docs/IMPLEMENTATION_PLAN.md` | 424 | `c62d1e0d1e50897be1fc48ed00d84f5acab7c535678045cc3a1f56dca80f0c86` |
| `docs/SUPPORT_MATRIX.md` | 3,359 | `4ad5c68b52b972b8b2703cc40cd98fd6a447da07ac6820409d6fbc5eae8c79be` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,285 | `820b2f615f3280739c095b54c1db9df381bdd36dd9d6e8ec7af13ada1c5bfcc3` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,710 | `997f1a6d41ec7547aa94ec314d6f38a157d705b1de98b98ab5f7040a1c2d5307` |
