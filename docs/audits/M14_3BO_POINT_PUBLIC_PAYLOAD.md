# M14.3bo POINT Public Payload

Retrieved: 2026-08-02

## Scope

M14.3bo closes the public POINT payload gap on the existing read and insert
paths. It adds thickness, extrusion, and the UCS X-axis angle without widening
the claim to POINT update, clone, delete, display, or `Complete` support.

## Normative evidence

- Autodesk [POINT (DXF)](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-9C6AD32D-769D-4213-85A4-CA9CCB5C5317.htm)
  defines required WCS location `10/20/30`, optional thickness `39` defaulting
  to zero, optional extrusion `210/220/230` defaulting to `(0, 0, 1)`, and
  optional group `50` as the UCS X-axis angle in effect when the point was
  drawn, defaulting to zero and used when PDMODE is nonzero.
- Autodesk [Common Group Codes for Entities](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm)
  states that consumers must not depend on the presentation order of group
  codes. SeaCad therefore reads source order independently while new records
  follow the POINT table's location, thickness, extrusion, angle order.

The earlier normative and clean-room boundary remains recorded in
`docs/audits/M14_3BK_POINT_DRAFT_RECORD.md`. No external or legacy code,
fixture, data, dependency, or generated artifact was copied, translated,
vendored, linked, or used at runtime for this checkpoint.

## Contract

- POINT evidence retains groups `39` and `50` in exact source order with raw
  spans and lexical failures. LINE does not acquire colliding scalar roles.
- Every POINT has eight stable cards: required WCS location, optional
  thickness, optional extrusion components, and optional angle. Absence,
  uniqueness, and duplicates remain explicit; no occurrence is selected.
- Typed POINT semantics default absent thickness and angle to positive zero and
  absent extrusion components to `(0, 0, 1)`. Present invalid or duplicate
  evidence remains invalid with provenance and never falls back to a default.
- `DxfPointDraft` accepts optional thickness, a complete extrusion tuple, and
  optional UCS X-axis angle. Unspecified values remain omitted. Explicit values
  retain exact finite binary64 bits; an all-zero extrusion vector is rejected.
- Canonical records emit optional family groups after location in `39`,
  `210/220/230`, `50` order for every supported ASCII and Binary dialect.
- Insertion verification compares the exact effective values and also requires
  explicit state for supplied values or defaulted state for omitted values.
  Silently dropping a requested default-valued field therefore cannot pass.

## Verification boundary

Paired minimal and all-explicit records cover ASCII and Binary AC1009 through
AC1032. Evidence tests include out-of-order fields, duplicates, malformed
numbers, wrong entity kind, wrong section, exact bits, fixed-card membership,
and cancellation. Semantic tests cover explicit/defaulted states and retained
invalid provenance. Draft tests cover canonical bytes, non-finite optional
values, zero extrusion, strict reparse, semantic verification, create-new
write, tampering of each optional field, and byte-identical inverse. Existing
three-record session tests exercise the all-explicit payload in one shared
reservation.

## Nonclaims

This checkpoint does not normalize the extrusion, derive display axes, inspect
PDMODE, render a point, add family update/reset, mix inserts with raw-ordinal
updates, clone/delete a closed set, insert another family, or advance POINT to
`Complete`.

## Verification

The four focused evidence/card, semantic, draft-record, and insert-session
suites passed 19/19 tests. The full workspace passed 910/910 tests. Generated-
schema and release-evidence checks, `cargo deny --locked check`, formatting,
workspace Clippy with warnings denied, the production forbidden-macro scan,
and `git diff --check` all passed. Production changes are 202 insertions and
9 deletions; focused test changes are 267 insertions and 30 deletions. No
manifest, lockfile, production dependency, fixture, or generated schema
changed. This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 294 | `433deed3abd7c3e7ff8de2289cb5cfdb94ef5d0e91cce7ae1abc7a92964202e7` |
| `crates/seacad-dxf-core/src/basic_geometry.rs` | 347 | `750ad27dcb18231809cb82558a164bbdae1e47b25cebe6b8a726f2d01976a7f4` |
| `crates/seacad-dxf-core/src/basic_geometry_card.rs` | 340 | `07eb6e071e6bfea351dcb3d22f83ffc263bf93dc29724f7f69a1072345c93788` |
| `crates/seacad-dxf-core/src/basic_geometry_semantic.rs` | 531 | `a81d89d60fd917b928e16a8fd9b7208ed8083cafe10deedb50fe20450edf7f5a` |
| `crates/seacad-dxf-core/src/entity_draft_record.rs` | 756 | `553ad013a79d571685cea1c1d32cbf15580eb650eac576c933bdda853fd6adf8` |
| `crates/seacad-dxf-core/src/entity_edit_verification.rs` | 917 | `601b1847e2cdff0452fe24fdeb724504bc9ffaad39c8543ee739bceb8701f759` |
| `crates/seacad-dxf-core/tests/basic_geometry_tests.rs` | 501 | `ea075cac7683db0f28674ef250f7ab7fc54c5141ae547c2f1fcc4ef888d13212` |
| `crates/seacad-dxf-core/tests/basic_geometry_semantic_tests.rs` | 408 | `c7182b022e51adb093658c3ca5b39b212f62f889569cf84232cf6fd53e3839f5` |
| `crates/seacad-dxf-core/tests/entity_draft_record_tests.rs` | 1,246 | `e90df82d98d5843a0460316a50b0941c1cb782ccd4c1186f2a33a4c757e24196` |
| `crates/seacad-dxf-core/tests/entity_insert_session_tests.rs` | 553 | `a42a6bd6495c408e0179c8a86d67f78399d858ff59dc8c53d9b398db567b815c` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,072 | `8001c6289975b55714d3f526621d9c4b10022b48287a54d4c86ec1a8b4eb21ac` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,497 | `f9fe52213f5ba0cdd0c97045365f66e4b2504cbe7974eef97a7a415a97aecb02` |
| `docs/SUPPORT_MATRIX.md` | 2,153 | `e29ace398d2baf4d1c081eade114a52dca16a51a8f32633df3fe89065499f632` |
