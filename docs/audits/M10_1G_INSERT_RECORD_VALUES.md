# M10.1g INSERT Record Values

Retrieved: 2026-07-30

## Normative boundary

Autodesk [INSERT
(DXF)](https://help.autodesk.com/cloudhelp/2018/ENU/AutoCAD-DXF/files/GUID-28FA4CFB-9D5E-4880-9F11-36C97578252F.htm)
documents the block name, insertion point, scale factors, rotation, array
counts/spacing, attributes-follow flag, and extrusion direction used by M10.1g.

This checkpoint retains exact defining-value evidence. It does not apply the
optional defaults documented by Autodesk.

## Implementation contract

`crates/seacad-dxf-core/src/insert_record_value.rs` discovers exact uppercase
`INSERT` markers in completely indexed BLOCKS and ENTITIES sections and
retains, in source order:

- block name `2`;
- insertion point `10/20/30`;
- scale factors `41/42/43` and rotation `50`;
- column/row counts `70/71` and spacing `44/45`;
- attributes-follow `66`;
- extrusion direction `210/220/230`.

Text remains source-backed. Double and signed-16-bit values preserve exact
binary domains or explicit ASCII lexical failures. Duplicates remain distinct,
and nested group `102` application content is excluded.

## Test evidence

`crates/seacad-dxf-core/tests/insert_record_value_tests.rs` covers ASCII/Binary
parity across all nine supported dialects, all 16 roles, BLOCKS and ENTITIES
records, exact marker/section filtering, duplicate and invalid values,
application-group exclusion, source-backed text decoding, lookups,
cancellation, and public bounds.

## Non-claims

M10.1g does not apply defaults, select duplicate values, assemble typed INSERT
semantics, resolve the block name, follow ATTRIB/SEQEND, transform member
geometry, edit, write, or render.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, formatting,
generated-schema drift, workspace Clippy with warnings denied, 456 workspace
tests with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/insert_record_value.rs` | 470 | `d333a5f46c94a09b170849071877ec1250219b8c25b4134ec616de1954a0935b` |
| `crates/seacad-dxf-core/src/lib.rs` | 459 | `d816857e95392beca961a248539be031f6a303a6f7dac5067715beee2fd45bb4` |
| `crates/seacad-dxf-core/tests/insert_record_value_tests.rs` | 331 | `5a31dedb83128cf851041ce474e37730640800690c61fd393dc696bb6632be89` |
| `docs/IMPLEMENTATION_PLAN.md` | 792 | `4dad643a4e04cb1a7271cd0d9cb82c96ae09a6891f32e1a5aebc1bd9ed01068c` |
| `docs/SUPPORT_MATRIX.md` | 675 | `deed10bccb639b422c49d74dafa625d334166a9bf16cc9ca46604941e90d3893` |
