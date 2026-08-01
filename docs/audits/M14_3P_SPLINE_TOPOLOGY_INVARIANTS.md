# M14.3p SPLINE Topology Invariants

## Scope

M14.3p composes M14.3o relations with exact degree, knot, and control-anchor
evidence. It reports topology readiness without evaluating, tessellating,
repairing, or rewriting a curve.

## Evidence boundary

Autodesk's 2024 SPLINE table defines group 71 as degree, groups 72 and 73 as
the knot and control-point counts, group 40 as the repeated knot sequence, and
bits 1 and 2 as closed and periodic:

`https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-E1F884F8-AA90-4864-A215-3182D47A9C74.htm`

The relation `knot_count = control_point_count + degree + 1` is the defining
full-knot-vector B-spline relation and is exposed as analytic-readiness
evidence, not as an extra Autodesk table claim. The read-only legacy validator
confirms observed nonpositive-degree and decreasing-knot diagnostics but does
not implement the NURBS count relation.

| Legacy artifact | SHA-256 |
|---|---|
| `D:\SeaCad\tham khảo\New folder\cad_2026-07-23_source\crates\cad-dxf-lossless\src\semantic_spline_validation.rs` | `f1d1e07660ed61dae2ed0170817a4947bd55b589397e59964ccb932183258104` |

No legacy or external code or fixture is copied, translated, vendored, or
linked.

## Contract

- Every retained SPLINE has one stable topology entry backed by the owned
  relation/evidence chain.
- Degree remains absent, duplicate, invalid, nonpositive, or explicit; no
  duplicate singleton occurrence is selected.
- Knot ordering accepts equality, records the first decreasing right-member
  index, and becomes unavailable when any knot is invalid. Invalid members are
  counted rather than skipped.
- A positive degree is compared with the observed group-10 control-anchor
  count and the minimum `degree + 1` controls.
- Observed knots are compared with `controls + degree + 1`; checked arithmetic
  exposes expected-count overflow instead of wrapping.
- Open, nonperiodic-closed, periodic-closed, and periodic-without-closed are
  distinct observations. The platform does not invent a closed/periodic rule
  absent from the cited DXF table.
- Construction is linear in knot evidence, uses checked compact metadata,
  fallible reservation, source-identity checks, and cancellation throughout.

## Nonclaims

M14.3p does not reinterpret declared count errors already exposed by M14.3e,
materialize point coordinates, validate knot multiplicity or endpoint clamps,
derive a parameter domain, construct or evaluate NURBS data, process HELIX,
edit, write, or advance SPLINE to `Complete`.

## Verification

The focused topology suite passes 3/3 tests and the workspace passes all 768
tests. Schema and release-evidence checks, `cargo deny --locked check`, format,
workspace Clippy with warnings denied, workspace tests, the production
forbidden-macro scan, and `git diff --check` all pass. The checkpoint adds 456
physical production lines: 450 in the topology module and six module/export
lines. This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 97 | `824cde052c1c85f90c92c552bec1d9c4e331852bdcc8399bb1c9c31cacbf8464` |
| `crates/seacad-dxf-core/src/lib.rs` | 890 | `55d8324ef884349d3441724ce768f8f9116066510b5e7f383ff8129b4ec2956a` |
| `crates/seacad-dxf-core/src/spline_topology.rs` | 450 | `94c9d50f119d68a1555634e21829bc51d62e9fbbc8f051493a8eac00909b85f5` |
| `crates/seacad-dxf-core/tests/spline_topology_tests.rs` | 435 | `cec1eb854d480fb943a3532d09cfaeea3196398b0ebf349f29f40dbff58cc47d` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 448 | `e4aa5c98ec9f9df14807c33db4b301dad4113a8f43ec50c6ff42c6d90cd782e4` |
| `docs/IMPLEMENTATION_PLAN.md` | 1849 | `095919750ad95528fce6c0371386d72773ee1b31e209c9148f35afa6b72bd05e` |
| `docs/SUPPORT_MATRIX.md` | 1515 | `a76d8757eeb53d4a9bb059ac8f0b983f4e20e5963170787e2f7db22f50e599db` |
