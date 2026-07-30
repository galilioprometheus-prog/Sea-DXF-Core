# M10.1ac Classic ATTDEF Justification

Retrieved: 2026-07-30

## Normative boundary

Autodesk [ATTDEF
(DXF)](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-F0EA099B-6F88-4BCC-BEC7-247BA64838A4.htm)
assigns classic horizontal justification to group `72`, vertical justification
to group `74`, and says the alignment point is meaningful only when either
value is nonzero. Autodesk [TEXT
(DXF)](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-62E5383D-8A14-47B4-BFC4-35824CAE8363.htm)
publishes horizontal codes `0..5` and vertical codes `0..3`, which ATTDEF
references.

## Implementation contract

`crates/seacad-dxf-core/src/block_attribute_definition_justification.rs`
lazily classifies usable M10.1ab values as:

- horizontal left, center, right, aligned, middle, or fit; and
- vertical baseline, bottom, middle, or top.

Typed semantic state preserves whether the original code was explicit or
defaulted, along with exact raw provenance when present. Codes outside the
published domains become typed invalid values. Underlying ASCII/cardinality
failures are retained as nested typed issues.

Alignment-point applicability is `true` when either classified code is
nonzero, `false` only when both classified codes are usable zero values, and
unavailable otherwise. This is applicability evidence only; it does not select
or validate a coordinate tuple.

Directory construction preserves source identity and reuses the cancellable,
bounded M10.1ab integer directory. Raw-record, exact-entry, and BLOCK-local
ATTDEF lookups do not allocate payload-sized buffers.

## Test evidence

`crates/seacad-dxf-core/tests/block_attribute_definition_justification_tests.rs`
covers ASCII/Binary parity across all nine supported dialects; every published
horizontal and vertical code; explicit and defaulted states; zero/nonzero
alignment applicability; unsupported signed codes; nested invalid ASCII
evidence; raw provenance; raw-record, exact-entry, and BLOCK-local lookups;
cancellation; source identity; bounds; and public traits.

## Non-claims

M10.1ac does not validate horizontal/vertical combinations, require or select
text-start/alignment tuples, recalculate placement, measure styled text, decode
MText extensions, compare ATTRIB tags, associate inserted attributes,
transform geometry, edit, write, or render.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, generated-schema
drift, formatting, workspace Clippy with warnings denied, 545 workspace tests
with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/block_attribute_definition_justification.rs` | 286 | `fbdf7f2264cb11fcc699cfbb1505f51e1394d7c98165e0204b3ab57927533c5f` |
| `crates/seacad-dxf-core/src/lib.rs` | 585 | `678cf4744210fc3aacc2c10c55b8c3945ca4f051119ed814ca0afa7a46f4a02b` |
| `crates/seacad-dxf-core/tests/block_attribute_definition_justification_tests.rs` | 347 | `c5e4a24688dc6248125d6737d1b52404399aeaec9cafe5ff9008d4d5ffdcc8fd` |
| `docs/IMPLEMENTATION_PLAN.md` | 1080 | `14a2ddb1059bff843875e2a0625c33647d6598c30f39be8886a07cf32d759455` |
| `docs/SUPPORT_MATRIX.md` | 894 | `1fa2778a1dfe3cdd6d503f66db56ea9b3c42ba766d600af52812de6f8895be0f` |
