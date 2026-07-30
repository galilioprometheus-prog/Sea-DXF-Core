# M10.1y Classic ATTDEF Cardinality Cards

Retrieved: 2026-07-30

## Normative boundary

Autodesk [ATTDEF
(DXF)](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-F0EA099B-6F88-4BCC-BEC7-247BA64838A4.htm)
publishes the classic ATTDEF field codes but does not authorize SeaCad to
silently choose among duplicate occurrences. It assigns both version and
lock-position meanings to group `280`.

M10.1y adds explicit per-role cardinality over the M10.1x wire evidence before
any typed semantic selection.

## Implementation contract

`crates/seacad-dxf-core/src/block_attribute_definition_card.rs` publishes
exactly 24 stable cards for every M10.1x ATTDEF record, in the same role order
as its evidence registry. Each card is independently:

- `Absent`;
- `Unique`; or
- `Multiple { occurrence_count }`.

Compact members reference every matching M10.1x value ordinal in source order.
Cards do not inspect whether ASCII numeric text is lexically valid, and they do
not depend on the owning BLOCK's closed/interrupted/unclosed state. Empty
ATTDEF records still receive all 24 absent cards. Both group-280 occurrences
remain members of one neutral `VersionOrLockPosition` card.

The directory owns its complete evidence layer, preserves source identity,
checks cancellation throughout construction, bounds card/member ordinals to
compact integer domains, and provides allocation-free record, role, card,
member, and underlying-value lookups after construction.

## Test evidence

`crates/seacad-dxf-core/tests/block_attribute_definition_card_tests.rs` covers
ASCII/Binary parity across all nine supported dialects; exact 24-role order;
all-unique classic fields; absent legacy and duplicate modern group `280`;
multiple records; 24 absent cards for an empty ATTDEF; duplicate text/double
roles; invalid ASCII values with unchanged cardinality; source-order member
references; record locality; cancellation; source identity; bounded misses;
and public traits.

## Non-claims

M10.1y does not select or decode a canonical value, apply defaults,
distinguish group-280 meanings by order, validate text/numeric domains,
interpret flags or justification, decode MText extensions, compare ATTRIB
tags, associate inserted attributes, transform geometry, edit, write, or
render.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, generated-schema
drift, formatting, workspace Clippy with warnings denied, 529 workspace tests
with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/block_attribute_definition_card.rs` | 340 | `15031175b9915276bda500cb4d99b56440582dbc09ce5d87b4f4a1757cebc816` |
| `crates/seacad-dxf-core/src/lib.rs` | 557 | `15e0865bda89ad2c3cc32d369a6a8ab9a7609b146d0bce7f7dfd24d58f588fb0` |
| `crates/seacad-dxf-core/tests/block_attribute_definition_card_tests.rs` | 372 | `c5ec02fba4fdb4f81551c30b642f5e818697fa01499985049656fae93f541285` |
| `docs/IMPLEMENTATION_PLAN.md` | 1023 | `e7ce1fc19404ae3c0f2c5fff015ad3ff4709e17418ebe32a5fa6653320319ef8` |
| `docs/SUPPORT_MATRIX.md` | 848 | `ce3346d2ad9a255fa2c6081fc3e22c0aac9f876f4eea5a0ca5d3fd90ba5b7d35` |
