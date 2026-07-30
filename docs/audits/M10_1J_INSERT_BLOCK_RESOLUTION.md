# M10.1j INSERT Block Resolution

Retrieved: 2026-07-30

## Normative boundary

Autodesk [INSERT
(DXF)](https://help.autodesk.com/cloudhelp/2018/ENU/AutoCAD-DXF/files/GUID-28FA4CFB-9D5E-4880-9F11-36C97578252F.htm)
identifies group `2` as the INSERT block name. M10.1j relates that exact usable
source value to the matched BLOCK-name evidence established through M10.1f.

## Implementation contract

`crates/seacad-dxf-core/src/insert_block_resolution.rs` publishes one entry per
M10.1i INSERT semantic record:

- `UnusableName` for missing or duplicate name semantics;
- `Missing` when no exact indexed BLOCK name matches;
- `Unique` for one exact target;
- `Ambiguous` with every duplicate target retained in source-record order.

`crates/seacad-dxf-core/src/block_name_index.rs` adds exact same-document
source-span lookup. It hashes and confirms source bytes in fixed 4-KiB chunks,
so INSERT resolution does not allocate a buffer proportional to an untrusted
name. Exact comparison remains authoritative after digest narrowing.

## Test evidence

`crates/seacad-dxf-core/tests/insert_block_resolution_tests.rs` covers
ASCII/Binary parity across all nine supported dialects, unique/ambiguous/
missing/unusable states, duplicate ordering, exact case and empty-name
behavior, a 4-KiB-boundary-crossing name, conflicting/unindexable BLOCK
exclusion, bounded lookups, cancellation, source identity, and public traits.

## Non-claims

M10.1j does not require targets to be closed definitions, choose an ambiguous
target, detect recursive references, follow ATTRIB/SEQEND, form an OCS
transform, edit, write, or render.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, formatting,
generated-schema drift, workspace Clippy with warnings denied, 467 workspace
tests with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/block_name_index.rs` | 487 | `16e00a64a90ec443df158e61e8ec32a7ec95e03781455c0238ae9c6f617d6d65` |
| `crates/seacad-dxf-core/src/insert_block_resolution.rs` | 261 | `3d15c379f148fa0fba14df055647a2f45eef3fa80c17ea22eab9d6bc98281e64` |
| `crates/seacad-dxf-core/src/lib.rs` | 475 | `040ffa0dbbddfa98544bda6fef11f28031326687ca6d894fed3f40207f09ebfe` |
| `crates/seacad-dxf-core/tests/insert_block_resolution_tests.rs` | 251 | `f449aefa04177fa5e1d5682afc9f739c5288cffe69077defbabfb16c9c75793f` |
| `docs/IMPLEMENTATION_PLAN.md` | 823 | `06d8f88ee32bd6a148248075573caf9b7539e19c1764f763cc98fdddcbec72e5` |
| `docs/SUPPORT_MATRIX.md` | 701 | `7819b07abfe23a484f3498be75be3a3cfd7c52440ac1e479400a1103fe6e88a2` |
