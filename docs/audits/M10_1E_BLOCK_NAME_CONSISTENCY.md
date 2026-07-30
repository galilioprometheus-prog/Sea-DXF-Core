# M10.1e BLOCK Name Consistency

Retrieved: 2026-07-30

## Normative boundary

Autodesk [BLOCK
(DXF)](https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-66D32572-005A-4E23-8B8B-8726E8C14302.htm)
describes both group `2` and group `3` as the block name. M10.1e compares them
without silently choosing either occurrence or claiming a repair policy.

Because raw source bytes remain authoritative, comparison uses exact
same-document bytes. It does not decode, case-fold, normalize, or infer that
distinct spellings name the same block.

## Implementation contract

`crates/seacad-dxf-core/src/block_name_consistency.rs` publishes one entry per
M10.1d BLOCK semantic record:

- `Matched` when both usable name payloads are byte-exact;
- `Conflicting` when both are usable but differ;
- `NotComparable` when either semantic name is unavailable.

Comparison uses fixed 4-KiB buffers and cancellation checks, so memory does not
grow with an untrusted retained value.

## Test evidence

`crates/seacad-dxf-core/tests/block_name_consistency_tests.rs` covers
ASCII/Binary parity across all nine supported dialects, matched/conflicting/
not-comparable states, empty names, case differences, exact UTF-8 bytes,
duplicates, missing names, values crossing the internal comparison-chunk
boundary, every M10.1a definition state, source identity, bounded lookups,
cancellation, and public traits.

## Non-claims

M10.1e does not choose a canonical name, reject or repair conflicts, build a
name index, resolve INSERT/xrefs, apply case-insensitive policy, edit, write, or
render.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, formatting,
generated-schema drift, workspace Clippy with warnings denied, 447 workspace
tests with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/block_name_consistency.rs` | 230 | `7df687aab1db029844f015d122f94a222fac726e93351dee2a0f944729ca2c85` |
| `crates/seacad-dxf-core/src/lib.rs` | 449 | `78597d75a17ade51a581c80b0430cbfadaa51cd4ce00ebb9d971f0f9e1c1d3ea` |
| `crates/seacad-dxf-core/tests/block_name_consistency_tests.rs` | 247 | `18340e889187b80ed9f3252f686152f6678e33d9b90908474e7e373b8bb38b2d` |
| `docs/IMPLEMENTATION_PLAN.md` | 769 | `5230d0e7034fafee24a3ee78352289f5f636a4f21b402741097d562b153718d7` |
| `docs/SUPPORT_MATRIX.md` | 654 | `cee87ffb126a7d0c60e77ec075f3dc5913e852662ad660c44200a3ad0251db89` |
