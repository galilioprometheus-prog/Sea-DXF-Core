# M14.4aa HATCH Boundary CircularArc Edge Cards

M14.4aa publishes six ordered cardinality cards for every M14.4u boundary edge
typed as CircularArc: OCS CenterX group 10, CenterY group 20, Radius group 40,
StartAngle group 50, EndAngle group 51, and Counterclockwise group 73. Each card
retains every exact M14.4t payload field and reports Absent, Unique, or Multiple
independently; duplicates remain ordered evidence and no value is selected.

Autodesk specifies those six fields in the
[Arc edge data table](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-DC5215D6-E73F-4DFF-8BE9-01CA9610FAEE.htm).

Declared/observed edge-count mismatch does not erase otherwise available
cards. Line, EllipticArc, Spline, malformed/unsupported edge types, empty edge
paths, and Polyline paths publish no CircularArc cards while retaining the full
lower edge-type/grouping directory. Every lookup is bounded, cancellation is
cooperative, source identity is exact, and Debug output does not expose values.

The focused suite passes 4/4 tests and the existing Line-card regression passes
4/4. Workspace passes 1,197/1,197 tests and all gates pass. Production adds 311
lines: a 304-line module and seven module/export lines. Tests add 320 lines. No
dependency, license, schema, release, corpus, CI, fixture, or protected-surface
change occurs. Numeric selection, required values, radius/angle/direction
semantics, OCS/WCS geometry, other edge payload families, HATCH applicability,
CRUD/write, rendering, and `Complete` remain open.

The repository batch note and cumulative external `Status: READY` packet are
synchronized through M14.4aa at
`D:\SeaCad\AntigravityReports\seacad-m14.3dr-m14.4aa-hatch-boundary-circular-arc-edge-cards-2026-08-10.yaml`.
No independent Antigravity PASS is claimed. This audit omits its own hash and
the self-referential active batch-note hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 255 | `cedbd9300df36f43ec8e0a68422ecea12c37f205b79df8875754043f39502eda` |
| `README.vi.md` | 255 | `419a367323220543b699fa09d8a695180053834c97d4e5c135e52742f3f8ce58` |
| `crates/seacad-dxf-core/src/hatch_boundary_circular_arc_edge_card.rs` | 304 | `2a6468d37d0001b85567a30b3ddf5a78bff9c822146fd36e255c710369fadb07` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,430 | `cc5d856d1e3e2703b489856227e3ea1e296ab93e4ca980a69be97d0b8b75cc68` |
| `crates/seacad-dxf-core/tests/hatch_boundary_circular_arc_edge_card_tests.rs` | 320 | `7efb5656911a85fd46343792d147fea6deab7a56f75cc7d45ff87db936234bda` |
| `docs/IMPLEMENTATION_PLAN.md` | 424 | `8f3d6ec0b3224872c33b4911b6053eff97a50a6c63292d0fd7ffdefc608e848a` |
| `docs/SUPPORT_MATRIX.md` | 3,295 | `96242de31d7b7cf0689c1905bed954d88fe59f588667cedae6e034f52275f44a` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,215 | `f317d6d6172aa7329693c216614f118819a93a8d2aa133f3cf0d2fa78ce63b82` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,636 | `9b6d29901274f27149e97524a671cc57b842326c0f5ecc7223fd692c009b6d2b` |
