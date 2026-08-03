# M14.3bv POINT Extrusion Partial Completion

Retrieved: 2026-08-03

## Scope

M14.3bv extends the typed POINT extrusion patch to every partial absent/unique
component tuple. It preserves the complete replacement and fully absent
insertion paths. It does not reset extrusion, update the UCS X-axis angle, mix
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

- `DxfPointPatch::SetExtrusion` keeps one logical patch identity across
  replacement, insertion, and partial completion.
- Every unique explicit component is replaced at its exact source span. Every
  consecutive missing run is encoded in canonical `210/220/230` order and
  inserted at its corresponding source-order gap.
- Partial completion reports `Composite` and records the actual physical patch
  count. One outer explicit component needs one replacement plus one two-group
  insertion; all other partial masks need three physical patches.
- Partial explicit components must already appear in canonical source order.
  Reordered or duplicate evidence fails typed and queues nothing; the planner
  does not move source groups or infer intent.
- Replacement and insertion preserve the local LF, CRLF, CR, or exact Binary
  framing. Strict reparse must publish the requested tuple with all three
  semantic states `Explicit`.
- Verification remains bound to the original source record and requested exact
  binary64 values. The inverse restores every source byte exactly.

## Verification boundary

All six partial component masks are exercised for ASCII and Binary AC1009
through AC1032. Focused cases also cover CRLF preservation, source-reordered
partial evidence, duplicate evidence, exact patch and insertion counts,
composition regression, strict reparse, semantic verification, cancellation,
tampering, and exact inverse restoration. Existing tests retain complete tuple
replacement and fully absent insertion coverage.

## Nonclaims

This checkpoint does not reset extrusion to its implicit default, update POINT
angle group `50`, mix entity insertion with source-bound updates, clone or
delete an entity graph, render a point, or advance POINT to `Complete`.

## Verification

The POINT edit suite passed 22/22 focused tests. The full workspace passed
932/932 tests, including all 17 locale tests. Generated-schema and release-
evidence checks, `cargo deny --locked check`, formatting, workspace Clippy with
warnings denied, the production forbidden-construct scan, and
`git diff --check` all passed. Production changes are 213 insertions and 62
deletions; focused test changes are 194 insertions and 56 deletions. No
manifest, lockfile, dependency, committed fixture, generated schema, locale
source, or external corpus changed. This audit intentionally omits its own
hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 327 | `f161609c76c92d70b68e913a1c2aaf2429cc9e30f7284bc3a1f2825bf1d4cf1f` |
| `crates/seacad-dxf-core/src/entity_edit_session.rs` | 1,500 | `1cf8c7e6411fd2e79d841df0abbd97b37c0185b6be7bcf0fc8732bce8362ec66` |
| `crates/seacad-dxf-core/src/point_edit.rs` | 775 | `3b0d950f62592a365f670804592623897afc2ec2cc0b8482bc41595ff5f5fe41` |
| `crates/seacad-dxf-core/tests/point_edit_session_tests.rs` | 1,772 | `4310be1ccac78d33b81cc4f9d2e1520b40861e9fbe07b03cb0a5642e4f03113c` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,161 | `7501ccf8e7501476f4643569803252044a0406a1543c90a9e9bf26e8425fdc1f` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,588 | `8eb3993ee615abaacebe3b21da7a632174ff5edd0f8dd1f61cf1c1c491c673b2` |
| `docs/SUPPORT_MATRIX.md` | 2,243 | `2de7e4ff454496d21791ce89b13d0e1175cabd5505bd25866f097a1551c98099` |
