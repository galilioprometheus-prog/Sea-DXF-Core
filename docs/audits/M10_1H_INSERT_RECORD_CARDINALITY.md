# M10.1h INSERT Record Cardinality

Retrieved: 2026-07-30

## Normative boundary

Autodesk [INSERT
(DXF)](https://help.autodesk.com/cloudhelp/2018/ENU/AutoCAD-DXF/files/GUID-28FA4CFB-9D5E-4880-9F11-36C97578252F.htm)
documents the 16 defining roles retained by M10.1g. M10.1h accounts for the
cardinality of every role in every exact INSERT record without applying the
documented optional defaults.

## Implementation contract

`crates/seacad-dxf-core/src/insert_record_card.rs` publishes 16 fixed cards per
M10.1g record in documented role order:

- `Absent` when the role has no retained value;
- `Unique` when it has exactly one value;
- `Multiple` with the exact occurrence count otherwise.

Compact card members reference every M10.1g value ordinal in source order.
Cards remain record-local across BLOCKS and ENTITIES. Lexical validity does not
alter cardinality, and the retained M10.1g directory remains authoritative.

## Test evidence

`crates/seacad-dxf-core/tests/insert_record_card_tests.rs` covers ASCII/Binary
parity across all nine supported dialects, all 16 roles, absent/unique/multiple
states, duplicate ordering, invalid numeric independence, group `102`
exclusion inherited from M10.1g, record locality, bounded lookup,
cancellation, source identity, and public traits.

## Non-claims

M10.1h does not apply defaults, select duplicate values, assemble typed INSERT
semantics, resolve the block name, follow ATTRIB/SEQEND, transform member
geometry, edit, write, or render.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, formatting,
generated-schema drift, workspace Clippy with warnings denied, 459 workspace
tests with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/insert_record_card.rs` | 324 | `7cf88b36f95feb86793840f5e78fe91bc631147d96a00f7b1b3b7713ddd53722` |
| `crates/seacad-dxf-core/src/lib.rs` | 464 | `90b337356b910378339056329b92974e813766d26a7037e82ee6094d433a5c6d` |
| `crates/seacad-dxf-core/tests/insert_record_card_tests.rs` | 393 | `863fa92dfe2659e600f7b947ac510563b53012c3dba2498faeffa0c3a2e6f5a2` |
| `docs/IMPLEMENTATION_PLAN.md` | 801 | `316c1c574509cd4922b80f3cd9d5584abf707b15ff6b3f2201044a535088396b` |
| `docs/SUPPORT_MATRIX.md` | 683 | `b73b2c43746c5cc4660971cd6603c1c9dc1af54d31d2fd8620eaadc1013218a3` |
