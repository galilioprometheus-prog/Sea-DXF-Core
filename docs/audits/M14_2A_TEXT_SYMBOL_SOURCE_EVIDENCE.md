# M14.2a Text-and-Symbol Source Evidence

Retrieved: 2026-07-31

## Normative boundary

Autodesk's [TEXT
reference](https://help.autodesk.com/cloudhelp/2016/ENU/AutoCAD-DXF/files/GUID-62E5383D-8A14-47B4-BFC4-35824CAE8363.htm),
[MTEXT
reference](https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-5E5DB93B-F8D3-4433-ADF7-E92E250D2BAB.htm),
[SHAPE
reference](https://help.autodesk.com/cloudhelp/2025/ENU/AutoCAD-DXF/files/GUID-0988D755-9AAB-4D6C-8E26-EC636F507F2C.htm),
and [TOLERANCE
reference](https://help.autodesk.com/cloudhelp/2018/ENU/AutoCAD-DXF/files/GUID-ADFCED35-B312-4996-B4C1-61C53757B3FD.htm)
define the per-entity group roles retained by M14.2a. Autodesk's [common entity
reference](https://help.autodesk.com/cloudhelp/2015/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm)
also defines group 420/430 ranges for entity color properties. Those ranges
therefore remain explicitly ambiguous on MTEXT until M14.11 adds
subclass-aware common-property ownership.

Autodesk assigns MTEXT group 50 both to rotation and repeated column heights.
M14.2a retains one `RotationOrColumnHeight` role and does not infer meaning
from incidental record position.

## Legacy evidence boundary

`D:\SeaCad\cad_2026-07-23_source` was consulted only as a read-only behavioral
oracle. Its Q4.2 evidence reports repeated MTEXT chunks, group-50 ambiguity,
layout/background/column fields, 2,354 target-scope TEXT/MTEXT source records,
446 generation-flag occurrences, 35 column occurrences, and explicit invalid
source cases. Its inventories also keep SHAPE and TOLERANCE as open families.
No legacy parser, validator, fixture, or production implementation was copied,
translated, vendored, or linked. Autodesk documentation remains normative.

## Implementation contract

`text_symbol_evidence.rs` exposes one immutable, source-bound directory for
exact uppercase TEXT, MTEXT, SHAPE, and TOLERANCE markers inside completely
closed BLOCKS and ENTITIES sections. Each record owns a source-order slice of
documented fields. Every field retains its raw group/span and one exact wire
domain:

- raw text, without decoding or allocating payload copies;
- binary64 with exact IEEE bits;
- signed 16-bit integer; or
- signed 32-bit integer.

Duplicates and invalid ASCII numbers remain separate occurrences. Lookup is
bounded by raw record or group occurrence; source identity and cancellation
are checked.

`text_symbol_role.rs` is the static per-family Autodesk role registry. It
freezes every supported role/wire mapping, keeps MTEXT group 50 ambiguous, and
labels group 420/430 collisions without claiming subclass ownership.

## Test evidence

`text_symbol_evidence_tests.rs` covers ASCII/Binary parity across all nine
AC1009--AC1032 dialects, every field whose group code fits the reviewed AC1009
one-byte Binary encoding, all extended group-code fields in post-R12 Binary
DXF, all four exact entity markers, BLOCKS/ENTITIES scope, repeated group
1/3/50 values, byte-exact text, binary64 bit identity, signed integer domains,
invalid and duplicate values, unrelated group rejection, lookup bounds, source
identity, cancellation, and public `Copy`/`Send`/`Sync` bounds.

The private role-registry unit test independently freezes all documented
family mappings, range endpoints, collisions, and ignored cross-family codes.

## Non-claims

M14.2a does not choose cardinality, merge MTEXT chunks, disambiguate group 50,
assign common-property subclass ownership, apply defaults, decode text/style
bytes, resolve STYLE or DIMSTYLE records, validate flags/layout/columns,
interpret inline MTEXT controls, transform OCS/WCS coordinates, shape glyphs,
edit, or write. It does not claim semantic completion for these four families
or for all currently recognized DXF entities.

## Required checkpoint gates

All required gates passed on 2026-07-31: dependency policy, generated-schema
drift, release-evidence drift, formatting, workspace Clippy with warnings
denied, 621 workspace tests with zero failures/ignored tests, and
`git diff --check`. No dependency manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 78 | `d96c6c3a7246d36c6fd2c57db39052b2f6909792171fe98662b3e33e58d3a4eb` |
| `crates/seacad-dxf-core/src/lib.rs` | 655 | `5f8c00e0ad9aa7ca397a635d58979f5671af16256b74af5dc5418c05eb5919c3` |
| `crates/seacad-dxf-core/src/text_symbol_evidence.rs` | 396 | `70bf116abc90c82e9dd1b3548e5813709e8be4da131bcf9b2dbc55d13453c21e` |
| `crates/seacad-dxf-core/src/text_symbol_role.rs` | 246 | `dc8a7085892af1c759a678bb7187f1234164c2749ca36fb49eba21b249bb3ff3` |
| `crates/seacad-dxf-core/tests/text_symbol_evidence_tests.rs` | 461 | `4932ed656d3bd115a5bf8d82418234f7d607b0cb15c7fcdc35c6844eacbb56aa` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 107 | `e1c509a6634ba8edb38350fb51efbc5e6a18179ff5f5638fc8e810612db4c2b3` |
| `docs/IMPLEMENTATION_PLAN.md` | 1398 | `82452e0486e8ea6b0ec2fcba154f1299d01bac65f14a281b8b6c837d557101a7` |
| `docs/SUPPORT_MATRIX.md` | 1119 | `21b85f96ece685798a619d72102276cbefeb5923aabbad9b1ac54c7c548b7e7a` |
