# M14.4i HATCH Boundary Path Flags

## Scope

M14.4i decodes and validates each M14.4h group-92 marker, exposes documented
flag helpers, and classifies valid paths as Polyline or Edges.

## Normative evidence

Autodesk defines group 92 as a bit-coded path type: Default 0, External 1,
Polyline 2, Derived 4, Textbox 8, and Outermost 16:

`https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-DC5215D6-E73F-4DFF-8BE9-01CA9610FAEE.htm`

No external or legacy implementation, fixture, data, or dependency was copied,
translated, vendored, linked, or consulted.

## Contract

- Every M14.4h path receives one independent flag entry with exact raw marker.
- Any nonnegative combination within mask `0x1F` is valid. Helpers expose all
  five bits; bit 2 selects Polyline and its absence selects Edges. Zero is valid.
- Malformed ASCII, negative values, and bits outside `0x1F` remain typed issues
  with raw provenance and no path-kind classification.
- Duplicate subclasses remain isolated. Construction is cancellation-aware,
  fallibly allocated, source-identity checked, and resource bounded.

## Nonclaims

M14.4i does not decode polyline/edge payloads, validate their cardinality,
resolve handles, derive geometry, establish applicability, add CRUD/write,
qualify a corpus, or claim `Complete` support.

## Verification

The focused path-flag suite passes 4/4 tests and the workspace passes
1,125/1,125 tests. All required gates and safety/link/protected checks pass.
No dependency, schema, license, release, corpus, or CI change. The checkpoint
adds 315 physical production lines: 309 in the flag module and six module/export
lines. This audit omits its own self-referential hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 212 | `81b3d2a498123243eb75755145469ba8e31941c43b297ad4e2815480a64adcbb` |
| `README.vi.md` | 210 | `999fe5921e853d851ebda2e8c06386c9edc54f1d9e7c53fc76d61104bd6982ab` |
| `crates/seacad-dxf-core/src/hatch_boundary_path_flags.rs` | 309 | `16d27be5eeae57b70814975594526b2b187e084bac6d0d4cadb12a8992647284` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,320 | `483a616c541bdefd2ba76995f104d0a8ad052ae9c994669024aae286c0d57456` |
| `crates/seacad-dxf-core/tests/hatch_boundary_path_flag_tests.rs` | 256 | `408c222b00e3e270007a1286a42a6ab54d8901ac49708eba3050a7253d6566d3` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `538808b22e58554062d3439307b0ad9ef79948484a5133e7f9b26cdb6c00daa8` |
| `docs/SUPPORT_MATRIX.md` | 3,103 | `4bb5d9ca44922144601c2b0cbf49e0512c7668b70478d592fa9ec49733e13129` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,020 | `81ee37fee62278655620461177992007fba8d2a4c744439642a340d53892b467` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,440 | `17aa250e7f5fee8bf2d5c8008fa0e8d0d4aa1534ed4fcf65b2a4dc87a02b4d94` |
