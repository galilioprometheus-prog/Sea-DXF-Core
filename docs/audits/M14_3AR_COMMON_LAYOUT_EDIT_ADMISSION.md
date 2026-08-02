# M14.3ar Common Layout Edit Admission

## Scope

M14.3ar admits a common group-410 edit only when its exact name resolves to one
same-document `OBJECTS`/`LAYOUT` target from the M14.3aq directory.

## Sources

- Autodesk common entity codes define group 410 as the layout tab name:
  `https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm`
- Autodesk defines group 1 inside subclass `AcDbLayout` as the layout name:
  `https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-433D25BF-655D-4697-834E-C666EDFD956D.htm`

The user-authorized legacy repository was consulted read-only. Its older edit
policy did not implement common group-410 name admission or strict post-image
verification, so it supplied no portable implementation. No legacy or external
code, fixture, data, or dependency was copied, translated, vendored, or linked.

## Contract

- `classify_entity_common_layout_edit` returns `NotLayout`, one target-bearing
  `Valid` value, or typed wrong-value, missing, or ambiguous issues without
  modifying the source.
- Matching considers only M14.3aq entries with one valid `AcDbLayout` group-1
  name and requires exact source bytes. It never selects one ambiguous target.
- The edit session builds one layout directory lazily and reuses it. The exact-
  text resource ceiling precedes directory construction and lookup.
- A valid proposal continues through the existing common singleton planner,
  strict independent reparse, semantic and raw verification, and executable
  inverse journal. Rejected proposals leave the queue and source unchanged.
- AC1009 Binary's one-byte group-code encoding cannot represent group 410 and
  returns the existing typed insertion/encoding issue without queueing.

## Nonclaims

This checkpoint does not define case folding, Unicode normalization, legal
layout-name characters, group-410 version applicability, reciprocal
`BLOCK_RECORD` ownership, layout creation/deletion, color-book resolution,
family graphs, or `Complete` entity support.

## Verification

Focused tests cover all-nine-dialect ASCII/Binary classifier parity, exact case
sensitivity, missing and ambiguous targets, malformed layout-object exclusion,
wrong value kinds, non-layout fields, rejected no-op behavior, cancellation,
modern ASCII/Binary canonical insertion, strict target-bearing post-image
semantics, semantic/raw verification, byte-identical inverse restoration, and
AC1009 Binary's physical group-code rejection. Final gate counts, production
diff, and artifact hashes are recorded after the release gate. This audit
intentionally omits its own hash.

The focused classifier/semantic/edit/write regression suites passed 26/26
tests and the full workspace passed 867/867 tests. Generated schema and
release-evidence checks, `cargo deny --locked check`, formatting, workspace
Clippy with warnings denied, workspace tests, and `git diff --check` all passed.
The dependency gate required access to Cargo's advisory database lock outside
the workspace sandbox and passed unchanged after that scoped approval. The
production diff is 219 added and 9 removed lines: 181 in layout edit admission,
32 net in edit-session integration, 5 module/exports, and removal of the 8-line
obsolete layout requirement from the symbol classifier. No production
dependency changed.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 221 | `bdd43bb82ffcd7dc834d8bce7841c6b6b1ce5c6c07cd7ed82dcc217e672c349d` |
| `crates/seacad-dxf-core/src/entity_common_layout_edit.rs` | 181 | `eaf5a82f01df5c658d9898b4e46391cf412a08273fcbce7018dbe88e06213ba1` |
| `crates/seacad-dxf-core/src/entity_common_symbol_edit.rs` | 207 | `94acc45d0bb1145b4cb14d5c299fd20d2a6912ef023368e604c7e4aa602806ac` |
| `crates/seacad-dxf-core/src/entity_edit_session.rs` | 628 | `60398ea77eada88ff157c8bda0540db4befad1f58734489e5d3c1dcc5f5b767b` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,029 | `d8ab11892ade46eab99f29f979fdc57681c841670f5f9f1d0887091a356a7762` |
| `crates/seacad-dxf-core/tests/entity_common_layout_edit_tests.rs` | 396 | `67ce1d3f6d60c310cba06ca0a99e7102127190c69c7c7ea1a684a0bc389b6d24` |
| `crates/seacad-dxf-core/tests/entity_common_symbol_edit_tests.rs` | 466 | `c78f33779ac3082f2e9424f34841c25533e3a017027b5f719e2386fa5523ed82` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 792 | `8993f173cd598561fe667abecd07e5a1b42ca87f6f5424e3ae78e369cf3b6ce3` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,224 | `4927a7923c07d2aeb8285deee5230169f5f791afd2d489fde1ecf00a7175b001` |
| `docs/SUPPORT_MATRIX.md` | 1,885 | `94c01ee471a27b61559506a16c4f58460a9ade050dfa9af3def61e70188db9a9` |
