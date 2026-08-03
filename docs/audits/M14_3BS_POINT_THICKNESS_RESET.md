# M14.3bs POINT Thickness Reset

Retrieved: 2026-08-03

## Scope

M14.3bs adds reset-to-default behavior to the existing typed POINT thickness
patch identity. It preserves the M14.3bq replacement and M14.3br insertion
paths. It does not add extrusion or angle updates, mixed insert/update
sessions, clone, delete, rendering, or `Complete` support.

## Normative evidence

- Autodesk [POINT (DXF)](https://help.autodesk.com/cloudhelp/2016/ENU/AutoCAD-DXF/files/GUID-9C6AD32D-769D-4213-85A4-CA9CCB5C5317.htm)
  defines group `39` as optional POINT thickness with a documented zero default.
- Autodesk [DXF Group Codes in Numerical Order](https://help.autodesk.com/view/OARX/2025/ENU/?guid=GUID-3F0380A5-1C15-464D-BC66-2C5F094BCFB9)
  identifies group `39` as the fixed entity thickness double when nonzero.

The clean-room boundary remains recorded in
`docs/audits/M14_3BK_POINT_DRAFT_RECORD.md`. No legacy or external parser code,
fixture, data, dependency, or generated artifact is copied, translated,
vendored, linked, or used at runtime.

## Contract

- `DxfPointPatch::ResetThickness` uses `DxfPointPatchKind::Thickness`, so one
  queued set or reset rejects a second logical thickness request.
- One unique explicit group `39` produces one exact-span deletion transaction.
  Multiple occurrences remain a typed failure and are never selected.
- Documented absence returns a POINT receipt with `AlreadyImplicit`, queues no
  transaction, and does not reserve the logical patch identity; a later set in
  the same session is therefore admissible.
- POINT receipts expose `Inserted`, `Replaced`, `Reset`, or `AlreadyImplicit`
  without retaining caller payload bytes.
- Post-image verification requires binary64 positive zero and the `Defaulted`
  semantic state on the same raw-record ordinal. An explicit zero is not a
  successful reset. The inverse restores the deleted bytes exactly.

## Verification boundary

Paired ASCII and Binary fixtures cover AC1009 through AC1032. Focused cases
cover exact deletion, already-implicit no-op, duplicate thickness, conflicting
set/reset admission, cancellation, explicit-zero tampering, strict reparse,
and exact inverse restoration.

## Nonclaims

This checkpoint does not update POINT extrusion or UCS X-axis angle, mix
entity insertion with raw-ordinal updates, clone or delete an entity graph,
render a point, or advance POINT to `Complete`.

## Verification

The POINT edit suite passed 14/14 focused tests. The full workspace passed
924/924 tests, including all 17 locale tests. Generated-schema and release-
evidence checks, `cargo deny --locked check`, formatting, workspace Clippy with
warnings denied, the production forbidden-construct scan, and
`git diff --check` all passed. Production changes are 166 insertions and 14
deletions; focused test changes are 235 insertions and 4 deletions. No manifest,
lockfile, dependency, fixture, generated schema, locale source, or external
corpus changed. This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 317 | `6bc41f130a0835cb62fa414dff9c865b1eccfffd356b12e1a5fa7f79ec6d4c93` |
| `crates/seacad-dxf-core/src/entity_edit_session.rs` | 1,460 | `5a4300b513efe295ebd8891394eb1cce067d0f60eb2e05d53661c208fdb0d589` |
| `crates/seacad-dxf-core/src/entity_edit_verification.rs` | 1,026 | `95288bee64797cf1c5abdbb077cc272cf353211fd324e940c284c1041adba9be` |
| `crates/seacad-dxf-core/src/point_edit.rs` | 417 | `d7acd8feefbc69db797591c0f333c5bcfac2b76ee9ba2065c33303d62b110617` |
| `crates/seacad-dxf-core/tests/point_edit_session_tests.rs` | 1,131 | `3743a8b4100d4181815555fa43d6e5b0a4302d3630c91599f45a4ed6ddf951cd` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,123 | `f8ff42b39526130ee5e269f904e8729f20af5cb3c6180e8bf442828606796b4b` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,548 | `4932896aaf041623d2510fc648b436fe9a8059b9bb5e606772eece2cb5152c45` |
| `docs/SUPPORT_MATRIX.md` | 2,205 | `c973c246a515cbad0f6e6b958b039423960403d9ca9278f01817bbc9726e1f95` |
