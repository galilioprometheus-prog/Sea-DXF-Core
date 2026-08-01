# M14.3r HELIX Source Evidence

## Scope

M14.3r adds source-anchored raw numeric evidence for the public `AcDbHelix`
subclass. It does not select duplicate singleton values or apply defaults,
domains, parameter relationships, embedded SPLINE semantics, or geometry.

## Evidence boundary

Autodesk defines HELIX as spline data followed by `AcDbHelix`, with group 90
and 91 versions, group 10/20/30 axis base, group 11/21/31 start point, group
12/22/32 axis vector, groups 40/41/42 radius/turns/turn height, group 290
handedness, and group 280 constraint type:

`https://help.autodesk.com/cloudhelp/2020/ENU/AutoCAD-DXF/files/GUID-76DB3ABF-3C8C-47D1-8AFB-72942D9AE1FF.htm`

Autodesk also requires table-driven, order-independent entity processing and
places entities in both `BLOCKS` and `ENTITIES`:

`https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-A35B8C2A-1885-4A8E-8533-E61D8A423D62.htm`

Two read-only legacy artifacts were consulted only to identify subclass-code
collision risks and expected wire families:

| Legacy artifact | SHA-256 |
|---|---|
| `D:\SeaCad\tham khảo\New folder\cad_2026-07-23_source\crates\cad-dxf-lossless\src\semantic_spline.rs` | `2ffabd938f01594f70a9edd87b84bfa5c2b92472c8d83f7eb2e0ec9e2b1587f2` |
| `D:\SeaCad\tham khảo\New folder\cad_2026-07-23_source\crates\cad-dxf-lossless\src\semantic_spline_decode.rs` | `a8dd9117f21d44497eb9b620c6aa1ca539f7c3658de7bb3b94e599c63635c4fc` |

No legacy or external code or fixture is copied, translated, vendored, or
linked.

## Contract

- Canonical exact `HELIX` records in `BLOCKS` and `ENTITIES` receive one stable
  record entry linked to their `DxfEntityRef` and source identity.
- Only groups after an exact `AcDbHelix` marker and before the next different
  subclass marker enter HELIX evidence. Embedded SPLINE data, case variants,
  near matches, and application-group content do not collide.
- All 16 documented roles retain exact `DxfRawGroup` provenance and their raw
  typed result. Versions decode as Int32, constraint type as Int16, handedness
  as ASCII integer or Binary BooleanByte, and coordinates/parameters as exact
  double values or invalid ASCII numeric states through bounded decoders.
- Duplicates and invalid ASCII numbers remain source ordered; this layer never
  chooses an occurrence or applies a semantic domain.
- Construction is bounded, cancellation-aware, source-identity checked, uses
  fallible allocation, and adds no dependency.

## Dialect note

AC1009 Binary cannot encode group codes 280 and 290 in its one-byte entity
group-code grammar, so the nine-dialect parity fixture omits those two roles in
both physical formats for AC1009. This is a wire-boundary observation, not an
applicability or HELIX-support claim for that dialect.

## Nonclaims

M14.3r does not add HELIX cardinality, defaults, domain validation, parameter
relationships, embedded SPLINE composition, geometry, CRUD, writes, reviewed
version applicability, or `Complete` support status.

## Verification

The focused HELIX evidence suite passes 4/4 tests and the workspace passes all
778 tests. Schema and release-evidence checks, `cargo deny --locked check`,
format, workspace Clippy with warnings denied, workspace tests, the production
forbidden-macro scan, and `git diff --check` all pass. The checkpoint adds 414
physical production lines: 409 in the HELIX evidence module and five
module/export lines. This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 106 | `391d4200923f829919523ea98a30a14dc1d7febed33c06cead2609820d0f45af` |
| `crates/seacad-dxf-core/src/lib.rs` | 906 | `0254a20f01e5b382f45d1364132db2c7bbd3d02e3ee31663c9771f93bfb916e5` |
| `crates/seacad-dxf-core/src/helix_evidence.rs` | 409 | `d44a4cc5bb444cef5bef11fbe62570fd41ef099eb70d59e576c43e414d2ebf89` |
| `crates/seacad-dxf-core/tests/helix_evidence_tests.rs` | 390 | `b5cb1291cd29e0001becfe1a2a12f12229eb0ed83925b66d375bb33c4b755f4a` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 476 | `0ce761c5790109c3b1bd0879f606e321401dbb9a27d2cff43b53f73767b71e6b` |
| `docs/IMPLEMENTATION_PLAN.md` | 1883 | `ec43eb23ce676aa2ae473d7a17d6dec344237101b45c816b1b456948797f0911` |
| `docs/SUPPORT_MATRIX.md` | 1546 | `e8b11c6f8de1ff0ae0dfe62aeb3fd4a58fabd2af1350b1c7232bff53c4c7f0dc` |
