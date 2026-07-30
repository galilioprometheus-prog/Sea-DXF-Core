# M10.1b BLOCK Record Values

Retrieved: 2026-07-30

## Normative boundary

Autodesk [BLOCK
(DXF)](https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-66D32572-005A-4E23-8B8B-8726E8C14302.htm)
defines primary block name `2`, bit-coded flags `70`, base-point components
`10/20/30`, secondary block name `3`, xref path `1`, and optional description
`4`. The same page permits application-defined codes inside group-102 control
groups, so M10.1b excludes their payload occurrences from BLOCK field evidence.

Autodesk [Blocks Group Codes in DXF Files
(DXF)](https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-A3E4F1D3-79C9-489C-B7EC-3924DA7F25C9.htm)
places the BLOCK defining groups on the BLOCK record before its member entity
records and ENDBLK boundary.

## Implementation contract

`crates/seacad-dxf-core/src/block_record_value.rs` layers source-order evidence
over every M10.1a BLOCK record. Four text roles retain exact source spans,
source identity, and document encoding for explicit caller-buffer decoding
without replacement. Base-point roles retain exact binary64 bits and flags
retain exact signed-i16 values. Duplicate occurrences, empty text, and ASCII
numeric failures remain visible. Application-control payloads, member entities,
and ENDBLK payloads cannot enter a BLOCK value slice.

## Test evidence

`crates/seacad-dxf-core/tests/block_record_value_tests.rs` covers ASCII/Binary
parity across all nine supported dialects, all eight documented roles, exact
negative-zero and signed-i16 retention, duplicates, empty text, typed numeric
failures, group-102 exclusion, member and ENDBLK locality, every M10.1a
definition state, text decoding, source-identity mismatch, lookups,
cancellation, and public traits.

## Non-claims

M10.1b does not select occurrences; reconcile groups `2/3`; apply defaults;
interpret flags; assemble the base point; resolve xref paths; decode member
entities; bind INSERT; transform geometry; edit; write; or render.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, formatting,
generated-schema drift, workspace Clippy with warnings denied, 434 workspace
tests with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/block_record_value.rs` | 434 | `94f5dacd07f33b865d55fbb60615cbb102106ce74a7856b62749196cf163deba` |
| `crates/seacad-dxf-core/src/lib.rs` | 435 | `0cc9bfed72b4849a7b6bbbee951a98a9a14993609d4ad5dccb155cce1bcfb3de` |
| `crates/seacad-dxf-core/tests/block_record_value_tests.rs` | 367 | `baa29f96c0cd28218d92e4e2292fc5f3de8fb3ded440c5add360899d1c5dd821` |
| `docs/IMPLEMENTATION_PLAN.md` | 735 | `b539f852f2c82322084a6f0e6d1af657b643d2946f2b6c02ed381a852681d818` |
| `docs/SUPPORT_MATRIX.md` | 625 | `31181d8af1b8ac4fec4a20a97e26573edb1337724fc33f9cf5d60714b6d1650c` |
