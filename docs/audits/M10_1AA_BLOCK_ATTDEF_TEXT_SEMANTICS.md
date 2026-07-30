# M10.1aa Classic ATTDEF Text Semantics

Retrieved: 2026-07-30

## Normative boundary

Autodesk [ATTDEF
(DXF)](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-F0EA099B-6F88-4BCC-BEC7-247BA64838A4.htm)
defines group `1` as the default value, group `3` as the prompt, group `2` as
the tag string, and optional group `7` as the text-style name with default
`STANDARD`.

M10.1aa applies only that documented style default and keeps the three defining
strings required and source anchored.

## Implementation contract

`crates/seacad-dxf-core/src/block_attribute_definition_text_semantic.rs`
lazily projects four M10.1y text cards:

- default value, prompt, and attribute tag are required;
- an explicit style is source anchored; and
- an absent style becomes typed `STANDARD` default data without invented raw
  provenance.

Missing required cards and duplicate cards remain distinct typed invalid
states. Duplicate failure retains the first occurrence's raw provenance without
selecting it as the semantic value. Explicit text retains its exact M10.1x
source span and encoding resolution and decodes only through an explicit
caller buffer, tied to the same source identity and without replacement.

The directory owns its M10.1y card/evidence chain, preserves source identity,
checks cancellation, and supports exact raw-record and BLOCK-local ATTDEF
ordinal lookup without payload-sized construction buffers.

## Test evidence

`crates/seacad-dxf-core/tests/block_attribute_definition_text_semantic_tests.rs`
covers ASCII/Binary parity across all nine supported dialects; explicit
default/prompt/tag/style text; missing required fields; the `STANDARD` default;
duplicate counts and first-occurrence provenance for all four roles; bounded
replacement-free decoding; raw-record and BLOCK-local definition lookups;
cancellation; source identity; bounded misses; and public traits.

## Non-claims

M10.1aa does not reject empty values or spaces in tags, resolve style-table
names, interpret text formatting or escapes, project ATTDEF integer fields,
decode MText extensions, compare ATTRIB tags, associate inserted attributes,
transform geometry, edit, write, or render.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, generated-schema
drift, formatting, workspace Clippy with warnings denied, 537 workspace tests
with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/block_attribute_definition_text_semantic.rs` | 397 | `6c177b019879a4f3c37d5a43a034c72537c0cd88ecdc9d4968b6981f0b318094` |
| `crates/seacad-dxf-core/src/lib.rs` | 569 | `8efd194e9b4c379da43f29e5b17db2c8e0fbc0216ffee437f556051a5dd2c179` |
| `crates/seacad-dxf-core/tests/block_attribute_definition_text_semantic_tests.rs` | 314 | `36e4fc94f99b0c50c8cee3f6fa52e3266de3105d3bf295ba1ccf0ad9defc8896` |
| `docs/IMPLEMENTATION_PLAN.md` | 1051 | `51dde3e18b554fa2226b582cdfd5c402b5cc7a22565cd2073cbf2496d955dbe5` |
| `docs/SUPPORT_MATRIX.md` | 869 | `0f4b5240feb4de4bd9fbe79c623a2b15d363b65a7b98579c21908a1e8e091ff2` |
