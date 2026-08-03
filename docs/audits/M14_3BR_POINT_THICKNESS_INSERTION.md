# M14.3br POINT Thickness Insertion

Retrieved: 2026-08-03

## Scope

M14.3br extends the existing typed POINT thickness patch to documented
defaulted absence. It preserves the M14.3bq replacement path and adds one safe
zero-width insertion path. It does not add reset, extrusion or angle updates,
mixed insert/update sessions, clone, delete, rendering, or `Complete` support.

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

- `DxfPointPatch::SetThickness` retains one patch kind and receipt for both
  replacement and insertion.
- One unique explicit group `39` is replaced exactly as in M14.3bq. Multiple
  occurrences remain a typed failure and are never selected.
- Absent thickness is eligible for insertion only when location groups `10`,
  `20`, and `30` are each unique. The last source occurrence anchors one
  zero-width patch, so out-of-order location evidence remains source-bound.
- The shared insertion-byte path preserves the predecessor's LF, CRLF, or CR
  ending; Binary uses the exact dialect group framing.
- Non-finite or otherwise unencodable values fail before a transaction enters
  the session. A second thickness patch retains the existing duplicate guard.
- Post-image verification requires the exact requested binary64 value and the
  `Explicit` semantic state on the same raw-record ordinal. The inverse removes
  the inserted bytes and restores the original source identity.

## Verification boundary

Paired ASCII and Binary fixtures cover AC1009 through AC1032. Focused cases
cover replacement, insertion, CRLF preservation, missing or duplicate location
anchors, duplicate thickness, wrong family, non-finite input, duplicate patch
admission, composition, cancellation, tampering, strict reparse, and exact
inverse restoration.

## Nonclaims

This checkpoint does not reset thickness to implicit zero, update extrusion or
UCS X-axis angle, mix entity insertion with raw-ordinal updates, clone or delete
an entity graph, render a point, or advance POINT to `Complete`.

## Verification

The POINT edit suite and the two shared insertion suites passed 18/18 focused
tests. The full workspace passed 920/920 tests, including all 17 locale tests.
Generated-schema and release-evidence checks, `cargo deny --locked check`,
formatting, workspace Clippy with warnings denied, the production forbidden-
construct scan, and `git diff --check` all passed. Production changes are 101
insertions and 36 deletions; focused test changes are 180 insertions and 46
deletions. No manifest, lockfile, dependency, fixture, generated schema, locale
source, or external corpus changed. This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `AGENTS.md` | 77 | `c07f3139fd4c37fe112734e34d8117ff8026533b34545f45099281b8859b512c` |
| `README.md` | 313 | `7416bfa1712ec2a106aff608ffaef524f5f548d3083aa525e6b50cb4c2e48e4c` |
| `crates/seacad-dxf-core/src/entity_field_insertion.rs` | 276 | `82160b281d71ab31847f5304bbfe846be8f3029f4fc8acacaaa3fb2d18de7b36` |
| `crates/seacad-dxf-core/src/point_edit.rs` | 342 | `d38154710aa63896926aca311bc40f0ace680cf20ea2fe9781f1f10bf28a3016` |
| `crates/seacad-dxf-core/tests/point_edit_session_tests.rs` | 900 | `700524fee8bbc009589551e6d693c14448485187992134fcef4e0aed8e791552` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,111 | `f904cbd9a5596497e850084eb193a693692958f96231ac26710b515225696d3a` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,535 | `950dcf32f6e1d6bc7f8eeaf82fd9c849634a525d5c429f78f61a99d6ed0ac9ce` |
| `docs/SUPPORT_MATRIX.md` | 2,193 | `451d43940734469bdc900edfdaa28b9a40a6c8656a74c44fcb0776b31aa8bde4` |
