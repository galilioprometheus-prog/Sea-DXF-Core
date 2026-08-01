# M14.3t HELIX Scalar Semantics

## Scope

M14.3t projects the seven non-coordinate HELIX roles through SeaCad's shared
`DxfSemanticValue<T, I>` model. It adds public enum domains documented by
Autodesk but does not infer parameter positivity, requiredness, or defaults.

## Evidence boundary

Autodesk defines the HELIX version, radius, turns, turn height, handedness, and
constraint fields. It explicitly defines handedness 0/1 and constraint type
0/1/2:

`https://help.autodesk.com/cloudhelp/2020/ENU/AutoCAD-DXF/files/GUID-76DB3ABF-3C8C-47D1-8AFB-72942D9AE1FF.htm`

The read-only legacy validator marks additional positivity and nonzero domains.
Those observations identify later validation risks but are not treated as
normative in this checkpoint:

| Legacy artifact | SHA-256 |
|---|---|
| `D:\SeaCad\tham khảo\New folder\cad_2026-07-23_source\crates\cad-dxf-lossless\src\semantic_helix_validation.rs` | `ab894332b09034c0197ac1bf381e36c86722f8a1ff3d8d27c68eb0d6de674cd0` |

No legacy or external code or fixture is copied, translated, vendored, or
linked.

## Contract

- Every retained HELIX has seven entries in stable scalar-role order, each
  carrying document, namespace, field, and optional exact raw provenance.
- Major and maintenance versions expose exact Int32 values without inventing a
  sign domain. Radius, turns, and turn height expose exact finite doubles;
  Binary NaN and infinity fail typed with their exact bits.
- Handedness maps 0 to left and 1 to right. Constraint type maps 0 to constrain
  turn height, 1 to constrain turns, and 2 to constrain height. Other values
  are typed invalid states with raw provenance.
- Absent cards become semantic `Absent`. Duplicate cards become typed
  `Invalid` without choosing or attaching one occurrence as the value.
- Invalid ASCII numbers remain typed invalid with exact raw provenance.
  Unexpected internal wire-role combinations fail closed.
- Construction uses checked capacity arithmetic, fallible allocation,
  source-identity verification, cancellation, and no new dependency.

## Dialect note

AC1009 parity retains absent handedness and constraint semantics because its
Binary one-byte entity group-code grammar cannot encode groups 290 and 280.
This does not assert HELIX applicability in AC1009.

## Nonclaims

M14.3t does not validate parameter positivity/nonzero relationships, require
fields, apply defaults, group WCS coordinates, validate axis vectors, compose
embedded SPLINE data, construct geometry, CRUD, write, review applicability,
or advance HELIX to `Complete`.

## Verification

The focused HELIX scalar-semantic suite passes 3/3 tests and the workspace
passes all 784 tests. Schema and release-evidence checks,
`cargo deny --locked check`, format, workspace Clippy with warnings denied,
workspace tests, the production forbidden-macro scan, and `git diff --check`
all pass. The checkpoint adds 366 physical production lines: 361 in the HELIX
scalar-semantic module and five module/export lines. This audit intentionally
omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 112 | `ebd8bbbc7539c9f1b64843d548b9f66c8e80596b12bda1cbf7b415abdd7ae72c` |
| `crates/seacad-dxf-core/src/lib.rs` | 916 | `778e4ba02b7a9bbed3ea2473359a0b513eace8ebf1e902a022a4c8f4d8f17099` |
| `crates/seacad-dxf-core/src/helix_scalar_semantic.rs` | 361 | `765bce8c03000c04589b170c3e2a215409ef8e68de8568926382bc4ebd01113c` |
| `crates/seacad-dxf-core/tests/helix_scalar_semantic_tests.rs` | 437 | `420b16a3f3b972ab9480eaaf53a85f938d715f9429ca9ec002491f5558f1918a` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 498 | `2e0abeef90a9321bb621d81328ec3781b37dcafe6edb08f95b9c120594a9a7e5` |
| `docs/IMPLEMENTATION_PLAN.md` | 1908 | `f5d2fae02b8bb4f1e3cddf8ffb9917f812b3f2df00268d7950ff94429ee2962b` |
| `docs/SUPPORT_MATRIX.md` | 1565 | `b435d08d20988df682c7f0c1e621b7c6327857fc25b07ba073a3689123551d02` |
