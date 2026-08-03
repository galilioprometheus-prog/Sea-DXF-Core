# M14.3cr Entity XDATA Capacity

Retrieved: 2026-08-04

## Normative and behavioral basis

Autodesk documents that XDATA is limited to 16 KB per entity and that all
registered applications on that entity share the limit:

<https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-AutoLISP/files/GUID-A94BC605-5517-437F-A6FE-D3EB8116A01A.htm>

Autodesk defines the generic group-1000 through group-1071 value families:

<https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-A2A628B0-3699-4740-A215-C560E7242F63.htm>

Autodesk documents `xdsize` as the exact memory-size query and `xdroom` as the
remaining entity capacity. AutoCAD 2027 Core Console X.60.0.0 was used only as
a behavioral oracle for the byte formula:

<https://help.autodesk.com/view/ACDLT/2026/ENU/?caas=caas%2Fdocumentation%2FACD%2F2014%2FENU%2Ffiles%2FGUID-F183738E-43BC-4820-AC51-5D0C1A3DA38C-htm.html>

The oracle returned 16,383 for an empty entity's `xdroom`, 16,383 for the exact
boundary list, and 16,384 for the one-byte-over list. It also established the
logical costs encoded below. A BMP scalar and a supplementary scalar each
increased a string by two bytes; this distinguishes Unicode-scalar count from
UTF-8 bytes and UTF-16 code units.

## Contract

`DxfEntityXDataCapacityDirectory` emits one source-bound entry for every
indexed entity, including exact zero-XDATA entries. Each nonempty registered,
structurally valid application contributes three bytes. Logical value bodies
contribute: group 1000 `3 + 2 * decoded Unicode scalar count`; group 1002 `2`;
resolved group 1003 `3`; group 1004 `2 + decoded length`; group 1005 `9`;
each complete 101x/102x/103x tuple `25`; groups 1040--1042 `9`; group 1070
`3`; and group 1071 `5`. An empty application contributes zero.

An exact total at or below 16,383 publishes `WithinLimit`; a larger exact total
publishes `Exceeded`. Orphans, missing/ambiguous APPIDs or layers, invalid
application structure, invalid typed values, partial tuples, unavailable or
malformed storage decoding, and malformed CIF/MIF escapes publish
`Indeterminate` with an accounted lower bound and compact typed issues. No
invalid source is normalized into a plausible exact size.

## Verification boundary

The focused suite pairs ASCII and Binary fixtures for AC1009, AC1012, AC1014,
AC1015, AC1018, AC1021, AC1024, AC1027, and AC1032. It covers empty and multiple
applications, every logical value family, raw UTF-8, BMP and supplementary CIF
escapes, exact 16,383 and exceeded 16,384 totals, missing/ambiguous APPIDs and
layers, orphans, invalid structure/value/text, partial tuples, source identity,
cancellation, lookup bounds, metadata bounds, and non-disclosing debug output.

This checkpoint does not apply point/vector coordinate transforms, assign
application-specific payload semantics, resolve or remap group-1005 targets,
clone/write XDATA, or advance any entity to `Complete`.

## Gate receipts

The focused capacity suite passed 3/3 tests and the full workspace passed all
989 listed tests across 186 targets. Generated schema and release-evidence
checks, `cargo deny --locked check`, formatting, workspace Clippy with warnings
denied, the production forbidden-construct scan, and `git diff --check` all
passed. No manifest, lockfile, dependency, generated schema, locale source,
committed fixture, or external corpus changed.

Production adds 761 lines split between a 374-line public contract/directory
and a 380-line internal logical-value measurer, plus seven root exports. This
is above the preferred 200--500 aggregate micro-milestone size; the deviation
keeps the externally reviewable API and the composed APPID/structure/value/
tuple/layer/text algorithm in separate files, each below 400 lines, without
shipping an unusable partially exact capacity claim. The focused target adds
442 lines. This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 448 | `58b2ba3846a3c659e22900dabf5ce521d14ff44b836239911f838bd88dfb2841` |
| `crates/seacad-dxf-core/src/entity_xdata_capacity.rs` | 374 | `d4248b20a231e9116dc52f41337b94b497068b098f4f6fb88cd037218d741569` |
| `crates/seacad-dxf-core/src/entity_xdata_capacity_measure.rs` | 380 | `47994bdd2c79c2810aeac2241c9d28b967196f6d91869cc6009e016a30671a14` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,107 | `fb1307b678d42a7a51fd727b72c63d0aedf692aca56e4977bfe3da1448638558` |
| `crates/seacad-dxf-core/tests/entity_xdata_capacity_tests.rs` | 442 | `7dd9adcfe57885d2d8e33d6cd96926140ee389c66218da308c16c64c821ad197` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,418 | `6e072b6adbaaedaad54ca4373e58ec45aa9814305da49f0ac5f20456b499c680` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,834 | `41d047361768b07303194c9f492edea18f22ee1d63dca7871a4f2aced31a1e88` |
| `docs/SUPPORT_MATRIX.md` | 2,478 | `d5ec2d22b12168f2a6253c33deafdc99c79fe55f953bd1e2bb366a87b9c895cb` |
