# M14.3bw POINT Extrusion Reset

Retrieved: 2026-08-03

## Scope

M14.3bw adds reset-to-default behavior to the typed POINT extrusion patch. It
deletes every unique explicit component from complete and partial tuples and
preserves the existing set paths. It does not update the UCS X-axis angle, mix
entity insertion with source-bound updates, clone, delete, render, or advance
POINT to `Complete`.

## Normative evidence

- Autodesk [POINT (DXF)](https://help.autodesk.com/cloudhelp/2016/ENU/AutoCAD-DXF/files/GUID-9C6AD32D-769D-4213-85A4-CA9CCB5C5317.htm)
  defines groups `210`, `220`, and `230` as the extrusion direction and gives
  the omitted default `(0, 0, 1)`.
- Autodesk [DXF Group Codes in Numerical Order](https://help.autodesk.com/view/OARX/2025/ENU/?guid=GUID-3F0380A5-1C15-464D-BC66-2C5F094BCFB9)
  identifies `210`, `220`, and `230` as extrusion-direction X, Y, and Z doubles.

The clean-room boundary remains recorded in
`docs/audits/M14_3BK_POINT_DRAFT_RECORD.md`. No legacy or external parser code,
fixture, data, dependency, or generated artifact is copied, translated,
vendored, linked, or used at runtime.

## Contract

- `DxfPointPatch::ResetExtrusion` shares the `Extrusion` patch identity with
  `SetExtrusion`, so a queued set or reset rejects a second extrusion request.
- Every unique present component becomes one exact deletion patch. One logical
  reset therefore contains one to three physical patches according to the
  source mask.
- A fully absent tuple returns `AlreadyImplicit`, consumes no session edit slot,
  reserves no patch identity, and leaves a subsequent set admissible.
- Any duplicate component fails typed before a transaction is admitted. Reset
  never selects one occurrence from ambiguous evidence.
- Strict reparse must publish the documented `(0,0,1)` value with all three
  semantic component states `Defaulted`. An explicit encoding of numerically
  identical values is a verification failure.
- Verification remains bound to the original source record. The inverse
  restores every deleted byte exactly for complete and partial tuples.

## Verification boundary

All seven nonempty explicit-component masks are exercised for ASCII and Binary
AC1009 through AC1032. Focused cases also cover the fully absent no-op, later
set admission, set/reset conflict, duplicate evidence, wrong family,
cancellation, explicit-default tampering, semantic verification, exact physical
patch counts, and byte-identical inverse restoration.

## Nonclaims

This checkpoint does not update POINT angle group `50`, mix entity insertion
with source-bound updates, clone or delete an entity graph, render a point, or
advance POINT to `Complete`.

## Verification

The POINT edit suite passed 26/26 focused tests. The full workspace passed
936/936 tests, including all 17 locale tests. Generated-schema and release-
evidence checks, `cargo deny --locked check`, formatting, workspace Clippy with
warnings denied, the production forbidden-construct scan, and
`git diff --check` all passed. Production changes are 153 insertions and 12
deletions; focused test changes are 261 insertions and no deletions. No
manifest, lockfile, dependency, committed fixture, generated schema, locale
source, or external corpus changed. This audit intentionally omits its own
hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 330 | `9d03f2fb48984f1ea2c9466e04957fa3aaabe3cdea2d120b46be176277732cb6` |
| `crates/seacad-dxf-core/src/entity_edit_session.rs` | 1,540 | `7cebaf52dab6d8f2cc8d39e57162e8ecd54dbe3ad438a9865362ea79b29d115a` |
| `crates/seacad-dxf-core/src/entity_edit_verification.rs` | 1,099 | `bcf857a2f61cb85582283e99952376f9138e5467af1424a4834df52f902bd70b` |
| `crates/seacad-dxf-core/src/point_edit.rs` | 853 | `22621c84d0f63cbc267e3e542a8ee395918f65a9c879cb4ced1332221cbd5e98` |
| `crates/seacad-dxf-core/tests/point_edit_session_tests.rs` | 2,033 | `bb4a8edc0b08bf5be145c2fc092a76fabf2cd55e7a2b8158f3afc07c832d7b27` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,172 | `bda08fbf2585cc68aba47746518b218877a9b977523a1ed1b3879a681f8ebdef` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,600 | `f8aec38aba236370f3e5969e90b66d48b8c6394e2d3cb8176265dded4ea16b68` |
| `docs/SUPPORT_MATRIX.md` | 2,254 | `773f34377c2a2fa3104a307b3024533d4d37929fa5f57c5860eab89a839053d5` |
