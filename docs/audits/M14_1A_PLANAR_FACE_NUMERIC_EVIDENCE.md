# M14.1a Planar-Face Numeric Evidence

Retrieved: 2026-07-31

## Normative boundary

Autodesk's [3DFACE
reference](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-747865D5-51F0-45F2-BEFE-9572DBC5B151.htm)
defines four WCS corner tuples at groups 10/20/30 through 13/23/33 and
signed-16-bit group 70 invisible-edge flags. Autodesk's [SOLID
reference](https://help.autodesk.com/cloudhelp/2018/ENU/AutoCAD-DXF/files/GUID-E0C5F04E-D0C5-48F5-AC09-32733E8848F2.htm)
and [TRACE
reference](https://help.autodesk.com/cloudhelp/2016/ENU/AutoCAD-DXF/files/GUID-EA6FBCA8-1AD6-4FB2-B149-770313E93511.htm)
define four OCS corner tuples, optional group 39 thickness, and optional
210/220/230 extrusion direction.

Autodesk's [entity
inventory](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-7D07C886-FD1D-4A0C-A7AB-B4D21F18E484.htm)
and [object/entity code
rules](https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-A35B8C2A-1885-4A8E-8533-E61D8A423D62.htm)
bound the wider M14 completion plan.

## Read-only legacy evidence

`D:\SeaCad\cad_2026-07-23_source` was consulted only as a behavioral oracle.
The reviewed inventory and normalized-view notes report 540 planar-face source
records across 41 files, distinguish 3DFACE WCS from SOLID/TRACE OCS, and
identify defaults, edge bits, corner ordering, extrusion failures, and
block-local expansion as later semantic risks. No legacy parser implementation
was copied, translated, vendored, or linked.

Reviewed evidence files:

- `cad-file/dxf/CURRENT_STATUS.md`;
- `cad-file/dxf/raw/2026-07-02-accoreconsole-dxf-inventory-v2.md`;
- `cad-file/dxf/raw/2026-07-02-accoreconsole-dxf-entity-scope-v2.md`;
- `cad-file/dxf/knowledge/current-normalized-view.md`; and
- `crates/cad-dxf-lossless/tests/semantic_face_tests.rs`.

Autodesk documentation remains normative when legacy observations differ or
are incomplete.

## Implementation contract

`crates/seacad-dxf-core/src/planar_face_geometry.rs` builds one immutable,
source-bound directory over complete raw records in `BLOCKS` and `ENTITIES`.
It recognizes exact uppercase `3DFACE`, `SOLID`, and `TRACE` markers and
retains every documented numeric occurrence in source order.

Each entry preserves the raw group and span, semantic role, exact binary64
bits or signed-16-bit value, duplicate occurrences, and typed ASCII lexical
failure. Family-specific wire domains remain closed: group 70 is accepted only
for 3DFACE, while groups 39 and 210/220/230 are accepted only for SOLID/TRACE.
Construction is cancellable, bounds public counts to compact metadata, checks
source identity, and provides bounded raw-record and group-occurrence lookups.

The same public projection is available from ASCII, Binary, and shared raw
document views for all nine supported AC1009--AC1032 dialects.

## Test evidence

`crates/seacad-dxf-core/tests/planar_face_geometry_tests.rs` covers:

- ASCII/Binary parity for all nine supported dialects and all three families;
- every documented corner role, thickness, extrusion, and edge flags;
- exact negative-zero and binary64-bit retention;
- duplicates, invalid syntax, underflow/out-of-range values, and signed-16-bit
  overflow;
- exact marker case and `BLOCKS`/`ENTITIES` section scoping;
- rejection of cross-family numeric groups;
- empty evidence slices and absent lookups; and
- cancellation plus public `Copy`/`Send`/`Sync` bounds.

## Non-claims

M14.1a is occurrence evidence, not semantic completion of these entities. It
does not select values, apply missing-Z/fourth-corner/thickness/extrusion
defaults, interpret invisible-edge bits, validate completeness or normals,
reorder SOLID/TRACE corners, transform OCS to WCS, assemble faces, expand
blocks, edit, write, render, or tessellate.

## Required checkpoint gates

All required gates passed on 2026-07-31: dependency policy, generated-schema
drift, release-evidence drift, formatting, workspace Clippy with warnings
denied, 606 workspace tests with zero failures/ignored tests, and
`git diff --check`. No dependency manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 78 | `489ee7bc9294f771ee24f6be6864f5c30e5b23cadb3d84036e1ef799d9fe1073` |
| `crates/seacad-dxf-core/src/lib.rs` | 633 | `97fb79a929f39274aeff35bf8f77919203dd9ac34819d7e63c4a0f4c423e5246` |
| `crates/seacad-dxf-core/src/planar_face_geometry.rs` | 386 | `86f20c963e9e0d73257bd57918bbc5baab059019835e8d765e5ae5a287effa77` |
| `crates/seacad-dxf-core/tests/planar_face_geometry_tests.rs` | 370 | `06b4c885733de4762c8204055a10bef22022310dd0814f1ed739391510165a13` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 93 | `00f0c183b057ce3d324d6b27163c731aac30f32042254b33d8f6e98cd1815124` |
| `docs/IMPLEMENTATION_PLAN.md` | 1356 | `d01fd865024b971e3ba7956db57ff5bd00eaea341f48f0f03816e8b49950002e` |
| `docs/SUPPORT_MATRIX.md` | 1077 | `ff5b55a88bc5c02c31badf1906ad4679a534c8355033e99334de113dc66bf22e` |
