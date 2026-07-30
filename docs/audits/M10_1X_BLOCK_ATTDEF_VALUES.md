# M10.1x Classic ATTDEF Defining Values

Retrieved: 2026-07-30

## Normative boundary

Autodesk [ATTDEF
(DXF)](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-F0EA099B-6F88-4BCC-BEC7-247BA64838A4.htm)
publishes the classic attribute-definition fields: OCS text placement,
default/prompt/tag text, attribute and text flags, text metrics and style,
justification, extrusion, and repeated group `280` meanings. The same page
places the MText extension after the `AcDbXrecord` subclass marker.

M10.1x retains exact wire evidence for the classic fields without selecting,
defaulting, or interpreting them.

## Implementation contract

`crates/seacad-dxf-core/src/block_attribute_definition_value.rs` layers value
evidence over every exact M10.1w ATTDEF record. Twenty-four stable roles cover:

- 14 binary64 roles for placement, metrics, alignment, and extrusion;
- four source-anchored text roles for default value, prompt, tag, and style;
- five unambiguous signed-16-bit flag/field/justification roles; and
- one neutral signed-16-bit `VersionOrLockPosition` role for group `280`.

Every occurrence remains in source order. ASCII numeric failures are typed;
Binary doubles retain exact bits; text retains its source span and the
document's encoding resolution and decodes only on explicit bounded request
without replacement.

Legacy payload is admitted directly. Modern subclass tracking admits
`AcDbText` and `AcDbAttributeDefinition`, ignores unrelated contexts, and
stops at `AcDbXrecord` so extension/MText fields cannot impersonate classic
roles. Content enclosed by group `102` application controls is excluded.

The directory owns its M10.1w topology and application-group evidence,
preserves source identity, checks cancellation throughout construction, and
provides allocation-free exact record/group lookups after construction.

## Test evidence

`crates/seacad-dxf-core/tests/block_attribute_definition_value_tests.rs`
covers ASCII/Binary parity across all nine supported dialects; all 24 roles;
both modern group-280 occurrences; legacy and modern subclass layouts; multiple
ATTDEF records; records outside definitions; duplicate values; invalid ASCII
double/int16 values; application-group exclusion; unrelated subclass
exclusion; the AcDbXrecord boundary; exact text decoding and source mismatch;
cancellation; source identity; bounded misses; and public traits.

## Non-claims

M10.1x does not assign cardinality, select a canonical occurrence, apply
defaults, distinguish the two group-280 meanings by position, validate
text/numeric domains, interpret flags or justification, decode MText
extensions, compare ATTRIB tags, associate inserted attributes, transform
geometry, edit, write, or render.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, generated-schema
drift, formatting, workspace Clippy with warnings denied, 526 workspace tests
with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/block_attribute_definition_value.rs` | 535 | `1c2c19fcb7392f59a7757e11460d5becd66ce358fde330f7899a7710e5548fa4` |
| `crates/seacad-dxf-core/src/lib.rs` | 551 | `b1eac11670cd09b0c524ff855aa3e7371ea9c5d8e831ee456407690ca520d449` |
| `crates/seacad-dxf-core/tests/block_attribute_definition_value_tests.rs` | 397 | `ef0add3f04171d73cf1ea7142e6544a078f2f2f7b069ab55e212f16c95494cc0` |
| `docs/IMPLEMENTATION_PLAN.md` | 1011 | `e672526b1a12fabda6ce3d4f5f247292494fb6d94d76863b18213e9fbbb71a50` |
| `docs/SUPPORT_MATRIX.md` | 839 | `c803f5ff7064be12c0c4d3e5db574d0f958d5db497a0844b14bd8869028f8771` |
