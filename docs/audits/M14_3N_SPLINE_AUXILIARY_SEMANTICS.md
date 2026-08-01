# M14.3n SPLINE Auxiliary Semantics

## Scope

M14.3n materializes effective group-41 weights and optional tangent/normal
vectors from M14.3m evidence. Unusable evidence stays typed and source-backed;
the projection does not evaluate curve or flag invariants.

## Evidence boundary

Autodesk's 2024 SPLINE table states that group-41 weight values are omitted
when they are 1, that start/end tangents may be omitted, and that normal Y/Z
components are optional:

`https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-E1F884F8-AA90-4864-A215-3182D47A9C74.htm`

The read-only legacy implementation confirms observed zero defaults for
missing optional vector components and reports nonpositive weights. The
nonpositive classification remains explicitly behavioral: M14.3n exposes it
but makes no Autodesk support claim from that oracle.

| Legacy artifact | SHA-256 |
|---|---|
| `D:\SeaCad\tham khảo\New folder\cad_2026-07-23_source\crates\cad-dxf-lossless\src\semantic_spline_validation.rs` | `f1d1e07660ed61dae2ed0170817a4947bd55b589397e59964ccb932183258104` |
| `D:\SeaCad\tham khảo\New folder\cad_2026-07-23_source\crates\cad-dxf-lossless\src\semantic_spline_decode.rs` | `a8dd9117f21d44497eb9b620c6aa1ca539f7c3658de7bb3b94e599c63635c4fc` |

No legacy code or fixture is copied, translated, vendored, or linked.

## Contract

- A missing weight sequence remains `ImplicitUnit` with the observed
  control-point count; no synthetic per-point raw evidence is created.
- Explicit matched and count-mismatched sequences remain distinct. Every
  explicit member becomes `Explicit`, `Invalid`, or behavioral `NonPositive`
  while retaining its exact `DxfSplineValue`.
- A wholly absent start tangent, end tangent, or normal remains `Absent`.
- A present vector requires a unique valid X. Unique missing Y/Z components
  receive effective zero values, and the result retains an explicit-component
  mask so defaulted and source-provided values cannot be confused.
- Duplicate components, invalid numerics, and missing X produce `Unavailable`
  with independent masks/flags. No duplicate occurrence is chosen.
- Construction uses checked compact ranges, fallible reservations, bounded
  three-vector expansion, and cancellation checks before/during/after work.

## Nonclaims

M14.3n does not reject an entity because a weight is nonpositive; validate the
rational, planar, or linear flags; decide whether a zero tangent/normal is
usable; materialize control/fit WCS points; reconcile degree/count/knot
topology; construct analytic NURBS data; process HELIX; migrate the older
SPLINE API; edit; write; or advance any entity to `Complete`.

## Verification

The focused semantic suite passes 3/3 tests and the workspace passes all 762
tests. Schema and release-evidence checks, `cargo deny --locked check`, format,
workspace Clippy with warnings denied, workspace tests, and `git diff --check`
all pass. The checkpoint adds 501 physical production lines (499 net); its
primary semantic module is 491 lines. This audit intentionally omits its own
hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 97 | `67e7a7bb7eed34e93959c33202a9517434286414901d4ad21071592a68912dce` |
| `crates/seacad-dxf-core/src/lib.rs` | 879 | `5b539e825b4dd551730774fa262e5805772ba1000e860ad0235d71f994c9a7e6` |
| `crates/seacad-dxf-core/src/spline_auxiliary.rs` | 498 | `502335037339a7942eefe45ad4afd8c601871b9ee7c7d7f51e6f9df2935b40d9` |
| `crates/seacad-dxf-core/src/spline_auxiliary_semantic.rs` | 491 | `41216d42a09a4e124a4d4cf816a5f4a741ad8e8dfe71b253ce491aa670a22197` |
| `crates/seacad-dxf-core/tests/spline_auxiliary_semantic_tests.rs` | 446 | `7d66c0cd6a57c3b72627733e7aaea060e431d49482effee4a4b8e5b3adb031d1` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 428 | `f2ea35e09e6816c7fda37628cbf432350f6922f490f328a7d9b8fe182483c027` |
| `docs/IMPLEMENTATION_PLAN.md` | 1826 | `155332a333f1d7fa148a64a663d9c58ac77693c3cf0f3db70a111f1036b6bba9` |
| `docs/SUPPORT_MATRIX.md` | 1496 | `828930dbeab23a7df6bda81cdd942a0804adc9a5af640987e6d204a10b6eefc9` |
