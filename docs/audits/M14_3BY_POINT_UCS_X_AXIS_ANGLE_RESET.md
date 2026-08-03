# M14.3by POINT UCS X-Axis Angle Reset

Retrieved: 2026-08-03

## Scope

M14.3by adds typed reset of optional POINT UCS X-axis angle group `50`
through the unified edit session. It does not mix entity insertion with
source-bound updates, clone, delete, render, or advance POINT to `Complete`.

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

- `DxfPointPatch::ResetUcsXAxisAngle` shares the existing `UcsXAxisAngle`
  logical patch identity with set/update and reports `Reset` when one deletion
  is queued.
- One unique explicit group `50` is deleted at its exact full source span.
  Multiple angle groups fail typed and no occurrence is selected.
- An absent group `50` reports `AlreadyImplicit`, queues no edit, consumes no
  edit-limit slot, and does not reserve the patch identity against a later set.
- A queued set or reset rejects a second angle operation through the shared
  duplicate-patch contract.
- Deletion preserves the surrounding LF, CRLF, CR, or exact Binary framing.
- Post-image verification requires both the documented zero value and
  `Defaulted` semantic state on the same source record. An explicit zero does
  not satisfy reset, and the inverse restores every original byte exactly.

## Verification boundary

Unique explicit reset is tested for ASCII and Binary AC1009 through AC1032.
Focused cases also cover absent no-op and later set admission, CRLF, duplicate
angle evidence, wrong family, conflicting set/reset requests, cancellation,
explicit-zero tampering, strict reparse, semantic verification, and
byte-identical inverse restoration.

## Nonclaims

This checkpoint does not mix entity insertion with source-bound updates, clone
or delete an entity graph, render a point, or advance POINT to `Complete`.

## Verification

The POINT edit suite passed 35/35 focused tests. The full workspace passed
945/945 tests, including all 17 locale tests. Generated-schema and release-
evidence checks, `cargo deny --locked check`, formatting, workspace Clippy with
warnings denied, the production forbidden-construct scan, and
`git diff --check` all passed. Production changes are 131 insertions and nine
deletions; focused test changes are 300 insertions. No manifest, lockfile,
dependency, committed fixture, generated schema, locale source, or external
corpus changed. This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 338 | `12456d78b7acc0311087afd8b079a5b8d08396d089f1658aff7964c2b1141a1c` |
| `crates/seacad-dxf-core/src/entity_edit_session.rs` | 1,620 | `788b6410862d499a84695869c07a68902e876101f4e9abee660f8ad26ad57b91` |
| `crates/seacad-dxf-core/src/entity_edit_verification.rs` | 1,163 | `bd630af4b542261c6fa2fdc5b7b27f27098ba402e8f7b27d3327d7a52048d60f` |
| `crates/seacad-dxf-core/src/point_edit.rs` | 1,084 | `bcef8c022f2c17e5bf672a06692a9c28787ef7e56fbc24194c0ab44a33c46e51` |
| `crates/seacad-dxf-core/tests/point_edit_session_tests.rs` | 2,752 | `370cf98b4a30a09050088344fcf081eb0e16287a151076f7e1d8ba97a6fa22f1` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,195 | `e261bc0a136850a796f71d7e2a1c1fd577cd6fb486b74238a7cb9747001b7c1a` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,624 | `939cb791ca0b4187829ccf11a91b0a7a5e7705ef7ab9d134290889992dab37b8` |
| `docs/SUPPORT_MATRIX.md` | 2,277 | `0b30798d51c863f77042bedaa7388c9bc4ee8e99a5f9d25492bab6f9db35d412` |
