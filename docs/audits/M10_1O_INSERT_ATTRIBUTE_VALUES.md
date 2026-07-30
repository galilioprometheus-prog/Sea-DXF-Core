# M10.1o Classic INSERT Attribute Values

Retrieved: 2026-07-30

## Normative boundary

Autodesk [ATTRIB
(DXF)](https://help.autodesk.com/cloudhelp/2020/ENU/AutoCAD-DXF/files/GUID-7DD8B495-C3F8-48CD-A766-14F9D7D0DD9B.htm)
documents the classic AcDbText/AcDbAttribute fields and a later
AcDbXrecord/AcDbMText extension that reuses several group codes. Autodesk also
warns in [Common Group Codes for Entities
(DXF)](https://help.autodesk.com/cloudhelp/2015/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm)
that readers must not depend on the displayed group-code order.

M10.1o therefore tracks exact subclass markers. It does not infer field meaning
from position, and it stops classic role assignment before AcDbXrecord.

## Implementation contract

`crates/seacad-dxf-core/src/insert_attribute_value.rs` layers value evidence
only over exact ATTRIB records retained by M10.1n. It publishes 23 classic
roles covering:

- thickness, OCS text start, height, value, and tag;
- attribute/text flags, field length, rotation, width, oblique angle, style,
  and horizontal/vertical justification;
- optional OCS alignment point and extrusion; and
- neutral `VersionOrLockPosition` evidence for group `280`.

Every occurrence remains in source order as exact source-anchored text,
binary64, or signed-16-bit data. Duplicate values, invalid ASCII numbers, raw
spans, sequence ownership, and sequence-local attribute ordinals remain
explicit. The two documented group-280 meanings share one neutral role because
the wire code does not distinguish them and group order is not authoritative.

Legacy records and exact AcDbText/AcDbAttribute contexts are admitted. Other
subclass contexts are ignored, group-102 application content is excluded, and
scanning stops at exact AcDbXrecord so later MText/Xrecord groups cannot
impersonate classic roles. All excluded bytes remain authoritative in the raw
document.

## Test evidence

`crates/seacad-dxf-core/tests/insert_attribute_value_tests.rs` covers
ASCII/Binary parity across all nine supported dialects; all 23 classic roles;
both group-280 occurrences where physically representable; source order;
duplicate and invalid numeric evidence; exact text projection and source
identity; legacy and subclass-aware records; unrelated subclass, group-102,
and AcDbXrecord exclusion; empty attributes; orphan ATTRIB exclusion; compact
ranges; lookup; cancellation; locality; and public traits.

## Non-claims

M10.1o does not assign cardinality, select values, apply defaults, validate
text/numeric domains, interpret flags or justification, distinguish the two
group-280 meanings, decode the MText extension, associate ATTDEF definitions,
transform attributes, edit, write, or render.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, formatting,
generated-schema drift, workspace Clippy with warnings denied, 491 workspace
tests with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

The production module is 533 physical lines, close to the 200-500 review guide;
the additional lines keep the 23-role wire contract, source-anchored text
receipt, and subclass boundary in one independently testable module.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/insert_attribute_value.rs` | 533 | `f2dcf9d8e35abaa6371c7ef532f265f930535ec5a9e9ada108b2f5e103e09e03` |
| `crates/seacad-dxf-core/src/lib.rs` | 501 | `6dc4093c1c46f85bceaaeb5d69c81b34082b8163e2522bf27764bc78f17ffc51` |
| `crates/seacad-dxf-core/tests/insert_attribute_value_tests.rs` | 381 | `0bc768e4ea19be6e9522720d6c2524f7146d36e63d7b6d41f4154fd763668173` |
| `docs/IMPLEMENTATION_PLAN.md` | 887 | `af118c5cfd199d4e1a3b1d76e6e60c3d6b5a96648bae81f315f5f649844c31d5` |
| `docs/SUPPORT_MATRIX.md` | 751 | `0872150b34d3d510426677b4c54cee995c51cff6fa0b6028e66c24fc78630907` |
