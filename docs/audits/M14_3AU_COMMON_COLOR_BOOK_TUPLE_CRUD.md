# M14.3au Common Color-Book Tuple CRUD

## Scope

M14.3au adds one typed composite edit that inserts or replaces common entity
groups 62, 420, and 430 as a single logical session operation with rollback on
partial planning failure.

## Sources

- Autodesk common entity codes define group 62 indexed color, group 420 24-bit
  true color, and group 430 color name:
  `https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm`
- Autodesk `acad_truecolordlg` returns groups 62, 420, and 430 for a color-book
  selection and defines group 430 as `colorbook$colorname`:
  `https://help.autodesk.com/cloudhelp/2018/DEU/AutoCAD-AutoLISP-Reference/files/GUID-E6FF435F-9E66-4F37-8770-2E3FB87B8E0B.htm`
- Autodesk's numerical group-code reference defines 420 as `0x00RRGGBB` and
  430 as its True Color name string:
  `https://help.autodesk.com/cloudhelp/2020/ENU/AutoCAD-DXF/files/GUID-3F0380A5-1C15-464D-BC66-2C5F094BCFB9.htm`

The two user-authorized legacy snapshots under
`D:\SeaCad\tham khảo\New folder` were inspected read-only. They project MTEXT
background groups 420/430 independently and contain no reusable common-color
tuple contract. The public ezdxf and LibreCAD repositories were considered as
behavioral references only; neither supplied stronger normative evidence than
Autodesk. No external or legacy code, fixture, data, or dependency was copied,
translated, vendored, or linked.

## Contract

- `DxfEntityCommonColorBookPatch` carries caller-owned exact name bytes and
  already validated indexed/true-color public types. Its debug representation
  exposes only name byte count and scalar values.
- `DxfEntityPatch::CommonColorBook` validates the one-separator name envelope,
  rejects any pre-existing pending edit for the three fields, and treats the
  three planned components as one logical request.
- If any component cannot be inserted or replaced, every component appended by
  that request is truncated from the pending queue. Earlier unrelated edits
  remain intact.
- AC1009 is rejected in both physical formats because its one-byte Binary group
  code cannot represent groups 420/430; this is a parity restriction, not a
  broader applicability claim.
- Accepted modern tuples retain canonical writer order, strict-reparse to one
  structured color-book semantic value, satisfy three generic postconditions,
  and carry an executable byte-identical inverse.

## Nonclaims

This checkpoint does not read `.acb` files, verify external book/name or
name-to-RGB/ACI mappings, define indexed-color proximity, reset all three
fields together, establish Autodesk version applicability beyond the AC1009
wire parity restriction, add entity-family graphs, or advance an entity to
`Complete`.

## Verification

Paired ASCII/Binary fixtures cover all nine dialects, complete tuple insertion
and replacement, AC1009 rejection, syntax rejection, existing pending-field
conflict, duplicate-source rollback after an earlier component was planned,
structured post-image values, three-field semantic/raw verification,
byte-identical inverse restoration, public traits, and debug redaction. Final
gate counts, production diff, and artifact hashes are recorded after the
release gate. This audit intentionally omits its own hash.

The focused color-book suite passed 4/4 tests and the full workspace passed
872/872 tests. Generated schema and release-evidence checks, `cargo deny
--locked check`, formatting, workspace Clippy with warnings denied, workspace
tests, and `git diff --check` all passed. The production diff is 155 added and
10 removed lines: 7 added and 3 removed in color-book admission, 145 added and
5 removed in the edit session, and 3 added and 2 removed in exports. No
production dependency changed.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 231 | `63e8fd9ab5b369e5375ca00e1b1131e5948886c3015306c088fa1884353dbe98` |
| `crates/seacad-dxf-core/src/entity_common_color_book_edit.rs` | 293 | `96af6706301ae1e93ac9fb86f4557856c401af0ec460dda8590c550a9a047800` |
| `crates/seacad-dxf-core/src/entity_edit_session.rs` | 814 | `2f84d074326b5387dbff7b269ef80a38d997671a42087b5f95b49e3af3a2afc6` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,037 | `177b525903db70f9e4e5b7597e478b520865a465a6cad7cb4fc5b79ba7996afd` |
| `crates/seacad-dxf-core/tests/entity_common_color_book_edit_tests.rs` | 556 | `76830237bca798dcc7bb6f5f7cea4a37ceceeb48eed6f4b682d0e4aeb332ce94` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 831 | `cffb153bd9ea8dea37c00df355759fd9e0e03a1f44758f286517d9d6694535c1` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,261 | `d92db0917fc77bac7d8b68e6a1f5b04472ec6cec450f571a848a82f8ddcbf8d6` |
| `docs/SUPPORT_MATRIX.md` | 1,919 | `f6516abd5a9b53f606a330ade8d9de0b24f2aece0539471e8234ef87cb8a2fd6` |
