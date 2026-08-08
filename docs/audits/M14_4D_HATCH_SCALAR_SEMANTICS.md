# M14.4d HATCH Scalar Semantics

## Scope

M14.4d selects the 25 M14.4c cards into source-anchored singleton semantics,
applies only the unconditional documented extrusion default, and validates
individual documented domains. It does not evaluate cross-field relations.

## Normative evidence

Autodesk defines the optional extrusion default `(0,0,1)`, flag values,
hatch-style and pattern-type domains, counts, and gradient fields in the HATCH
table:

`https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-C6C71CED-CE0F-4184-82A5-07AD6241F15B.htm`

The boundary-path and pattern-line tables continue to delimit fields excluded
from this scalar layer:

`https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-DC5215D6-E73F-4DFF-8BE9-01CA9610FAEE.htm`

`https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-7C05C0EC-B0FB-4A86-A164-B9E5C6C03990.htm`

No external or legacy implementation, fixture, data, or dependency was copied,
translated, vendored, linked, or consulted.

## Contract

- Each exact HATCH subclass has 25 ordered semantic entries with stable
  `entity.hatch` field ids and exact document identity.
- Absent extrusion X/Y/Z defaults independently to `0/0/1`. Every other absent
  scalar stays absent; conditional gradient defaults are not invented.
- Unique valid values are explicit with exact raw group occurrence and payload
  span. Multiple cards are invalid with their exact count and no selected raw
  member.
- Invalid ASCII syntax/range and non-finite Binary doubles are typed invalid
  with exact raw provenance.
- Individual domains cover 0/1 flags; style/type 0-2; nonnegative path, seed,
  and pattern-line counts; gradient kind/mode 0-1; reserved group 451 = 0;
  color count 0 or 2; shift/tint 0-1; and reserved group 463 = 0 or 1.
- Construction is source-identity checked, cancellation-aware, fallibly
  allocated, and bounded by existing evidence limits.

## Nonclaims

M14.4d does not add elevation/extrusion tuples, zero-vector validation,
requiredness, solid/pattern conditional fields, boundary or seed count
relations, group-450 gradient envelope relations/defaults, nested boundary or
pattern state, geometry, rendering, applicability, CRUD/write, corpus
qualification, or `Complete`.

## Verification

The focused HATCH semantic suite passes 4/4 tests and the workspace passes
1,105/1,105 tests. Dependency policy, formatting, schema, release evidence,
workspace Clippy with warnings denied, safety/link/protected-surface scans, and
whitespace checks pass. No dependency, schema, license, release, corpus, or CI
configuration changes. The checkpoint adds 385 physical production lines: 380
in the semantic module and five module/export lines. This audit intentionally
omits its own self-referential hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 198 | `b4917b4f3bbc973a66df6fcec5a89b4c3388b5cef91cdc77c4adda45c84f6cd9` |
| `README.vi.md` | 196 | `c7d2094dc5cc351a85cf33b86432640280823ae7d19851c902681635a85734f9` |
| `crates/seacad-dxf-core/src/hatch_scalar_semantic.rs` | 380 | `696c5c142f68cb286e1a3c9eb7cf07d1f417bf025987f71f5f17d1f28641151d` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,289 | `b97ee181e9d45adf60feb1668e6d24d17f6c357a6360f75fdc1eff8b11784edc` |
| `crates/seacad-dxf-core/tests/hatch_scalar_semantic_tests.rs` | 497 | `fb2fc1d5b98409c0cedaadacc39ad05c45b324352e91b9a77d04e1ab2b82320b` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `d7eba0807a45d9dd85cc196bc273324ccf242069da9953dcd962b523eff789b9` |
| `docs/SUPPORT_MATRIX.md` | 3,040 | `ea661cbd72f4f6b047eec76c5cb987008be9283a14898706ac978cef888af296` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 1,967 | `21a150449367d8ba98d565bc4cc89e065f5fdabe97d78ebb51f7734d00968b13` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,390 | `94ccbf7663e93a9fdce4c14c2e86cb668165395174a2e8af16fe0d19131dc556` |
