# M10.1c BLOCK Record Cardinality

Retrieved: 2026-07-30

## Normative boundary

Autodesk [BLOCK
(DXF)](https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-66D32572-005A-4E23-8B8B-8726E8C14302.htm)
documents the eight defining roles retained by M10.1b: names `2/3`, flags `70`,
base point `10/20/30`, xref path `1`, and optional description `4`. It does not
authorize silently choosing among duplicate occurrences.

M10.1c therefore adds neutral occurrence-count evidence before any canonical
selection, defaulting, flag interpretation, or point assembly.

## Implementation contract

`crates/seacad-dxf-core/src/block_record_card.rs` creates:

- eight fixed cards in stable documented-role order for every M10.1b BLOCK;
- `Absent`, `Unique`, or `Multiple` state independent of text/numeric validity;
- compact source-order members referring back to exact M10.1b value ordinals;
- the same card shape for closed, interrupted, and unclosed definitions.

## Test evidence

`crates/seacad-dxf-core/tests/block_record_card_tests.rs` covers ASCII/Binary
parity across all nine supported dialects, stable card order, all eight roles,
duplicate text, exact text/double/i16 member resolution, unique invalid numbers,
multiple invalid numbers, application/member/ENDBLK locality inherited from
M10.1b, empty BLOCK records, every M10.1a definition state, bounded lookups,
cancellation, source identity, and public traits.

## Non-claims

M10.1c does not select or decode canonical values; reconcile names; apply
defaults; interpret flags; assemble points; resolve xrefs; bind INSERT;
transform geometry; edit; write; or render.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, formatting,
generated-schema drift, workspace Clippy with warnings denied, 438 workspace
tests with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/block_record_card.rs` | 317 | `ed6a33febd0ed75376f018c1c738a49d989c66f79f884d531296270860638d99` |
| `crates/seacad-dxf-core/src/lib.rs` | 440 | `a8d5eae500e6b3471c4139f75fd2cd77d0f1e60d9780f5879523cb774332f40a` |
| `crates/seacad-dxf-core/tests/block_record_card_tests.rs` | 378 | `f0be1328e354872e5ae8f9d90d29925aef99149c1240363689bb9a553bc75cb5` |
| `docs/IMPLEMENTATION_PLAN.md` | 745 | `7d2d46b4fd4615e5470848b4870c98064d9d7b3261031abc54dccf352cbc6a0a` |
| `docs/SUPPORT_MATRIX.md` | 634 | `3e2de89ce1dc7bc770d77c1b3bd51f4e9d7360a30e2128c1708ae8d8c4adec65` |
