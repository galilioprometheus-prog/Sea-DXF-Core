# M14.4q HATCH Polyline OCS Line Geometry

M14.4q projects exact-endpoint OCS line geometry for each M14.4p Straight
segment. Both vertex positions must be usable. Start and end failures remain
distinct typed issues, Arc retains its exact bulge, and Indeterminate retains
the original bulge issue. Shape classification precedes coordinate
availability, signed-zero endpoint bits remain exact, and zero-length straight
lines remain valid.

Autodesk documents group 42 as the Polyline vertex bulge:
`https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-DC5215D6-E73F-4DFF-8BE9-01CA9610FAEE.htm`

The focused suite passes 4/4 tests and workspace passes 1,157/1,157 tests. All
gates pass. No dependency, license, schema, release, corpus, or CI surface
changes. Production adds 236 lines: a 231-line module and five module/export
lines. Arc center/radius/sweep construction, OCS-to-WCS transformation,
applicability, CRUD/write, and `Complete` remain open. This audit omits its own
hash.

One cumulative Antigravity review packet is prepared as `Status: READY` through
M14.4q at
`D:\SeaCad\AntigravityReports\seacad-m14.3dr-m14.4q-hatch-polyline-line-geometry-2026-08-09.yaml`.
It replaces the stale blocked M14.4o attempt; no independent Antigravity PASS is
claimed at this checkpoint.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 229 | `e37c9456c917cddafc1378a87fdcafd1043179524c6cfa69c862d24c135adcb5` |
| `README.vi.md` | 228 | `386d57eeb08073701491db223868f13bc3a1a24a80ff75f812a5472295b67c74` |
| `crates/seacad-dxf-core/src/hatch_polyline_line_geometry.rs` | 231 | `ce3448a1a1c766de669862eaf85282fd5340540523354ebf1e7f6d0b45d61e24` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,368 | `8125fa8f309455c54b6d4ccfb6bba6ce58776d32bf44f794d25538cdb4478b11` |
| `crates/seacad-dxf-core/tests/hatch_polyline_line_geometry_tests.rs` | 394 | `50a1a78ace72330aaacf82d1bad1cfb4f2cea1f20cb63117e938b7a23a2b2359` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `25f11c920071e95634975b4049c9b77ab1b12b4a2b12aa69994e9b0d6bdcfd94` |
| `docs/SUPPORT_MATRIX.md` | 3,178 | `b45a619dfd6a9f15fc182b76b87a9199014ead43b763bd99a43fdd8044c7dfae` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,090 | `4699944ef35b4e6fa0f3e2aa422a50e9a7a8eeece561866e37ca38e22cbf6daf` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,508 | `34f79d62db74fdf4b2050c75d892199c8c6e9a7efc5d17834b90ad978dc7022d` |
