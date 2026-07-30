# M10.1p Classic ATTRIB Cardinality

Retrieved: 2026-07-30

## Policy boundary

M10.1p adds cardinality only to the 23 classic ATTRIB roles established by
M10.1o. Autodesk's normative role inventory remains recorded in
`M10_1O_INSERT_ATTRIBUTE_VALUES.md`; this checkpoint does not reinterpret or
expand it.

## Implementation contract

`crates/seacad-dxf-core/src/insert_attribute_card.rs` publishes exactly 23
cards for every M10.1o ATTRIB record in stable classic role order. Each card:

- reports `Absent`, `Unique`, or duplicate-preserving `Multiple`;
- owns a compact half-open member range; and
- links every member to its exact source-order M10.1o value occurrence.

Cardinality is independent of ASCII lexical validity, sequence boundary state,
and sequence-local record ordinal. Empty ATTRIB records still receive 23 absent
cards. Both group-280 occurrences remain one neutral multiple card, so
cardinality does not invent version/lock meaning from order.

Directory allocation is bounded by the already-indexed attribute record/value
counts, reserves the fixed card product fallibly, checks arithmetic, preserves
source identity, and observes cancellation throughout construction.

## Test evidence

`crates/seacad-dxf-core/tests/insert_attribute_card_tests.rs` covers
ASCII/Binary parity across all nine supported dialects; stable 23-role order;
unique, absent, and multiple states; empty ATTRIB records; two group-280
members; duplicate text/numeric roles; invalid ASCII numbers counted as exact
occurrences; source-order member references; sequence-record locality;
cancellation; source identity; bounded lookups; and public traits.

## Non-claims

M10.1p does not select or decode a canonical value, apply defaults,
distinguish group-280 meanings, interpret flags or justification, decode MText
extensions, associate ATTDEF definitions, transform attributes, edit, write,
or render.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, formatting,
generated-schema drift, workspace Clippy with warnings denied, 494 workspace
tests with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/insert_attribute_card.rs` | 332 | `ddbea2f16b2e0fa8c94cec8a8a84b8ef46c83af51ec653823bb1b5457df1e0c8` |
| `crates/seacad-dxf-core/src/lib.rs` | 507 | `0f3509782112b452ac17405561911221541ced21ed55cc281ce30968f722cf8f` |
| `crates/seacad-dxf-core/tests/insert_attribute_card_tests.rs` | 366 | `a502f6517900ccd0892f318d4ed0d92c127c9b0eeab0cb19a903a51f44b39540` |
| `docs/IMPLEMENTATION_PLAN.md` | 899 | `0ccaeaf0918e756ae0576455d4adfd47e618a6a4efce569cfb4e2bd74bf1f138` |
| `docs/SUPPORT_MATRIX.md` | 759 | `9852e91e9e34826aef9265f3e26504df2aa226f0afdd31cabe5c4b4215eb40f9` |
