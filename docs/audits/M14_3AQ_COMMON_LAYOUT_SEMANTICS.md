# M14.3aq Common Layout Semantics

## Scope

M14.3aq indexes public layout-name evidence from closed `OBJECTS` sections and
resolves common entity group 410 to exact same-document layout objects without
changing source bytes.

## Sources

- Autodesk common entity codes define group 410 as the layout tab name:
  `https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm`
- Autodesk defines `LAYOUT` as an object, places the layout name in group 1
  after subclass marker `AcDbLayout`, and lists the preceding inherited
  `AcDbPlotSettings` subclass:
  `https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-433D25BF-655D-4697-834E-C666EDFD956D.htm`
- Autodesk requires order-independent entity/object group processing and
  preservation of future undefined codes:
  `https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-A35B8C2A-1885-4A8E-8533-E61D8A423D62.htm`

The user-authorized legacy repository was inspected read-only. Its older
standard-reference projection associated layout objects with block records but
did not provide the required subclass-aware group-1 cardinality contract. It
was treated only as a negative behavioral boundary. No legacy or external
code, fixture, data, or dependency was copied, translated, vendored, or linked.

## Contract

- `layout_object_directory` indexes exact uppercase `LAYOUT` markers only in
  completely closed `OBJECTS` sections and retains source identity, raw-record
  evidence, and application-group evidence.
- Group 1 is counted only while the exact `AcDbLayout` subclass is active.
  Plot-settings names, application content, later subclasses, and wrong-section
  records cannot become layout names.
- Every admitted record exposes `Missing`, `Unique`, or `Duplicate` name state;
  duplicate values are never selected.
- `DxfEntityCommonTextDirectory` projects group 410 as absent, exact unique,
  missing, ambiguous, or a preserved raw-field failure. A digest narrows lookup
  candidates, followed by an exact source-byte collision check.
- Work remains cancellation-aware and bounded by the existing raw record,
  group, value, and source limits.

## Nonclaims

This checkpoint does not define case folding, Unicode normalization, legal
layout-name characters, version applicability, reciprocal `BLOCK_RECORD`
ownership, layout edit admission, layout creation/deletion, color-book
resolution, family graphs, or `Complete` entity support.

## Verification

Focused tests cover all nine dialects in paired ASCII/Binary form, inherited
plot-settings group 1, application-group exclusion, missing and duplicate
layout names, ambiguous target names, wrong-section records, raw field failure,
cancellation, source identity, public bounds, and debug redaction. Final gate
counts, production diff, and artifact hashes are recorded after the release
gate. This audit intentionally omits its own hash.

The focused common-text and edit/write regression suites passed 22/22 tests;
the authoritative full-workspace rerun passed 863/863 tests. The first full
run was terminated by the command's 120-second host timeout and its test
harness consequently reported a broken pipe; the unchanged command completed
successfully when rerun with a five-minute ceiling. Generated schema and
release-evidence checks, `cargo deny --locked check`, formatting, workspace
Clippy with warnings denied, workspace tests, and `git diff --check` all passed.
The production diff is 385 added and 7 removed lines: 222 in the new layout
object directory, 146 net in common-text semantic composition, and 3 module/
exports. No production dependency changed.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 219 | `5e204705958b19016d5915a00bf288e0f1e8e48fc16e8a6ea211e685e159c7e8` |
| `crates/seacad-dxf-core/src/entity_common_text_semantic.rs` | 560 | `31d0edef2fd72b706f2d69ba529cb61d9155f43c8badd62c95da5f32b34e6ccd` |
| `crates/seacad-dxf-core/src/layout_object.rs` | 222 | `aef98cfd58cc4b9fc6f4407f537cd1a97b911413cf9c86ebe648e6762e47469b` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,024 | `03e9aea1a5763c3d5396ae8cabed7d8689a54ee29583f7062c7b34c1f1edb1a1` |
| `crates/seacad-dxf-core/tests/entity_common_text_semantic_tests.rs` | 535 | `c218f4749cece8a7e77e6a858636242225a6aed46417d91b2d5fb2dcf34a3e83` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 778 | `8856adaa2662e66de3fa243935d411ff879af54d51d5d0711bf196417d472701` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,211 | `304b3ae491828edacfa8dff333383c0f619642404a8534d666f520844b008020` |
| `docs/SUPPORT_MATRIX.md` | 1,872 | `3f22c5f2860996a78754c90400f17fd4febcbcb733f04a0af7545130771fc987` |
