# M14.3u HELIX WCS Vector Semantics

## Scope

M14.3u projects the nine HELIX coordinate components into axis-base,
start-point, and axis-vector WCS semantics. It does not validate the axis or
apply implicit component defaults.

## Evidence boundary

Autodesk defines group 10/20/30 as the axis base point, group 11/21/31 as the
start point, and group 12/22/32 as the axis vector:

`https://help.autodesk.com/cloudhelp/2020/ENU/AutoCAD-DXF/files/GUID-76DB3ABF-3C8C-47D1-8AFB-72942D9AE1FF.htm`

The cited table labels the triples but does not publish omission defaults. The
read-only legacy validator behaviorally defaults omitted Y/Z to zero and
rejects a near-zero axis. Those observations are retained as risks for a later
explicitly evidenced relation milestone, not imported as normative rules here:

| Legacy artifact | SHA-256 |
|---|---|
| `D:\SeaCad\tham khảo\New folder\cad_2026-07-23_source\crates\cad-dxf-lossless\src\semantic_helix_validation.rs` | `ab894332b09034c0197ac1bf381e36c86722f8a1ff3d8d27c68eb0d6de674cd0` |

No legacy or external code or fixture is copied, translated, vendored, or
linked.

## Contract

- Every retained HELIX has exactly three vector-semantic entries in stable
  axis-base, start-point, axis-vector order.
- Every entry has X/Y/Z `DxfSemanticValue` components with document, namespace,
  field, and exact raw provenance when one occurrence exists.
- Unique finite doubles are explicit. Absence remains absent. Duplicate,
  invalid ASCII, and Binary NaN/infinity evidence becomes typed invalid without
  selecting a component value.
- `vector_value()` returns an exact WCS triple only when all three components
  are usable. It never fabricates missing Y/Z values.
- An explicit zero axis is retained unchanged; validity and normalization are
  separate relation/geometry concerns.
- Construction uses checked capacity arithmetic, fallible allocation,
  source-identity verification, cancellation, and no new dependency.

## Nonclaims

M14.3u does not default components, validate or normalize the axis, validate
radius/turn/height relations, compose embedded SPLINE data, construct analytic
HELIX geometry, CRUD, write, review applicability, or advance HELIX to
`Complete`.

## Verification

The focused HELIX WCS-vector suite passes 3/3 tests and the workspace passes
all 787 tests. Schema and release-evidence checks,
`cargo deny --locked check`, format, workspace Clippy with warnings denied,
workspace tests, the production forbidden-macro scan, and `git diff --check`
all pass. The checkpoint adds 340 physical production lines: 335 in the HELIX
vector-semantic module and five module/export lines. This audit intentionally
omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 115 | `5a05dfe5049d74715b90cad332dea721121d927acf87964d8e2be50b2623d91a` |
| `crates/seacad-dxf-core/src/lib.rs` | 921 | `64e11507b19df9d7b2468d08a7f18e0f71dc86f0f5be4501c7b84bf5dd3226eb` |
| `crates/seacad-dxf-core/src/helix_vector_semantic.rs` | 335 | `6a54498043e227fc823bedb66f789d6d548997d44e03d20ffcaaedb02c4315d1` |
| `crates/seacad-dxf-core/tests/helix_vector_semantic_tests.rs` | 394 | `bde7be531d4c677b29f5fc6210a8d6ca274e29e83fe8166ec6bc5ed082ccaa43` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 509 | `4159e1c34e5c1770030aa21a7b14040c44392d6c497434e3c3860b3cc0193f89` |
| `docs/IMPLEMENTATION_PLAN.md` | 1921 | `aeb57fec5b5d990fa3c33a8f6372c546de3c1bc42fc74d2451cfce15e101954f` |
| `docs/SUPPORT_MATRIX.md` | 1574 | `8da0ab7a138450caa2bf15e03da639bea39dd1689e85257780deff92c451ee71` |
