# M14.2b Text-and-Symbol Cardinality

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
define the fixed role sets counted by M14.2b.

MTEXT expressly permits repeated group 3 text chunks and reuses group 50 for
rotation and column heights. `Multiple` is therefore cardinality evidence, not
an automatic semantic error.

## Legacy evidence boundary

`D:\SeaCad\cad_2026-07-23_source` remains a read-only behavioral oracle. Its
Q4.2 notes identify repeated text chunks, group-50 ambiguity, columns, invalid
numeric inputs, and record-local field ownership as compatibility risks. No
legacy parser, card index, test fixture, or validator was copied, translated,
vendored, or linked. Autodesk documentation remains normative.

## Implementation contract

`text_symbol_card.rs` owns the M14.2a evidence directory and creates one stable
card for every documented role in every record:

- 19 cards for TEXT;
- 33 cards for MTEXT;
- 12 cards for SHAPE; and
- 11 cards for TOLERANCE.

Each card is `Absent`, `Unique`, or `Multiple { occurrence_count }`. Members
refer to exact M14.2a value ordinals; no raw value is copied. Cardinality stays
independent from numeric lexical validity and from later required/optional or
repeatable-role rules.

The fixed role arrays live beside the M14.2a role registry so evidence and
cardinality cannot drift into separate field inventories.

## Test evidence

`text_symbol_card_tests.rs` covers ASCII/Binary parity across all nine
AC1009--AC1032 dialects; exact per-family card counts; absent, unique, and
multiple states; repeated TEXT content, MTEXT chunks, and ambiguous group 50;
signed-32-bit members; lexical invalidity independent of cardinality;
cross-family role absence; source identity; lookup bounds; cancellation; and
public `Copy`/`Send`/`Sync` bounds.

## Non-claims

M14.2b does not decide whether a role is required, optional, or repeatable. It
does not select scalar values, order/merge MTEXT chunks, disambiguate group 50,
assign common-property ownership, apply defaults, decode text, validate
layouts, resolve styles, derive geometry, edit, or write.

## Required checkpoint gates

All required gates passed on 2026-07-31: dependency policy, generated-schema
drift, release-evidence drift, formatting, workspace Clippy with warnings
denied, 624 workspace tests with zero failures/ignored tests, and
`git diff --check`. No dependency manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 78 | `11e5816691e5d4ae10b5ea166797aa0a2c11e919b5ddd620de0feba62ccb4ef9` |
| `crates/seacad-dxf-core/src/lib.rs` | 660 | `cd50b634f1d75cee3e3ed66cd0adfd6bfcac71122fa44fd69b72d2f690474ea4` |
| `crates/seacad-dxf-core/src/text_symbol_card.rs` | 305 | `c252a7a28a4de45662e2b80f3fbd88ec483db39d925f01bbf94876aada77e137` |
| `crates/seacad-dxf-core/src/text_symbol_role.rs` | 342 | `b41054dde5609f9da5d0e640bc07c028a9a6e47111edf34cc74f2222a2e5762c` |
| `crates/seacad-dxf-core/tests/text_symbol_card_tests.rs` | 345 | `c61fd35f6d7589bffc9b74e69b6dc2ccb40c75d6e06c9dd2cd340c742f94ad7b` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 110 | `ffd0426c401a729a09d43045acd31902579d24ca69cbf1f6b98b4d940609e41f` |
| `docs/IMPLEMENTATION_PLAN.md` | 1405 | `7e767bed1b613e6d48ad2c5e4653b1dbd02f1f419f5e7c3a4589161951dde1e9` |
| `docs/SUPPORT_MATRIX.md` | 1126 | `5d1fcb0a728fb4d72a690da0d1a45fa0e44bd330b820e5ab5f4581ea85239087` |
