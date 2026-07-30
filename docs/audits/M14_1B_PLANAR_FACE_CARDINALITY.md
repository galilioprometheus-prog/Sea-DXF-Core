# M14.1b Planar-Face Cardinality

Retrieved: 2026-07-31

## Normative boundary

Autodesk's [3DFACE
reference](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-747865D5-51F0-45F2-BEFE-9572DBC5B151.htm)
defines twelve corner-component roles plus invisible-edge flags. Autodesk's
[SOLID
reference](https://help.autodesk.com/cloudhelp/2018/ENU/AutoCAD-DXF/files/GUID-E0C5F04E-D0C5-48F5-AC09-32733E8848F2.htm)
and [TRACE
reference](https://help.autodesk.com/cloudhelp/2016/ENU/AutoCAD-DXF/files/GUID-EA6FBCA8-1AD6-4FB2-B149-770313E93511.htm)
define twelve corner-component roles plus thickness and three extrusion
components.

M14.1b classifies the cardinality of those public roles. It does not infer
cardinality from the legacy oracle or use record position as semantic evidence.

## Legacy evidence boundary

`D:\SeaCad\cad_2026-07-23_source` remains a read-only behavioral oracle under
the M14.1a receipt. Its planar-face corpus observations inform later malformed
and geometry risk cases, but no legacy implementation was copied, translated,
vendored, or linked for this checkpoint. Autodesk documentation is normative.

## Implementation contract

`crates/seacad-dxf-core/src/planar_face_geometry_card.rs` builds an immutable,
source-bound cardinality directory over M14.1a evidence:

- every 3DFACE has thirteen stable cards;
- every SOLID and TRACE has sixteen stable cards;
- each card is explicitly absent, unique, or multiple;
- members retain source order and point back to exact M14.1a occurrences;
- lexical validity remains independent from occurrence count; and
- roles outside an entity family's public contract have no card.

The directory owns its evidence projection, checks source identity, observes
cancellation during construction, bounds public ordinals to compact metadata,
and provides raw-record, role, card, member, and evidence lookups. It copies no
numeric value and performs no selection or fallback.

## Test evidence

`crates/seacad-dxf-core/tests/planar_face_geometry_card_tests.rs` covers
ASCII/Binary parity for all nine AC1009--AC1032 dialects; the 13/16/16 stable
card shapes; absent, unique, and multiple states; exact member value domains;
negative-zero retention through evidence lookup; invalid ASCII values under
unique and multiple cardinality; cross-family role absence; bounded missing
lookups; cancellation; and public `Copy`/`Send`/`Sync` bounds.

## Non-claims

M14.1b does not select values, apply missing-Z/fourth-corner/thickness/extrusion
defaults, interpret invisible-edge bits, validate completeness or normals,
reorder SOLID/TRACE corners, transform OCS to WCS, assemble faces, expand
blocks, edit, write, render, or tessellate.

## Required checkpoint gates

All required gates passed on 2026-07-31: dependency policy, generated-schema
drift, release-evidence drift, formatting, workspace Clippy with warnings
denied, 609 workspace tests with zero failures/ignored tests, and
`git diff --check`. No dependency manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 78 | `324939e4c25f28f3199d2757bc5beee85df5dd8cb22657e12b292d69b2f5d223` |
| `crates/seacad-dxf-core/src/lib.rs` | 638 | `b638e5916ce27093cefd06c3fc5d3272792efa637971456666adfe19027c1088` |
| `crates/seacad-dxf-core/src/planar_face_geometry_card.rs` | 346 | `ea33d951368593adc678049ecbcd5fb97ee69f2c8bcfa35f34c3d32cb44bbc68` |
| `crates/seacad-dxf-core/tests/planar_face_geometry_card_tests.rs` | 322 | `2ccd371fb3af8b8d69a30e110b7f20aac9b9fcb3c1d05629681a8279b94778ed` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 95 | `7b91688afaaaac8c5587054da5ab4e1c8db1f0f5257ad727b647bee9eebb674e` |
| `docs/IMPLEMENTATION_PLAN.md` | 1362 | `125b7da3a9b62ff0923394dc835c4f6a1f2fdde4f4541ed15ba336f168f8b293` |
| `docs/SUPPORT_MATRIX.md` | 1085 | `a3e3d5b62e9e4900d238e2ebb29f4240a147b7a341e77f371635c0a7fc87dd78` |
