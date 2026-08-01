# M14.3ak Common Text Semantics

## Scope

M14.3ak extends completely closed named-symbol membership with `LAYER` and
`LTYPE`, then projects the four exact-text common entity fields. It resolves
only exact same-document layer and linetype names; layout and color name remain
explicitly unreviewed exact text.

## Sources

- Autodesk common entity codes define layer group 8, optional linetype group 6
  with BYLAYER default, layout group 410, and color-name group 430:
  `https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm`
- Autodesk defines group 2 as the layer name for `LAYER` table entries:
  `https://help.autodesk.com/cloudhelp/2015/ENU/AutoCAD-DXF/files/GUID-D94802B0-8BE8-4AC9-8054-17197688AFDB.htm`
- Autodesk defines group 2 as the linetype name for `LTYPE` table entries:
  `https://help.autodesk.com/cloudhelp/2025/ENU/AutoCAD-DXF/files/GUID-F57A316C-94A2-416C-8280-191E34B182AC.htm`
- Autodesk's symbol-table structure states that a table entry has the same
  group-0 marker as its table and group 2 supplies the entry name:
  `https://help.autodesk.com/cloudhelp/2017/ENU/AutoCAD-DXF/files/GUID-5AB9300F-F0AC-4ADE-89EA-A9D1D152D8B8.htm`

The user-authorized legacy trees remained read-only. No external or legacy
code, fixture, data, or dependency was copied, translated, vendored, linked,
or added at runtime.

## Contract

- `DxfNamedSymbolTableKind` admits exact `LAYER` and `LTYPE` records only from
  completely closed matching TABLE envelopes with one group-2 name.
- `DxfEntityCommonTextDirectory` owns the generic common-field semantics, named
  membership evidence, and four stable entries per entity-bearing record.
- Layer and linetype source spans are hashed to bound candidate lookup, then
  compared byte-for-byte to defeat digest collisions. Exact duplicate names
  remain ambiguous; no occurrence is selected.
- A unique match retains the source text and exact table record. Missing and
  ambiguous names retain raw provenance as typed invalid states.
- Omitted linetype remains defaulted `ByLayer` with no fabricated source or
  target. Field decode/cardinality failure takes precedence over name lookup.
- Layout and color name pass through the exact four-state generic semantics as
  `ExactUnreviewed`.
- Construction and lookup are source-bound, cancellation-aware,
  allocation-fallible, group-order independent, and payload-redacted.

## Nonclaims

M14.3ak does not define AutoCAD case-insensitive matching, valid symbol-name
characters, XREF-dependent name rules, Unicode normalization, layout-object
resolution, color-book semantics, applicability, or name-safe editing. It does
not advance any entity topic to `Complete`.

## Verification

Paired ASCII/Binary fixtures cover AC1009 through AC1032 with exact unique
layer/linetype targets. Negative fixtures cover missing names, duplicate exact
table names, duplicate entity fields, `BYLAYER` default, missing modern layout,
unreviewed explicit/absent text, wrong-section marker retention, exact
provenance, cancellation, source mismatch, bounded lookup, public traits, and
debug redaction. Final gate counts, production diff, and artifact hashes are
recorded below. The focused suite passed 4/4 tests, named-symbol and generic
field regressions passed 7/7, and the full workspace passed 846/846 tests.
Schema generation and release-evidence checks, `cargo deny --locked check`,
formatting, workspace Clippy with warnings denied, workspace tests, and `git
diff --check` all passed. The production diff is 425 added and 2 removed lines:
412 in the common-text projection, 8 added and 2 removed in named-symbol table
membership, and 5 module/exports. This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 189 | `cbd9dbfdd2fe37ef187c66ee6879d545cae93e239a00dddca42dc2af0feff0ff` |
| `crates/seacad-dxf-core/src/entity_common_text_semantic.rs` | 412 | `4093a00b224da8697fed476c1db8e18f42096254386a2ba53dad9b3699cb134d` |
| `crates/seacad-dxf-core/src/named_symbol_table.rs` | 273 | `60f894f7efe0be3304bb230caed1c2b415182f26533d4325362b5e369186510d` |
| `crates/seacad-dxf-core/src/lib.rs` | 996 | `16adbe44a3d1248fd8e4ad81c6c543ba33412bb10932578ae1dd2d123904f0eb` |
| `crates/seacad-dxf-core/tests/entity_common_text_semantic_tests.rs` | 420 | `b04a226ef42753e547229be6bf988fb6f4604c3aada8612602539d52fa1c21b5` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 706 | `0d08526fd3d3733cf8f02864b6cf63487d78357d5f04871cffe836b89ceaa575` |
| `docs/IMPLEMENTATION_PLAN.md` | 2133 | `423f03b3cb4fe9bd36d6b301db66afa1b826b01985236bcbcacd6b8b830b7d19` |
| `docs/SUPPORT_MATRIX.md` | 1792 | `474bce3fcc8c590e00a8e34a610d96ea0e7a7ad54ffd6d6621da43d159332f60` |
