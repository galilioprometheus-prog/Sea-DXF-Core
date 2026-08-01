# M14.3an Common Reference Targets

## Scope

M14.3an validates the exact public record kind of three uniquely resolved
common entity references without changing the source or selecting an
ambiguous handle target.

## Sources

- Autodesk common entity codes assign extension dictionary group 360,
  material group 347, and plot-style group 390:
  `https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm`
- Autodesk defines the exact `DICTIONARY` object marker:
  `https://help.autodesk.com/cloudhelp/2015/ENU/AutoCAD-DXF/files/GUID-40B92C63-26F0-485B-A9C2-B349099B26D0.htm`
- Autodesk defines the exact `MATERIAL` object marker:
  `https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-E540C5BB-E166-44FA-B36C-5C739878B272.htm`
- Autodesk defines the exact `ACDBPLACEHOLDER` object marker:
  `https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-3BC75FF1-6139-49F4-AEBB-AE2AB4F437E4.htm`
- Autodesk's managed `PlaceHolder` reference states that placeholder objects
  are used in the Plot Style Name Dictionary:
  `https://help.autodesk.com/view/OARX/2026/ENU/?guid=OARX-ManagedRefGuide-Autodesk_AutoCAD_DatabaseServices_PlaceHolder`
- Autodesk's OBJECTS inventory includes dictionary, material, and placeholder
  object topics:
  `https://help.autodesk.com/cloudhelp/2025/ENU/AutoCAD-DXF/files/GUID-2D71EE99-A6BE-4060-9B43-808CF1E201C6.htm`

These public definitions directly establish the target markers and section;
no native behavioral receipt is needed for this checkpoint. No legacy or
external code, fixture, data, or dependency was copied, translated, vendored,
or linked.

## Contract

- `ExtensionDictionary`, `Material`, and `PlotStyle` expose their exact
  expected `OBJECTS` target marker.
- A uniquely resolved explicit target is usable only when its record section
  and exact marker both match. Another marker or section retains the target as
  typed incompatible evidence.
- Raw field failures, null, missing, ambiguous, absence, and the generated
  material `ByLayer` default are projected before target-kind validation.
- Owner group 330 passes through the same source semantics as explicitly
  unreviewed. Its valid target kind depends on entity family and placement.
- The directory is source-bound, cancellation-aware, allocation-bounded, and
  exposes stable per-entity and per-field lookup without a second source scan.

## Nonclaims

This checkpoint does not establish authoritative ownership, one-owner
conformance, dictionary membership, pointer lifecycle, reference-safe edits,
version applicability, family graph validity, or `Complete` entity support.

## Verification

Focused tests cover ASCII/Binary parity for AC1009 through AC1032, all three
valid target kinds, wrong markers, a correct marker in the wrong section,
source null/missing/ambiguous precedence, generated defaults, cancellation,
source identity, public bounds, and bounded public types. Final gate counts,
production diff, and artifact hashes are recorded below after the release
gate. The focused suite passed 8/8 tests and the full workspace passed 854/854
tests. Generated schema and release-evidence checks, `cargo deny --locked
check`, formatting, workspace Clippy with warnings denied, workspace tests, and
`git diff --check` all passed. The production diff is 348 added and 0 removed
lines: 342 in the target-kind projection and 6 module/exports. This audit
intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 204 | `75f10117943951afb9b3bb5288e08506984b9bfd5828f3599e59a5396f2a9f84` |
| `crates/seacad-dxf-core/src/entity_common_reference_target.rs` | 342 | `aa7a88a590fe3a0c73b00a86c8b9ded941e9ee64f84cca792e0e9988b0cd7e03` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,011 | `6eb3420d03a0867f7f235657eaec4ca0bd34983d50941d7e147e3017c70b96b5` |
| `crates/seacad-dxf-core/tests/entity_common_handle_semantic_tests.rs` | 866 | `5909fbb197631555092bf5e6defabc7f189b046b20922679bdd42587805c3cf4` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 739 | `5205660d83c249df2aebf1690943e9e55f84eab3b74f29736d9438f418615a12` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,169 | `aa11a06c866a1b1bb8e3f0c049a01ea1edc072ae41bd90293d221918ca6f16ed` |
| `docs/SUPPORT_MATRIX.md` | 1,829 | `b1f202adb278ab94ec9b5c0f4eafca3f7133afbb70ff864e5cd709d21504e04d` |
