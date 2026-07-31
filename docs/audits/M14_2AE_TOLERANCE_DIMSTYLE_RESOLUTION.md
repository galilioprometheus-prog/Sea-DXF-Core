# M14.2ae TOLERANCE DIMSTYLE Resolution

## Scope

M14.2ae introduces exact DIMSTYLE table-name evidence and resolves each usable
TOLERANCE group-3 dimension-style name to zero, one, or multiple table records.
It preserves raw record, field, and source-span evidence and does not interpret
DIMSTYLE properties.

## Normative basis

Autodesk defines symbol tables as exact `TABLE` records followed by group 2
identifying the table and terminated by `ENDTAB`; DIMSTYLE is one of the
enumerated uppercase table names:
<https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-5AB9300F-F0AC-4ADE-89EA-A9D1D152D8B8.htm>.

Autodesk defines DIMSTYLE entry group 2 as the dimension-style name:
<https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-F2FAD36F-0CE3-4943-9DAD-A9BCD2AE81DA.htm>.
The TOLERANCE entity's group 3 is its dimension-style name:
<https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-ADFCED35-B312-4996-B4C1-61C53757B3FD.htm>.

## Contract

- Only exact `DIMSTYLE` records inside a complete exact
  `TABLE` + group-2 `DIMSTYLE` + `ENDTAB` envelope participate.
- Each target must contain exactly one group-2 name outside application groups.
- Interrupted/unclosed tables, wrong-case markers/names, duplicate/missing
  target names, and application-group decoys are excluded without guessing.
- Resolution compares authoritative source spans byte-for-byte and remains
  case-sensitive; it does not decode, normalize, allocate a name-sized buffer,
  or use the advisory table-count field.
- Outcomes are typed as unusable name, missing, unique, or ambiguous. Every
  duplicate target is retained in source order.
- TOLERANCE field semantics, exact DIMSTYLE entries, raw records, application
  groups, source identity, and target lookup remain reachable.

## Duplication reduction and size

A new bounded source-span helper centralizes SHA-256 hashing, span comparison,
and span-to-byte comparison previously duplicated by BLOCK name consistency,
BLOCK name indexing, and block-local ATTDEF tag indexing. Their 14 focused
regression tests remain unchanged and passing.

The new helper is 121 lines, DIMSTYLE table module 236 lines, resolution module
258 lines, and focused test module 287 lines. Refactored production modules are
385, 184, and 477 lines; every changed module remains below 500 lines. No
dependency or localized UI string is added.

## Coverage

All nine supported dialects have ASCII/Binary parity for unique, missing,
ambiguous, and unusable outcomes. Focused tests cover duplicate preservation,
case sensitivity, complete/interrupted/unclosed envelopes, wrong-case markers,
duplicate names, application-group decoys, cancellation, source identity,
lookup, and public traits.

## Nonclaims

This checkpoint does not project DIMSTYLE fields, interpret tolerance strings,
construct glyph geometry, edit, or write.

## Verification

All required gates passed on 2026-07-31: dependency policy, generated-schema
drift, release-evidence drift, formatting, workspace Clippy with warnings
denied, 711 workspace tests with zero failures or ignored tests, production
forbidden-macro scanning, and `git diff --check`. The focused resolution suite
passed 4/4 tests and the three refactored comparator consumers passed 14/14
regression tests. No dependency manifest or lockfile changed.

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 79 | `702c86c9877c04780afc29b1eea3fd69fc37b8c98fe2b984834173b19624fc9c` |
| `crates/seacad-dxf-core/src/lib.rs` | 789 | `e5f2454f44c3601a24e210343257560d4659fc8ef20a9fc3b2a9c23c4b798a7a` |
| `crates/seacad-dxf-core/src/source_span.rs` | 121 | `6e806892e85e095cfbeed99a7e846d0b9c677c301834515a2a95adae818d50b2` |
| `crates/seacad-dxf-core/src/dimstyle_table.rs` | 236 | `f6fde4e574f7c4c6b54499a33323cf90f7967bfc4fae27fec5c6b501a1df6ddf` |
| `crates/seacad-dxf-core/src/tolerance_dimstyle_resolution.rs` | 258 | `7efab66dfef683ef5d7a3c79e06c81c6bb74bca77b11ac3e61a1dbe8597bcf0a` |
| `crates/seacad-dxf-core/src/block_attribute_definition_tag_index.rs` | 477 | `e6d334b5e72cf5d278f61e42e4caf55d3ac1147b75967e0995ba843be470b9cd` |
| `crates/seacad-dxf-core/src/block_name_consistency.rs` | 184 | `5d63a6ce5ad0e49a3420b2f7d4ece780697047ab3aca591373cc2a49bbb187b3` |
| `crates/seacad-dxf-core/src/block_name_index.rs` | 385 | `950f91096e6080483a82ba31dc53c142ab059bb5d5cd8c5b406fb5f27986741a` |
| `crates/seacad-dxf-core/tests/tolerance_dimstyle_resolution_tests.rs` | 287 | `691cf474099b9ba430db872dfb1db323da45d6cf29ae630596b311fc8568ab32` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 280 | `6f39303eb5ab12c5bda9758dab99853b7be4b25ec9b747546ee68bcb48231fbd` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,638 | `3519214cc220bae7020b62e8fe4e830bd616c6d9a3df5c70d324d1fc0c8be4dc` |
| `docs/SUPPORT_MATRIX.md` | 1,332 | `50215444ffeb6312c25ce2c16182aef76013ba5beacd8da514984a2026078493` |
