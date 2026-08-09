# M14.4v HATCH Boundary Line Edge Cards

M14.4v publishes four stable cardinality cards for every grouped M14.4u edge
typed as Line: StartX group 10, StartY group 20, EndX group 11, and EndY group
21. Every matching field remains exact source evidence, and each role reports
Absent, Unique, or Multiple independently. Count mismatch does not erase cards.
Non-Line and invalid typed edges publish no Line cards while the complete
M14.4u type/grouping/provenance and conservative payload slices remain
available. No numeric value is selected and no geometry is derived.

Autodesk documents these four fields in the
[Boundary Path Data table](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-DC5215D6-E73F-4DFF-8BE9-01CA9610FAEE.htm).

The focused suite passes 4/4 tests and workspace passes 1,177/1,177 tests. All
gates pass. No dependency, license, schema, release, corpus, or CI surface
changes. Production adds 333 lines: a 326-line module and seven module/export
lines. Numeric selection, required-coordinate semantics, OCS/WCS Line
geometry, circular/elliptic/spline edge payloads, HATCH applicability,
CRUD/write, rendering, and `Complete` remain open.

The earlier M14.4p Antigravity result was `BLOCKED` only because its retired
target tag did not equal the newer M14.4u HEAD; it reported no mutation and a
valid tag chain. The repository batch note and one cumulative external
`Status: READY` packet are synchronized through M14.4v at
`D:\SeaCad\AntigravityReports\seacad-m14.3dr-m14.4v-hatch-boundary-line-edge-cards-2026-08-09.yaml`.
No independent Antigravity PASS is claimed. This audit omits its own hash and
the self-referential active batch-note hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 241 | `dd59d8e7bd26f97bea66767a2a0f36de363d394df98ff7525c42a294f256cde6` |
| `README.vi.md` | 240 | `772dfa775f4e6b4afa9e444c4242857cbbe9752e8bb1b5916e73840dd27c1d63` |
| `crates/seacad-dxf-core/src/hatch_boundary_line_edge_card.rs` | 326 | `7ff7500c93a8e6b9753b191e4204176f63b2001864491eec72410f7092a92836` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,398 | `80dde3d12688c5da17178a8363270d55ffe192bc96b46979dcec6b20c4c7d995` |
| `crates/seacad-dxf-core/tests/hatch_boundary_line_edge_card_tests.rs` | 301 | `bbcb46b022fc568b0adf4d63c5d5dd18c3f627e9ac51ad718da70ca5b3edf59c` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `015a15d22fe37b2b52dcc554e3739b008622a7f500b6c17fa952c4cd15e6acc3` |
| `docs/SUPPORT_MATRIX.md` | 3,235 | `f8e65c1021e6a78540c25a0101101729d1f02de94861b39f8c426ba1f573f202` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,142 | `9463fecd3870c1aaad217acd3418bea274b4248b3784ff4c871faa274f1815df` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,561 | `3a9a56d6d0e6a74278d9cbe1c04a29b321ec7e9285030f62b933be3837880d3c` |
