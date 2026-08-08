# M14.4c HATCH Scalar Cardinality

## Scope

M14.4c adds one fixed cardinality card for every M14.4b HATCH scalar role and
every exact `AcDbHatch` subclass. It does not select occurrences or add typed
semantic defaults, domains, or cross-field relations.

## Evidence boundary

The role inventory and wire-domain evidence remain frozen by M14.4b from the
Autodesk HATCH, boundary-path, pattern-line, and group-value type references:

`https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-C6C71CED-CE0F-4184-82A5-07AD6241F15B.htm`

`https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-DC5215D6-E73F-4DFF-8BE9-01CA9610FAEE.htm`

`https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-7C05C0EC-B0FB-4A86-A164-B9E5C6C03990.htm`

No external or legacy implementation, fixture, data, or dependency was copied,
translated, vendored, linked, or consulted.

## Contract

- Every M14.4b HATCH scalar entry receives exactly 25 cards in
  `DXF_HATCH_SCALAR_ROLES` order, including cards with no occurrences.
- Card state is `Absent`, `Unique`, or `Multiple { occurrence_count }` solely
  from the exact number of retained typed occurrences for that role.
- Members are compact global occurrence ordinals. Resolving a member returns
  the original M14.4b occurrence with its raw group, role, value, and issue.
- Invalid numeric evidence remains a normal cardinality member. This layer
  never chooses a valid duplicate over an invalid one.
- Duplicate `AcDbHatch` markers receive independent card sets. Record-level
  lookup returns all such card sets contiguously without merging them.
- Construction is source-identity checked, cancellation-aware, fallibly
  allocated, and bounded by the existing raw document and evidence limits.

## Dialect boundary

All nine Core dialects have paired ASCII/Binary card and member parity. AC1009
retains all 25 cards while the nine group-450-through-470 roles are absent,
matching the M14.4b one-byte Binary group-code boundary. No applicability claim
is inferred.

## Nonclaims

M14.4c does not add selected values, defaults, domains, flags semantics, count
relations, tuples, boundary/path or pattern state, gradient relations, geometry,
rendering, applicability, CRUD/write, corpus qualification, or `Complete`.

## Verification

The focused HATCH cardinality suite passes 4/4 tests and the workspace passes
1,101/1,101 tests. Dependency policy, formatting, schema, release evidence,
workspace Clippy with warnings denied, safety/link/protected-surface scans, and
whitespace checks pass. No dependency, schema, license, release, corpus, or CI
configuration changes. The checkpoint adds 331 physical production lines: 326
in the cardinality module and five module/export lines. This audit intentionally
omits its own self-referential hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 196 | `67493c28df2833f78301b9b21c1432e173df4930041d1d60dabea9cfe8679b14` |
| `README.vi.md` | 194 | `f569b45183a3aec5ee30b97624208b8898aaf408e60984e3ed53307c8b062cf9` |
| `crates/seacad-dxf-core/src/hatch_scalar_card.rs` | 326 | `27a0d63020afa8fee257b54ea051c467c3f7dfbc7271c952d1cca3d06d0a133a` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,284 | `8f64f4e7516e8075429f9fdf7a3d830abdbbea54fd664fbf86a44153ac2ac823` |
| `crates/seacad-dxf-core/tests/hatch_scalar_card_tests.rs` | 424 | `88947fdc8c9827ee3efdfa556912062e3f1833835d7890c6db97b3db3269a439` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `07273e72287440669d7dc64706bd5fd365a516a862c723dc4fee24c4e7108a87` |
| `docs/SUPPORT_MATRIX.md` | 3,027 | `5500899e8e16a12c29bcbfd1b32680257d9753c40a3231d6135df4b9ed2f73a3` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 1,954 | `afdac35738d176379f897f9f0b14f4fcd775d80589854989dad2f3e136f57cd9` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,379 | `8c7b2aad8d87a6f108576d8b52b57b38a54d5f957157e03ba09600fe0f115c64` |
