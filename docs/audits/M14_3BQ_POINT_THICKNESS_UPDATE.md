# M14.3bq POINT Thickness Update

Retrieved: 2026-08-03

## Scope

M14.3bq adds atomic replacement of one existing explicit POINT thickness
through the unified entity edit session. It does not widen the claim to absent
thickness insertion, reset, extrusion or angle updates, clone, delete, display,
or `Complete` support.

## Normative evidence

- Autodesk [POINT (DXF)](https://help.autodesk.com/cloudhelp/2016/ENU/AutoCAD-DXF/files/GUID-9C6AD32D-769D-4213-85A4-CA9CCB5C5317.htm)
  defines group `39` as optional POINT thickness with a documented zero default.
- Autodesk [DXF Group Codes in Numerical Order](https://help.autodesk.com/view/OARX/2025/ENU/?guid=GUID-3F0380A5-1C15-464D-BC66-2C5F094BCFB9)
  identifies group `39` as the fixed entity thickness double when nonzero.

The clean-room boundary remains recorded in
`docs/audits/M14_3BK_POINT_DRAFT_RECORD.md`. No external or legacy code,
fixture, data, dependency, or generated artifact was copied, translated,
vendored, linked, or used at runtime for this checkpoint.

## Contract

- `DxfEntityPatch::Point(DxfPointPatch::SetThickness)` targets one source-bound
  canonical POINT and retains a payload-free patch kind and receipt.
- Admission requires one unique explicit group `39`. Missing and duplicate
  occurrences fail typed; the documented default is not converted into an
  invented insertion.
- The requested value must be a finite `DxfDouble` encodable in the source
  ASCII/Binary dialect. No partial plan enters the session.
- Location and thickness have distinct duplicate identities and may compose
  for one POINT. A second thickness patch is rejected and insertion remains
  excluded from an update batch.
- Post-image verification requires both exact thickness bits and the
  `Explicit` semantic state for the same raw-record ordinal.
- The writer changes only the selected group `39` raw span. Unknown groups and
  all unrelated bytes remain exact, and the inverse restores byte-identical
  source.

## Verification boundary

Paired ASCII and Binary fixtures cover AC1009 through AC1032. Focused cases
cover wrong-family admission, missing and duplicate thickness, duplicate
patches, non-finite input, location composition, cancellation, verifier
tampering, strict reparse, and byte-identical inverse restoration.

## Nonclaims

This checkpoint does not insert an absent thickness, reset thickness to its
implicit zero default, update extrusion or UCS X-axis angle, mix record
insertion with raw-ordinal updates, clone/delete a closed set, render a point,
or advance POINT to `Complete`.

## Verification

The five focused POINT/edit/verification/write/insert suites passed 25/25
tests. The full workspace passed 918/918 tests, including all 17 locale tests.
Generated-schema and release-evidence checks, `cargo deny --locked check`,
formatting, workspace Clippy with warnings denied, the production forbidden-
construct scan, and `git diff --check` all passed. Production changes are 253
insertions and 72 deletions; the focused test changes are 272 insertions and 5
deletions. No manifest, lockfile, dependency, fixture, generated schema, or
locale source changed. This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 313 | `42476916c287815a95a49ad3e9cdb41e424b0eb8d21aab8fe40454e5953da0ab` |
| `crates/seacad-dxf-core/src/point_edit.rs` | 280 | `4253758993d90e3500b1f9707b6e4ecd927f338b6e0b157b2ab7963ce57b616d` |
| `crates/seacad-dxf-core/src/entity_edit_session.rs` | 1,403 | `0aa949ec45cf1fd95fae19cbcdaec2531a1dfded0f11bcc02569c59079e2d971` |
| `crates/seacad-dxf-core/src/entity_edit_verification.rs` | 1,006 | `46d2ed22779701a265e8ddb95540f03068af2a5a91486d673211fc860f1a0e0e` |
| `crates/seacad-dxf-core/tests/point_edit_session_tests.rs` | 766 | `65f38867329e43b48cb5261f6b86634df265fda329a12258d72f0fcb4f2ce8d5` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,098 | `9e7a7ca1190acf4588a5b19ac045b72807a3f304de4e90fb9cddc46b6428be8d` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,521 | `95cd0a19dae9d2893d89a07ac623b436cf46b1837dcb846af07972108cb1127c` |
| `docs/SUPPORT_MATRIX.md` | 2,181 | `e92b75b19c33f91153740063595f7699d4bfd0c2d4c5cd9ae4cf4eb3fdaa3452` |
