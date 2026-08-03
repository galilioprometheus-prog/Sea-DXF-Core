# M14.3bt POINT Extrusion Replacement

Retrieved: 2026-08-03

## Scope

M14.3bt adds atomic existing-record replacement for one complete explicit
POINT extrusion direction tuple. It does not insert or reset defaulted
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

- `DxfPointPatch::SetExtrusion` has a distinct
  `DxfPointPatchKind::Extrusion` identity and retains no caller payload in its
  receipt.
- Groups `210`, `220`, and `230` must each be present exactly once. One absent
  or multiple component rejects the entire request without selecting evidence.
- The requested direction must be nonzero. Every component passes the original
  ASCII/Binary dialect encoder, so non-finite input fails typed before the plan
  enters the session.
- Three exact source-span replacements count as one logical edit with a
  `Replaced` disposition. Location, thickness, and extrusion may compose under
  their separate identities.
- Post-image verification resolves the same raw-record ordinal, requires the
  exact requested tuple and `Explicit` semantic state on all three components,
  and releases an inverse that restores the source bytes exactly.

## Verification boundary

Paired ASCII and Binary fixtures cover AC1009 through AC1032. Focused cases
cover replacement, missing and duplicate components, zero direction,
non-finite input, wrong family, duplicate admission, composition with location
and thickness, cancellation, semantic tampering, strict reparse, and exact
inverse restoration.

## Nonclaims

This checkpoint does not insert or reset defaulted POINT extrusion, update the
UCS X-axis angle, mix entity insertion with raw-ordinal updates, clone or delete
an entity graph, render a point, or advance POINT to `Complete`.

## Verification

The POINT edit suite passed 18/18 focused tests. The full workspace passed
928/928 tests, including all 17 locale tests. Generated-schema and release-
evidence checks, `cargo deny --locked check`, formatting, workspace Clippy with
warnings denied, the production forbidden-construct scan, and
`git diff --check` all passed. Production changes are 182 insertions and 2
deletions; focused test changes are 305 insertions and 4 deletions. No manifest,
lockfile, dependency, committed fixture, generated schema, locale source, or
external corpus changed. This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 325 | `e9b069ae460703fdbc0c6842ac4bd7ecf84d8028d4f3794fc477039d69af8979` |
| `crates/seacad-dxf-core/src/entity_edit_session.rs` | 1,491 | `5c72e4477ac4a51447fed5a33fd6e1dc7ecf392d100aec0c81f6c876bcecba83` |
| `crates/seacad-dxf-core/src/entity_edit_verification.rs` | 1,076 | `7a56c1c7ff3e705b22cd3ff3f1a572fbfc9a02556ea928c3e2a028ab2e0fd266` |
| `crates/seacad-dxf-core/src/point_edit.rs` | 516 | `9eb941263750243ded0a3bf4afc291d3f5a1c208736ddcf736dc352562803206` |
| `crates/seacad-dxf-core/tests/point_edit_session_tests.rs` | 1,432 | `f037f3405979af2c2655b2f51de62d2afb386b77750c5fcf0eb93162a894821a` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,136 | `085bd2db288c57d68fdf0ec0028f4527cdf5a0d13db6b92706bdd5ccdc5e3b32` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,562 | `d0a046901c808eec981a1cfaa68dd08543b1f2fef5261eb28dc334a7c0f8f6d4` |
| `docs/SUPPORT_MATRIX.md` | 2,218 | `5929f71d9faaba89305e15011ff1756bb644a279bdbef151ad649ee573966e1f` |
