# M10.1ab Classic ATTDEF Integer Semantics

Retrieved: 2026-07-30

## Normative boundary

Autodesk [ATTDEF
(DXF)](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-F0EA099B-6F88-4BCC-BEC7-247BA64838A4.htm)
defines group `70` as required attribute flags; optional group `73` field
length, group `71` text-generation flags, group `72` horizontal justification,
and group `74` vertical justification each default to zero. The four published
attribute bits are invisible `1`, constant `2`, verification-required `4`, and
preset `8`. The TEXT definitions used by ATTDEF publish backward `2` and
upside-down `4` as text-generation bits.

The ATTDEF contract assigns two meanings to group `280`. M10.1ab therefore
leaves that wire code as neutral M10.1y card evidence rather than selecting a
version or lock-position meaning.

## Implementation contract

`crates/seacad-dxf-core/src/block_attribute_definition_integer_semantic.rs`
lazily projects five M10.1y signed-16-bit cards:

- attribute flags are required;
- absent field length, text-generation flags, and horizontal/vertical
  justification become typed zero defaults without invented raw provenance;
- explicit values preserve the exact signed wire value, including unknown
  bits; and
- helpers test only the six published bits without changing the source value.

Invalid ASCII numbers, missing required flags, and duplicate cards remain
distinct typed invalid states. Duplicate failure retains the first
occurrence's raw provenance without selecting it as a semantic value.

The directory owns its M10.1y card/evidence chain, preserves source identity,
checks cancellation, and supports exact raw-record and BLOCK-local ATTDEF
ordinal lookup without payload-sized construction buffers.

## Test evidence

`crates/seacad-dxf-core/tests/block_attribute_definition_integer_semantic_tests.rs`
covers ASCII/Binary parity across all nine supported dialects; exact values and
published bit helpers; preservation of an unknown attribute bit; missing
required flags versus four zero defaults; invalid ASCII; duplicates and
first-occurrence provenance; neutral duplicate group `280` evidence;
raw-record and BLOCK-local definition lookups; cancellation; source identity;
bounded misses; and public traits.

## Non-claims

M10.1ab does not validate field-length or justification ranges, reject unknown
flag bits, classify justification or alignment-point applicability,
distinguish the two group-280 meanings, decode MText extensions, compare ATTRIB
tags, associate inserted attributes, transform geometry, edit, write, or
render.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, generated-schema
drift, formatting, workspace Clippy with warnings denied, 541 workspace tests
with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/block_attribute_definition_integer_semantic.rs` | 407 | `11ebcec5e7d047015e8a66433a2a58bf134ac7383c86fb3d086dbf2f569f9185` |
| `crates/seacad-dxf-core/src/lib.rs` | 575 | `46a2ff1adbb227413bb36c7a8c7530005cae5a2338b6385f61b268ad209f6d9f` |
| `crates/seacad-dxf-core/tests/block_attribute_definition_integer_semantic_tests.rs` | 307 | `54d3a240e57b3c049eba8894cef78e1823db79eefdfcac256cc9c38a26251f9a` |
| `docs/IMPLEMENTATION_PLAN.md` | 1067 | `bf61590065b0be7192cbb375a299b06fcd3bce011ed7407b21432fa57242d935` |
| `docs/SUPPORT_MATRIX.md` | 883 | `9a0ec32c606979cebf5f2f29763d875aa061b40474eeffaee7b53ba6043b587d` |
