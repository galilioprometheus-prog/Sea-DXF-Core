# M10.1f BLOCK Name Index

Retrieved: 2026-07-30

## Normative boundary

Autodesk [BLOCK
(DXF)](https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-66D32572-005A-4E23-8B8B-8726E8C14302.htm)
labels both group `2` and group `3` as the block name. M10.1f indexes a record
only after M10.1e proves those two usable payloads byte-exact.

The index is evidence, not a claim that every indexed spelling is a valid
AutoCAD namespace member or that a duplicate has a preferred target.

## Implementation contract

`crates/seacad-dxf-core/src/block_name_index.rs` retains the complete M10.1e
consistency directory and publishes:

- one lookup match for each `Matched` BLOCK record;
- exact raw-byte lookup states `Missing`, `Unique`, and `Ambiguous`;
- every duplicate target in source-record order within its exact-name group;
- source-identity and cancellation checks for construction and lookup.

Names are digested from the immutable source in fixed 4-KiB chunks without a
name-sized allocation. SHA-256 narrows candidates, while exact bounded
source-byte comparison remains authoritative and prevents a digest collision
from creating a false match.

## Test evidence

`crates/seacad-dxf-core/tests/block_name_index_tests.rs` covers ASCII/Binary
parity across all nine supported dialects, missing/unique/ambiguous lookup,
duplicate ordering, empty/case-distinct/UTF-8 names, a name crossing the 4-KiB
boundary, conflicting and unusable exclusion, all M10.1a definition states,
cancellation, source mismatch, and public bounds.

## Non-claims

M10.1f does not validate a legal block namespace, choose among duplicates,
decode/case-fold/normalize names, resolve INSERT or xrefs, transform member
geometry, edit, write, or render.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, formatting,
generated-schema drift, workspace Clippy with warnings denied, 452 workspace
tests with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/block_name_index.rs` | 435 | `0691847b10462af9847bae638ea4f4f428a20fc7b6ea3f63935d64cc15aea1e6` |
| `crates/seacad-dxf-core/src/lib.rs` | 453 | `f72c664d60fb166dec130a999daad85fb916519b11788deb66270adf91784b52` |
| `crates/seacad-dxf-core/tests/block_name_index_tests.rs` | 313 | `79698a4fde450bb4cca6b74bcab7a80a4b994c9830e6f51f0adc63e381db1c81` |
| `docs/IMPLEMENTATION_PLAN.md` | 780 | `57d5b5ebe91c244eaa2478d39c32de766ff860b335da88b7d43e847458d3bd3b` |
| `docs/SUPPORT_MATRIX.md` | 665 | `6ebe0d2153ba9fe9911d63e2a7494bca4c92bb872f325be89f05019cb4eaecd0` |
