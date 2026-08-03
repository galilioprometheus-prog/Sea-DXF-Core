# M14.3bx POINT UCS X-Axis Angle Set

Retrieved: 2026-08-03

## Scope

M14.3bx adds typed replacement and insertion of optional POINT UCS X-axis angle
group `50` through the unified edit session. It does not reset the angle, mix
entity insertion with source-bound updates, clone, delete, render, or advance
POINT to `Complete`.

## Normative evidence

- Autodesk [POINT (DXF)](https://help.autodesk.com/cloudhelp/2016/ENU/AutoCAD-DXF/files/GUID-9C6AD32D-769D-4213-85A4-CA9CCB5C5317.htm)
  defines optional group `50` as the angle of the X axis for the UCS in effect
  when the point was drawn and gives an omitted default of zero.
- Autodesk [DXF Group Codes in Numerical Order](https://help.autodesk.com/view/OARX/2025/ENU/?guid=GUID-3F0380A5-1C15-464D-BC66-2C5F094BCFB9)
  identifies groups `50` through `58` as double-precision angles in degrees.

The clean-room boundary remains recorded in
`docs/audits/M14_3BK_POINT_DRAFT_RECORD.md`. No legacy or external parser code,
fixture, data, dependency, or generated artifact is copied, translated,
vendored, linked, or used at runtime.

## Contract

- `DxfPointPatch::SetUcsXAxisAngle` owns the distinct `UcsXAxisAngle` logical
  patch identity and reports `Replaced` or `Inserted` in its payload-free
  receipt.
- One unique explicit group `50` is encoded and replaced at its exact source
  span. Multiple angle groups fail typed and no occurrence is selected.
- When group `50` is absent, the last unique explicit extrusion component is
  the preferred insertion predecessor. Every one of the eight extrusion masks
  therefore has a deterministic canonical angle anchor.
- With a fully absent extrusion tuple, insertion falls back to one unique
  thickness group and then to the last source occurrence among three unique
  required location components. Duplicate or missing fallback evidence fails
  typed and queues nothing.
- The existing dialect encoder rejects non-finite input. Replacement and
  insertion preserve LF, CRLF, CR, or exact Binary framing.
- Post-image verification requires the exact requested binary64 value and
  `Explicit` semantic state on the same source record. The inverse restores
  every original byte exactly.

## Verification boundary

Explicit replacement and all eight extrusion-mask insertion states are tested
for ASCII and Binary AC1009 through AC1032. Focused cases also cover CRLF,
duplicate angle, duplicate extrusion, duplicate thickness, missing location,
wrong family, non-finite encoding, duplicate patch admission, four-field POINT
composition, cancellation, tampering, strict reparse, semantic verification,
and byte-identical inverse restoration.

## Nonclaims

This checkpoint does not reset group `50` to its implicit zero default, mix
entity insertion with source-bound updates, clone or delete an entity graph,
render a point, or advance POINT to `Complete`.

## Verification

The POINT edit suite passed 31/31 focused tests. The full workspace passed
941/941 tests, including all 17 locale tests. Generated-schema and release-
evidence checks, `cargo deny --locked check`, formatting, workspace Clippy with
warnings denied, the production forbidden-construct scan, and
`git diff --check` all passed. Production changes are 255 insertions and two
deletions; focused test changes are 425 insertions and six deletions. No
manifest, lockfile, dependency, committed fixture, generated schema, locale
source, or external corpus changed. This audit intentionally omits its own
hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 334 | `9a40fecf2d71f25c744b705ec8b372150a675de38c5ae95f09eceed9daf054f8` |
| `crates/seacad-dxf-core/src/entity_edit_session.rs` | 1,580 | `9d9de5d3741270e37cf966a1785d999aad0e035ec4e097b5012661c301e73921` |
| `crates/seacad-dxf-core/src/entity_edit_verification.rs` | 1,143 | `107fb722031559f6ebedebe64e084c044957018df266402518ff68416390ab20` |
| `crates/seacad-dxf-core/src/point_edit.rs` | 1,022 | `c9da631c0fdda830206feb1ca233b52b9fff7b24819738e2b8d7320d4e8963b4` |
| `crates/seacad-dxf-core/tests/point_edit_session_tests.rs` | 2,452 | `2cea4733a461674bdbb58ef51fe58233610897b2028e8357fb30ef64a0020392` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,184 | `bb6d5a54e9d70a55d2cfc427d7f6916809b44148cca56913f1741345cd71aa2e` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,613 | `6c592fb42ad8e07e5dd22f66cf5437de8305cdf596bdff9c67239b3389db31e5` |
| `docs/SUPPORT_MATRIX.md` | 2,266 | `48f643175aa8fe4b52469c3136fc2fd005c1642e597eb33d6af303a1ad5a497c` |
