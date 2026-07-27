# M6.5o DIMENSION Formatting and Precision Integer HEADER Audit

## Scope

M6.5o appends fourteen reviewed `$DIM*` HEADER fields. Every field is one
group-code 70 signed 16-bit integer:

- `$DIMADEC`, `$DIMALTD`, `$DIMALTTD`, `$DIMALTTZ`, `$DIMALTU`;
- `$DIMALTZ`, `$DIMAUNIT`, `$DIMAZIN`, `$DIMDEC`, `$DIMDSEP`;
- `$DIMLUNIT`, `$DIMTDEC`, `$DIMTZIN`, and `$DIMZIN`.

The checkpoint preserves the exact source integer and provenance only. It does
not interpret precision, unit formats, zero-suppression masks, decimal
separator characters, defaults, ranges, version applicability, or
relationships between fields. In particular, `$DIMDSEP` remains an `i16`; it
is not converted to a Unicode scalar or encoded character.

## Normative evidence

Autodesk's published HEADER table identifies every selected variable and group
code 70:

<https://help.autodesk.com/cloudhelp/2026/ENU/AutoCAD-DXF/files/GUID-A85E8E67-27CD-4C59-BE61-4DC9FADBE74A.htm>

The independently normalized 206-row Autodesk 2015, 2018, 2021, 2024, 2025,
and 2026 inventories remain identical, with SHA-256
`d1034c4246758f368ac79982fa1c21d59328851d9fdca8ccebe944f20f71cfaf`.
Sixty-eight rows begin with `$DIM`; fourteen formatting and precision rows in
this batch use group code 70.

Autodesk's DXF file value-type table assigns group codes 60-79 to signed
16-bit integer values:

<https://help.autodesk.com/cloudhelp/2026/ENU/AutoCAD-DXF/files/GUID-2553CF98-44F6-4828-82DD-FE3BC7448113.htm>

The ASCII numeric grammar, Binary little-endian representation, and bounded
semantic reads remain those audited in M6.5a and M6.5b. No external parser
implementation or fixture was copied, translated, or ported.

## Generic decode and physical coverage

All fourteen fields enter the existing schema-driven numeric directory. No
field-specific parser, lookup state machine, public value type, dependency,
source limit, writer behavior, or file-sized allocation is added.

Group code 70 is representable by the one-byte pre-R13 Binary group-code
header. The standard fixture therefore exercises every new field in both ASCII
and Binary for all supported AC1009-AC1032 versions.

Each explicit value retains its exact `i16`, field provenance, group
occurrence, value byte span, and original raw bytes. ASCII lexical or range
failure and structural conflicts remain typed invalid states. Binary signed
boundary values remain exact raw integers.

## Append-only ordinal contract

The prior 136 field ids remain frozen at ordinals 0-135:

| Ordinal | Field id | DXF name |
| ---: | --- | --- |
| 136 | `dimadec` | `$DIMADEC` |
| 137 | `dimaltd` | `$DIMALTD` |
| 138 | `dimalttd` | `$DIMALTTD` |
| 139 | `dimalttz` | `$DIMALTTZ` |
| 140 | `dimaltu` | `$DIMALTU` |
| 141 | `dimaltz` | `$DIMALTZ` |
| 142 | `dimaunit` | `$DIMAUNIT` |
| 143 | `dimazin` | `$DIMAZIN` |
| 144 | `dimdec` | `$DIMDEC` |
| 145 | `dimdsep` | `$DIMDSEP` |
| 146 | `dimlunit` | `$DIMLUNIT` |
| 147 | `dimtdec` | `$DIMTDEC` |
| 148 | `dimtzin` | `$DIMTZIN` |
| 149 | `dimzin` | `$DIMZIN` |

Stable field ids remain the canonical persisted identity. Schema ordinals are
append-only positions and must not replace field ids in durable external data.

## Verification coverage

Tests cover:

- all 150 generated schema rows and 147 numeric directory entries;
- exact values and raw provenance for every new field across the complete
  AC1009-AC1032 ASCII/Binary matrix;
- append-only ordinals and deterministic generated output;
- a wrong group code, out-of-range ASCII integer, and duplicate variable with
  exact conflict evidence;
- exact Binary `i16::MIN` value and provenance for `$DIMDSEP`, demonstrating
  that no character conversion occurs;
- existing cancellation and public `Copy`/`Send`/`Sync` bounds.

## Verification result

Rust 1.97.1 passed:

- `cargo fmt --all -- --check`;
- deterministic schema generation in `--check` mode;
- Clippy for the workspace and all targets with warnings denied;
- 231 workspace tests: 200 core unit tests, 9 numeric integration tests,
  12 CLI tests, and 10 schema-generator tests;
- `git diff --check`.

The authoritative schema/generated production registry diff adds 324 lines and
removes 2, within the approximate 200-500-line production target.

No dependency, public API, production decoder, unsafe block, field-specific
parser/lookup branch, `panic!`, `unwrap`, `expect`, `todo!`, or
`unimplemented!` was added.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `schema/dxf/v1/header.bootstrap.json` | `70d60f2ed25e862c06c58adf48ebc55ec6abdbfc2214317661b154a88bb105b3` |
| `crates/seacad-schema-gen/src/main.rs` | `b2ca7d7b2ee63d3651718da347521dd9fbd0155d5f0e21204fc0f0a2813145fc` |
| `crates/seacad-dxf-core/src/generated/header_schema.rs` | `1dedd30742e041195cfd026062b5fd0f361ce8bfd6bf2177611bb569d7580bec` |
| `crates/seacad-dxf-core/src/header_schema_directory.rs` | `e321fe649d22260765f6567b79b609f19cb4f55715df32de98b101014ac93ae7` |
| `crates/seacad-dxf-core/tests/header_numeric_tests.rs` | `5eed18878f8dc50086c1b2756e8c684b15474b746c535806239bcdefc27d88b8` |
| `docs/IMPLEMENTATION_PLAN.md` | `1ea803f87c50e05325bc9ad2ff2d4783a86326d6f3ff4603fd17c99201eea3cf` |
