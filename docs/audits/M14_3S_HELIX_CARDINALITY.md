# M14.3s HELIX Cardinality

## Scope

M14.3s adds fixed per-role cardinality over the exact HELIX source evidence
from M14.3r. It does not select values or infer defaults and requiredness.

## Evidence boundary

Autodesk's HELIX table remains the normative inventory for the 16 roles:

`https://help.autodesk.com/cloudhelp/2020/ENU/AutoCAD-DXF/files/GUID-76DB3ABF-3C8C-47D1-8AFB-72942D9AE1FF.htm`

M14.3r already records the source and legacy-oracle boundary in
`docs/audits/M14_3R_HELIX_SOURCE_EVIDENCE.md`. This checkpoint consumes only
SeaCad's owned evidence API and consults no additional external implementation,
fixture, or corpus artifact.

## Contract

- Every indexed HELIX record receives exactly 16 cards in `DXF_HELIX_ROLES`
  order, independent of source group order.
- A card is `Absent`, `Unique`, or `Multiple { occurrence_count }`. Invalid
  numeric syntax still counts as a source occurrence.
- Each card owns a checked compact half-open member range. Members point to the
  exact source-ordered `DxfHelixValue` entries and expose no value-selection
  shortcut.
- Records with no `AcDbHelix` values still receive 16 absent cards. Multiple
  records remain source ordered and independently addressable by raw ordinal.
- Construction uses checked capacity arithmetic, fallible reservation,
  source-identity verification, cancellation, and no new dependency.

## Dialect note

As in M14.3r, AC1009 Binary cannot encode groups 280 and 290 in its one-byte
entity group-code grammar. Its parity fixture therefore has absent cards for
handedness and constraint type in both physical formats. This remains a wire
observation rather than an applicability claim.

## Nonclaims

M14.3s does not define required/optional fields, defaults, typed semantic
domains, coordinate tuple semantics, axis/radius/turn/height relationships,
embedded SPLINE composition, geometry, CRUD, writes, reviewed applicability,
or `Complete` status.

## Verification

The focused HELIX cardinality suite passes 3/3 tests and the workspace passes
all 781 tests. Schema and release-evidence checks, `cargo deny --locked check`,
format, workspace Clippy with warnings denied, workspace tests, the production
forbidden-macro scan, and `git diff --check` all pass. The checkpoint adds 323
physical production lines: 318 in the HELIX cardinality module and five
module/export lines. This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 109 | `43f63c7787386770e39f839559e8fb617480ca846bbeb9d875cdf6cdd14d6b12` |
| `crates/seacad-dxf-core/src/lib.rs` | 911 | `65fbf065d9260c231c2752829df5412fb9f4078e955d261ce490fb29a3143745` |
| `crates/seacad-dxf-core/src/helix_card.rs` | 318 | `6e78406f1cb50cdae136829613c6253fc73e90b0d3cfe2873e8326794916ef20` |
| `crates/seacad-dxf-core/tests/helix_card_tests.rs` | 381 | `627a3b624483eeef2707043f806157533ee1c3198bbb6e82be723be633e4c111` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 486 | `4b3230d7c676ecb9635dd32aed89b13c5924f316b9c55f74becf664afa120e46` |
| `docs/IMPLEMENTATION_PLAN.md` | 1895 | `15723637bd8a6d22437f1e1d70d61578c39bf1ea753092724bc49009675ddb9f` |
| `docs/SUPPORT_MATRIX.md` | 1555 | `0f7269e667b88170a5950d954249b19098fd93cadb37ed7ed7b7161ef6d879db` |
