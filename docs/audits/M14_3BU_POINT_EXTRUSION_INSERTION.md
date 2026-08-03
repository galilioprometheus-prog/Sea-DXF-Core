# M14.3bu POINT Extrusion Insertion

Retrieved: 2026-08-03

## Scope

M14.3bu extends the typed POINT extrusion patch to a completely absent tuple
using the documented `(0,0,1)` default. It preserves the M14.3bt replacement
path. It does not mix partial explicit/default extrusion components, reset
extrusion, update the UCS X-axis angle, mix insert/update sessions, clone,
delete, render, or advance POINT to `Complete`.

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

- `DxfPointPatch::SetExtrusion` retains one extrusion patch identity for
  replacement and insertion and reports the physical disposition in its
  payload-free receipt.
- Three unique explicit components retain M14.3bt exact-span replacement. When
  all three components are absent, the requested nonzero tuple is encoded in
  canonical `210/220/230` order and admitted as one zero-width patch.
- Partial absent/unique tuples remain typed missing-component failures. Multiple
  components remain typed duplicate failures and are never selected.
- A unique thickness group `39` is the preferred predecessor. When thickness is
  absent, the last source occurrence among three unique location components is
  used. Duplicate thickness or missing/duplicate fallback location evidence
  cannot anchor insertion and queues nothing.
- The shared framing helper is applied to each encoded group before
  concatenation, preserving LF, CRLF, CR, or exact Binary group bytes.
- Post-image verification requires the exact requested tuple and `Explicit`
  state on all three components. The inverse removes the entire inserted
  sequence and restores the original source bytes exactly.

## Verification boundary

Paired ASCII and Binary fixtures cover AC1009 through AC1032. Focused cases
cover defaulted insertion, replacement regression, CRLF preservation,
thickness and fallback-location anchors, duplicate thickness, missing and
duplicate location evidence, strict reparse, semantic verification, and exact
inverse restoration. The preceding M14.3bt cases continue to cover partial
tuples, zero direction, non-finite input, wrong family, duplicate patch
admission, composition, cancellation, and tampering.

## Nonclaims

This checkpoint does not mix insertion and replacement for a partial POINT
extrusion tuple, reset extrusion to its default, update the UCS X-axis angle,
mix entity insertion with raw-ordinal updates, clone or delete an entity graph,
render a point, or advance POINT to `Complete`.

## Verification

The POINT edit suite passed 20/20 focused tests. The full workspace passed
930/930 tests, including all 17 locale tests. Generated-schema and release-
evidence checks, `cargo deny --locked check`, formatting, workspace Clippy with
warnings denied, the production forbidden-construct scan, and
`git diff --check` all passed. Production changes are 149 insertions and 32
deletions; focused test changes are 202 insertions and no deletions. No manifest,
lockfile, dependency, committed fixture, generated schema, locale source, or
external corpus changed. This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 328 | `9e800ad15debaed034dd07a91940cab212d72e3392e63b009daf91c896cc2493` |
| `crates/seacad-dxf-core/src/entity_edit_session.rs` | 1,500 | `27fa7e8c556cd88a3a38a8411b254c13fcd76c73b1274c248339bec712a81aed` |
| `crates/seacad-dxf-core/src/point_edit.rs` | 624 | `6dc71d208ef858c527add7ad357ba76cca80050211c486e5ebc5acf6df16c632` |
| `crates/seacad-dxf-core/tests/point_edit_session_tests.rs` | 1,634 | `599bdb6285eb4c746c548229378074c3cf73393ee07eb5c243b2297bb54c4953` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,149 | `c96b0379e88b9a95bfd537dbaa0cb2252a2bd1f0fb66c9f441074acdd388de6c` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,575 | `f95d732035c11fa10fe9e60b4e513fb2aa1c4ed6de61596d8d884938a243d3e0` |
| `docs/SUPPORT_MATRIX.md` | 2,231 | `ec41b499a74c031195b18dd0e9db0acb0256f0e46f01cd5f6b6afc88cface855` |
