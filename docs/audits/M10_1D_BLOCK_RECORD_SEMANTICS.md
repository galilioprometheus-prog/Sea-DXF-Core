# M10.1d BLOCK Record Semantics

Retrieved: 2026-07-30

## Normative boundary

Autodesk [BLOCK
(DXF)](https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-66D32572-005A-4E23-8B8B-8726E8C14302.htm)
publishes names `2/3`, flags `70`, and base point `10/20/30` without an
optional marker or default. It marks description `4` optional and defines seven
independent flag bits; bits `32` and `64` are ignored on input but remain exact
source evidence.

Autodesk [About BLOCKS Section Group Codes
(DXF)](https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-9DDFC343-6C87-4AF8-B3B9-77E0AB5A3031.htm)
states that external-reference definitions additionally include group `1` for
the path and filename. M10.1d therefore keeps an omitted xref path absent rather
than requiring it for every ordinary block or inventing an empty default.

## Implementation contract

`crates/seacad-dxf-core/src/block_record_semantic.rs` lazily projects every
M10.1c card. Primary/secondary names, flags, and base-point components are
required. Xref path and description remain optional. Unique values retain exact
text/double/i16 evidence; missing required values, invalid ASCII numbers, and
duplicates fail typed with provenance. Seven flag helpers expose exact bits,
and a base-point tuple is available only when all three components are usable.

## Test evidence

`crates/seacad-dxf-core/tests/block_record_semantic_tests.rs` covers
ASCII/Binary parity across all nine supported dialects, exact text and
negative-zero preservation, all seven flag helpers, base-point assembly,
missing required fields, optional absence, explicit empty text, duplicate text,
invalid doubles/i16, per-field independence, every M10.1a definition state,
lookups, cancellation, source identity, and public traits.

## Non-claims

M10.1d does not require path presence from xref flag bit `4`; reconcile names;
validate empty text or numeric finiteness; resolve xrefs; bind INSERT; transform
member geometry; edit; write; or render.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, formatting,
generated-schema drift, workspace Clippy with warnings denied, 442 workspace
tests with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/block_record_semantic.rs` | 497 | `b75ae7b2ee71923f0628b5c5e450b10c7fdaa277270dc8d7c5fb640728f87f78` |
| `crates/seacad-dxf-core/src/lib.rs` | 445 | `5fa305cceadfa18d0fefb3c58a11e4b5e97f928c8dd8317ffb7165fbb71b4a45` |
| `crates/seacad-dxf-core/tests/block_record_semantic_tests.rs` | 397 | `2cd1f7209bff62b8e90c0ce7fcf733c6a1c29904e05b676e8b5dedd55a1521da` |
| `docs/IMPLEMENTATION_PLAN.md` | 758 | `d4d86056f8d3626ede2d1bb36e3757b9f6163e419725adcbdb6386c7a4ed6fbf` |
| `docs/SUPPORT_MATRIX.md` | 645 | `3f638c417a6f5725530e8b41a083a1331fc764b3f423267dce855d4620e1188f` |
