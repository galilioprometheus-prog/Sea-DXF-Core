# M14.4ag HATCH Boundary EllipticArc Edge Numerics

M14.4ag selects the eight M14.4af cardinality cards into one source-stable
numeric entry for every boundary edge typed as EllipticArc. Unique finite OCS
CenterX group 10, CenterY group 20, relative major-axis EndpointX group 11 and
EndpointY group 21, MinorToMajorRatio group 40, StartAngle group 50, and
EndAngle group 51 become explicit binary64 values. Unique Counterclockwise
group 73 remains an exact signed Int16 without Boolean-domain enforcement.

Autodesk specifies those eight fields in the
[Ellipse edge data table](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-DC5215D6-E73F-4DFF-8BE9-01CA9610FAEE.htm).

Every explicit value retains exact `entity.hatch` field provenance and raw
group occurrence/span provenance. Absent cards remain Absent. Multiple cards
retain their occurrence count without selecting a winner. Malformed ASCII
numbers and non-finite Binary doubles remain typed issues; uniquely malformed
values retain their raw provenance. Declared/observed edge-count mismatch does
not erase otherwise available entries. Line, CircularArc, Spline,
malformed/unsupported edge types, empty edge paths, and Polyline paths publish
no EllipticArc numeric entries while retaining the complete card/type/grouping
chain.

Every lookup is bounded, cancellation is cooperative, source identity is
exact, signed zero remains bit-exact, and Debug output does not expose raw
values. The focused numeric suite passes 4/4 and the EllipticArc card-to-numeric
chain passes 8/8. Workspace passes 1,221/1,221 tests and all gates pass.
Production adds 438 lines: the 430-line numeric module and eight module/export
lines. Tests add 421 lines. No dependency, license, schema, release, corpus,
CI, fixture, or protected-surface change occurs. Requiredness, major-axis
vector and ratio domains, angle normalization, direction domain, OCS/WCS
geometry, Spline edge payloads, HATCH applicability, CRUD/write, rendering,
and `Complete` remain open.

The repository batch note and cumulative external `Status: READY` packet are
synchronized through M14.4ag at
`D:\SeaCad\AntigravityReports\seacad-m14.3dr-m14.4ag-hatch-boundary-elliptic-arc-edge-numerics-2026-08-10.yaml`.
No independent Antigravity PASS is claimed. This audit omits its own hash and
the self-referential active batch-note hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 273 | `f1e88ccfb2caf67c106a42fb42e125fc426edcc26b1576d378ee29bd09a046c8` |
| `README.vi.md` | 273 | `023454d576c71b55c8812cb3917572215d77a8b38ea2421e73479cd441da1965` |
| `crates/seacad-dxf-core/src/hatch_boundary_elliptic_arc_edge_numeric.rs` | 430 | `03d5aa5ad4601a262379cdd5517f6d087203c5f87984eb7dd7b9216de4b71319` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,479 | `2d8ab2b504e35befd9e71cca66e591c93290f85347f5467b3a70d93f6d22760d` |
| `crates/seacad-dxf-core/tests/hatch_boundary_elliptic_arc_edge_numeric_tests.rs` | 421 | `f5eff00beb286bc17006fe9a58af00ec29c8dc6fa8dbfd6c078c37aad69bb8ae` |
| `docs/IMPLEMENTATION_PLAN.md` | 424 | `7477e37c92808607ca04151c26cd41308ae734309a29ad27b34e12aaa30261d6` |
| `docs/SUPPORT_MATRIX.md` | 3,372 | `8775414f63bc2b5b33cf3128921021adc88575041af72b3ddbb5a6c7528d3b42` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,297 | `3d731b0a5ad35662a583b08c1410bd6abdff15f1b68eb30d4af2f50630680706` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,722 | `63f258297779a4b0da829ab7cb3f13412da6ba6d3719c134b552afd0aa10a21d` |
