# M14.3bp POINT Location Update

Retrieved: 2026-08-02

## Scope

M14.3bp adds atomic update of the required POINT WCS location through the
unified entity edit session. It does not widen the claim to other POINT fields,
reset, clone, delete, display, or `Complete` support.

## Normative evidence

- Autodesk [POINT (DXF)](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-9C6AD32D-769D-4213-85A4-CA9CCB5C5317.htm)
  defines the required WCS location as double groups `10`, `20`, and `30`.
- Autodesk [Common Group Codes for Entities](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm)
  states that consumers must not depend on group-code presentation order.
  Admission therefore uses the existing role/card evidence and never positional
  field order.

The clean-room boundary remains recorded in
`docs/audits/M14_3BK_POINT_DRAFT_RECORD.md`. No external or legacy code,
fixture, data, dependency, or generated artifact was copied, translated,
vendored, linked, or used at runtime for this checkpoint.

## Contract

- `DxfEntityPatch::Point(DxfPointPatch::SetLocation)` targets one source-bound
  canonical POINT key and retains a payload-free patch kind and receipt.
- Admission requires one unique component for each location role. Missing and
  duplicate members fail typed; no occurrence is selected.
- Each requested component must be a finite `DxfDouble` encodable in the
  source ASCII/Binary dialect. The three replacements form one logical edit
  and no partial plan enters the session.
- A second location patch for the same key is rejected. A POINT-family update
  may compose with independent common-property updates; insertion remains
  excluded from an update batch.
- Post-image verification projects typed POINT geometry for the same raw-record
  ordinal and requires exact component bits before releasing the inverse.
- The writer changes only the three selected raw spans. Unknown groups and all
  unrelated bytes remain exact, and the inverse restores byte-identical source.

## Verification boundary

Paired ASCII and Binary fixtures cover AC1009 through AC1032 with deliberately
out-of-order location groups. Focused cases cover wrong-family admission,
missing and duplicate components, duplicate patches, non-finite input,
common-field composition, update/insert exclusion, cancellation, verifier
tampering, strict reparse, and byte-identical inverse restoration.

## Nonclaims

This checkpoint does not add location insertion/defaulting, update thickness,
extrusion, or UCS X-axis angle, reset a POINT family field, mix record insertion
with raw-ordinal updates, clone/delete a closed set, render a point, or advance
POINT to `Complete`.

## Verification

The focused POINT/edit/verification/insert suites passed 16/16 tests. The full
workspace passed 914/914 tests, including all 17 locale tests. Generated-schema
and release-evidence checks, `cargo deny --locked check`, formatting, workspace
Clippy with warnings denied, the production forbidden-macro scan, and
`git diff --check` all passed. Production changes are 394 insertions and 20
deletions; the focused test adds 499 lines. No manifest, lockfile, dependency,
fixture, generated schema, or locale source changed. This audit intentionally
omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 303 | `499fbc702d1848faa4d085e6de2aad5335abc997e1e974b9d5c1fc21a7baf43f` |
| `crates/seacad-dxf-core/src/point_edit.rs` | 180 | `3c450e23ed9041a6519e8e42d5ee60ab885db7c0527c5c547315e6b0a4c08c3b` |
| `crates/seacad-dxf-core/src/entity_edit_session.rs` | 1,366 | `937ae9a8c3207dfa09f0e8a1818fb69cabda3e34302dabdf28bd3afbf677ea19` |
| `crates/seacad-dxf-core/src/entity_edit_verification.rs` | 962 | `bbe6f25ef0ac391b7a14cad97e6163d9743f4e75aee0ae7008a18ef86c787c51` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,066 | `9e975f4cd504224e3594d8ba1c522790040a6e72b48f16318857a8ec48a93333` |
| `crates/seacad-dxf-core/tests/point_edit_session_tests.rs` | 499 | `7040887e09e8a9e9b7c134170c32147e8552fe1880d1c54d14bba8859a80f0b9` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,085 | `def56a41fca5839aa7c42d2f1e073c6a49241a8625993183b7f94e277f8170cd` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,509 | `5ba29b06921013f4ab4b899f80125d50dd0ca978d426377cb737db3a8dcd8b73` |
| `docs/SUPPORT_MATRIX.md` | 2,167 | `39b6e663e48e01f4be5d9a847c32cbc2b36d691b00e4285d926a7ddb0b484cd8` |
