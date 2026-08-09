# M14.4u HATCH Boundary Edge Types

M14.4u decodes exactly one signed 16-bit group-72 discriminator for every
grouped M14.4t HATCH boundary edge. Values 1, 2, 3, and 4 publish Line,
CircularArc, EllipticArc, and Spline. Malformed ASCII remains an exact
source-anchored InvalidAsciiNumber issue and every other signed 16-bit value
remains ValueOutOfDomain. Count mismatches retain independently known types;
empty grouped paths expose an empty typed slice, while Polyline and unavailable
paths expose no typed entries. Complete M14.4t grouping, marker provenance, and
conservative payload slices remain available without selecting edge payload
fields or deriving geometry.

Autodesk documents the four group-72 values in the
[HATCH group-code table](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-C6C71CED-CE0F-4184-82A5-07AD6241F15B.htm).

The focused suite passes 4/4 tests and workspace passes 1,173/1,173 tests. All
gates pass. No dependency, license, schema, release, corpus, or CI surface
changes. Production adds 211 lines: a 206-line module and five module/export
lines. Line/circular/elliptic/spline edge payload fields and geometry remain
unselected. HATCH entity applicability remains `NotYetReviewed` because the
reviewed Autodesk sources do not directly prove a version floor. CRUD/write,
rendering, and `Complete` remain open. This audit omits its own hash.

One cumulative Antigravity review packet is prepared as `Status: READY` through
M14.4u at
`D:\SeaCad\AntigravityReports\seacad-m14.3dr-m14.4u-hatch-boundary-edge-types-2026-08-09.yaml`.
It supersedes the M14.4t packet; no independent Antigravity PASS is claimed at
this checkpoint.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 238 | `0ed553c7a5864cce75eef0c9af1ea2578065e437fbb3998243f6feac433d47e8` |
| `README.vi.md` | 237 | `597e4e4587e344fa8c092eb98acec3a7aed906c7a2b5694f63349c18f6a953d8` |
| `crates/seacad-dxf-core/src/hatch_boundary_edge_type.rs` | 206 | `f250d662af2782e907745fc62afcc14ff3fa88058a146942a938f5f111798dfa` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,391 | `10170a59aa3745992eafc4f2a82446e3333b0f2dbe1184f36d3f3754e0a4b844` |
| `crates/seacad-dxf-core/tests/hatch_boundary_edge_type_tests.rs` | 281 | `840a3bda352e35af6977aaaa1a12e48178a255070ce2095102b1b7ce68358dd9` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `f92772420038a33224cdbf8126e3f0b7bc867437f3f5d3fa2c17560ffe9aab2c` |
| `docs/SUPPORT_MATRIX.md` | 3,225 | `8fa3a7a88b7a12daa640718d861d6a10181d992cdec6d846066f26fc416b924e` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,132 | `ae733dff397897aac6c03ff27b91e1d08b36cb72380c64ad660c1aebcf351706` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,551 | `0c7e2b16532a58f41ae3d5deb1bd2848afe9bb50ed98bc37828bfdab9a76b08e` |
