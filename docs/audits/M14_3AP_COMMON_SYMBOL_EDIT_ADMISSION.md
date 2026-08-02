# M14.3ap Common Symbol Edit Admission

## Scope

M14.3ap admits common exact-text edits only when a reviewed symbol name resolves
to one exact same-document table entry, while unreviewed object-backed names
require specialized resolvers.

## Sources

- Autodesk common entity codes define layer group 8, linetype group 6 with
  `BYLAYER` omission default, layout group 410, and color-name group 430:
  `https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm`
- Autodesk defines group 2 as the `LAYER` table entry name:
  `https://help.autodesk.com/cloudhelp/2015/ENU/AutoCAD-DXF/files/GUID-D94802B0-8BE8-4AC9-8054-17197688AFDB.htm`
- Autodesk defines group 2 as the `LTYPE` table entry name:
  `https://help.autodesk.com/cloudhelp/2025/ENU/AutoCAD-DXF/files/GUID-F57A316C-94A2-416C-8280-191E34B182AC.htm`
- Autodesk's symbol-table structure requires matching TABLE/entry markers and
  uses group 2 for the entry name:
  `https://help.autodesk.com/cloudhelp/2017/ENU/AutoCAD-DXF/files/GUID-5AB9300F-F0AC-4ADE-89EA-A9D1D152D8B8.htm`

The user-authorized legacy repository was searched read-only. Its evidence
explicitly left layer edit behavior and broader topology unclaimed, reinforcing
the fail-closed resolver boundary. No legacy or external code, fixture, data,
or dependency was copied, translated, vendored, or linked.

## Contract

- `classify_entity_common_symbol_edit` returns `NotSymbol`, a target-bearing
  `Valid` value, or a typed issue without changing source bytes.
- Layer and linetype require `ExactRawText` and exactly one byte-identical name
  in the corresponding completely closed table. Zero and multiple targets are
  distinct typed outcomes; no occurrence is selected.
- Layout and color name return dedicated resolver requirements. Generic
  singleton replacement cannot bypass those open semantic policies.
- `DxfEntityEditSession` builds one named-symbol directory lazily and reuses it
  across requests. Oversized exact text fails the resource ceiling before the
  scan; every semantic rejection leaves the queue unchanged.
- Accepted edits use the existing encode/plan, strict semantic post-image,
  raw-byte verification, and executable inverse pipeline.

## Nonclaims

This checkpoint does not define case folding, legal symbol characters, XREF
name policy, Unicode normalization, layout-object or color-book resolution,
symbol-table record insertion, version applicability, family graphs, or
`Complete` entity support.

## Verification

Focused tests cover both reviewed symbol kinds, missing and duplicate exact
names, wrong value kinds, layout/color specialized requirements, cancellation,
all-nine-dialect ASCII/Binary parity, accepted and rejected session behavior,
strict post-image semantics, create-new write regressions, cleanup behavior,
and byte-identical inverse restoration. Final gate counts, production diff,
and artifact hashes are recorded below after the release gate.
The focused and edit/write regression suites passed 21/21 tests and the full
workspace passed 862/862 tests. Generated schema and release-evidence checks,
`cargo deny --locked check`, formatting, workspace Clippy with warnings denied,
workspace tests, and `git diff --check` all passed. The production diff is 285
added and 5 removed lines: 215 in symbol edit admission, 58 net in edit-session
integration/resource precedence, 2 net in the shared symbol-kind helper, and 5
module/exports. This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 213 | `5952a0968c00b061cf2254840fcdc68e3b1501153bb34fcbe1802bc21e6381a3` |
| `crates/seacad-dxf-core/src/entity_common_symbol_edit.rs` | 215 | `e29ca6b5969928abf5779c379efadb550b14afc00e95432cd0a715861f0affa2` |
| `crates/seacad-dxf-core/src/entity_common_text_semantic.rs` | 414 | `0f42bd8da8abdd09e4b8919886f42883befb3246e7486a147f9077d73c1cc93e` |
| `crates/seacad-dxf-core/src/entity_edit_session.rs` | 596 | `5a186b3ee2ece02d7e502a5e0b5958f1579e74cec831fd3614a0f88b13313103` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,021 | `fd624925f1a2d86b35ab2f613551321416267fb7652c4058f2759a2e6fb5fb2c` |
| `crates/seacad-dxf-core/tests/entity_common_symbol_edit_tests.rs` | 460 | `dcb3a61afe06593115fcc01bb078b64eaf768a94ec740951cf5ae6b415baf587` |
| `crates/seacad-dxf-core/tests/entity_edit_session_tests.rs` | 654 | `9bc5bdcf51aac0f092dbaa0e6e9a3d67466f5e9064d52587b5107075786b64fd` |
| `crates/seacad-dxf-core/tests/entity_edit_verification_tests.rs` | 530 | `b15b8ce3267a22ea44edb1b7c4342cbfe893456da78edd3d6b74d6686cb3bc7e` |
| `crates/seacad-dxf-core/tests/entity_edit_write_tests.rs` | 541 | `5e1faa032b9f2856c72dd5c4ed4f14f2ba616c69fb3c388d20e50b2a4b513c08` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 765 | `c5a6b1366534e0f22c74a7eeb465e88555bff02d72b2942ad8d18f79f8962030` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,197 | `caff3953a328d82da8db5d1e338ec31aa9cec39222b683ab12a47b72b13ffbf0` |
| `docs/SUPPORT_MATRIX.md` | 1,858 | `6c1b81d54876ce3bfc4350ea2c45c9009b0735f014de115b5d3b877e55aa2364` |
