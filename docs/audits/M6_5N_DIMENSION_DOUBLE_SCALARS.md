# M6.5n DIMENSION Double-Scalar HEADER Audit

## Scope

M6.5n appends the first nineteen reviewed `$DIM*` HEADER fields. Every field
is a single group-code 40 double:

- `$DIMALTF`, `$DIMALTRND`, `$DIMASZ`, `$DIMCEN`, `$DIMDLE`, `$DIMDLI`;
- `$DIMEXE`, `$DIMEXO`, `$DIMFAC`, `$DIMGAP`, `$DIMLFAC`, `$DIMRND`;
- `$DIMSCALE`, `$DIMTFAC`, `$DIMTM`, `$DIMTP`, `$DIMTSZ`, `$DIMTVP`, and
  `$DIMTXT`.

The checkpoint preserves the exact source scalar and provenance only. It does
not interpret alternate-unit conversion, rounding, arrow or center-mark size,
line spacing, extension distances, fraction or tolerance scaling, text
placement, text height, drawing units, defaults, ranges, signs, zero values,
version applicability, or relationships between fields.

## Normative evidence

Autodesk's published HEADER table identifies every selected variable and group
code 40:

<https://help.autodesk.com/cloudhelp/2026/ENU/AutoCAD-DXF/files/GUID-A85E8E67-27CD-4C59-BE61-4DC9FADBE74A.htm>

The independently normalized 206-row Autodesk 2015, 2018, 2021, 2024, 2025,
and 2026 inventories remain identical, with SHA-256
`d1034c4246758f368ac79982fa1c21d59328851d9fdca8ccebe944f20f71cfaf`.
Sixty-eight rows begin with `$DIM`; nineteen of those rows use group code 40.

Autodesk's DXF file value-type table assigns group code 40 to a
double-precision floating-point value:

<https://help.autodesk.com/cloudhelp/2026/ENU/AutoCAD-DXF/files/GUID-2553CF98-44F6-4828-82DD-FE3BC7448113.htm>

The ASCII grammar, Binary little-endian IEEE-754 representation, non-finite
payload preservation, and bounded semantic reads remain those audited in
M6.5a and M6.5b. No external parser implementation or fixture was copied,
translated, or ported.

## Generic decode and physical coverage

All nineteen fields enter the existing schema-driven numeric directory. No
field-specific parser, lookup state machine, public value type, dependency,
source limit, writer behavior, or file-sized allocation is added.

Group code 40 is representable by the one-byte pre-R13 Binary group-code
header. The standard fixture therefore exercises every new field in both ASCII
and Binary for all supported AC1009-AC1032 versions.

Each explicit value retains its exact `DxfDouble` bits, field provenance, group
occurrence, value byte span, and original raw bytes. ASCII lexical or range
failure and structural conflicts remain typed invalid states; Binary NaN
payload bits remain explicit raw values.

## Append-only ordinal contract

The prior 117 field ids remain frozen at ordinals 0-116:

| Ordinal | Field id | DXF name |
| ---: | --- | --- |
| 117 | `dimaltf` | `$DIMALTF` |
| 118 | `dimaltrnd` | `$DIMALTRND` |
| 119 | `dimasz` | `$DIMASZ` |
| 120 | `dimcen` | `$DIMCEN` |
| 121 | `dimdle` | `$DIMDLE` |
| 122 | `dimdli` | `$DIMDLI` |
| 123 | `dimexe` | `$DIMEXE` |
| 124 | `dimexo` | `$DIMEXO` |
| 125 | `dimfac` | `$DIMFAC` |
| 126 | `dimgap` | `$DIMGAP` |
| 127 | `dimlfac` | `$DIMLFAC` |
| 128 | `dimrnd` | `$DIMRND` |
| 129 | `dimscale` | `$DIMSCALE` |
| 130 | `dimtfac` | `$DIMTFAC` |
| 131 | `dimtm` | `$DIMTM` |
| 132 | `dimtp` | `$DIMTP` |
| 133 | `dimtsz` | `$DIMTSZ` |
| 134 | `dimtvp` | `$DIMTVP` |
| 135 | `dimtxt` | `$DIMTXT` |

Stable field ids remain the canonical persisted identity. Schema ordinals are
append-only positions and must not replace field ids in durable external data.

## Verification coverage

Tests cover:

- all 136 generated schema rows and 133 numeric directory entries;
- exact values and raw provenance for every new field across the complete
  AC1009-AC1032 ASCII/Binary matrix;
- append-only ordinals and deterministic generated output;
- a wrong group code, malformed ASCII double, and duplicate variable with
  exact conflict evidence;
- exact Binary NaN payload bits and provenance for a new dimension field;
- existing cancellation and public `Copy`/`Send`/`Sync` bounds.

## Verification result

Rust 1.97.1 passed:

- `cargo fmt --all -- --check`;
- deterministic schema generation in `--check` mode;
- Clippy for the workspace and all targets with warnings denied;
- 230 workspace tests: 200 core unit tests, 8 numeric integration tests,
  12 CLI tests, and 10 schema-generator tests;
- `git diff --check`.

The authoritative schema/generated production registry diff adds 439 lines and
removes 2, within the approximate 200-500-line production target. Test-only
updates add 178 lines and remove 5. The implementation plan adds 7 lines and
removes 2.

No dependency, public API, production decoder, unsafe block, field-specific
parser/lookup branch, `panic!`, `unwrap`, `expect`, `todo!`, or
`unimplemented!` was added.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `schema/dxf/v1/header.bootstrap.json` | `5a372c5de3065e1a656045e4dc1468ccdf963d2e09cd52ad6e23f0d0d1902cf0` |
| `crates/seacad-schema-gen/src/main.rs` | `695bd31e40da2dac9cf151746642eea981f741f108c7912e70b6f1358ef8626a` |
| `crates/seacad-dxf-core/src/generated/header_schema.rs` | `f6a8ae6f2acd0355d5d8a002615a9ad1c7427e9cbe52924d5188c55ca8965a6c` |
| `crates/seacad-dxf-core/src/header_schema_directory.rs` | `17eccbefb46f277f59aa0be7212efc449e48a6f5edc0ed4311c8f1861f8c65c2` |
| `crates/seacad-dxf-core/tests/header_numeric_tests.rs` | `ff6af527d2a49d917e445acd2d9ea3ce4c73f4269844cde4f41e179e17e73ae8` |
| `docs/IMPLEMENTATION_PLAN.md` | `49cf1bc1c7c055d3881f3b69f090f424e822e1d49821db37bed195e54848c913` |
