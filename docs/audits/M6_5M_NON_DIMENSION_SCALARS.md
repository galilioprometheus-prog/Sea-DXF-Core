# M6.5m Remaining Non-DIMENSION HEADER Scalars Audit

## Scope

M6.5m appends the remaining sixteen reviewed numeric or Boolean HEADER fields
outside the `$DIM*` family:

- group 370/380 signed 16-bit values: `$CELWEIGHT` and `$CEPSNTYPE`;
- group 280 signed 16-bit values: `$CSHADOW` and `$SORTENTS`;
- group 62/70 signed 16-bit values: `$DISPSILH`, `$INSUNITS`,
  `$INTERFERECOLOR`, `$INTERSECTIONCOLOR`, `$OBSCOLOR`, `$UCSORTHOVIEW`,
  `$UNITMODE`, `$USRTIMER`, `$VISRETAIN`, and `$WORLDVIEW`;
- group 290 strict Boolean values: `$XCLIPFRAME` and `$XEDIT`.

The checkpoint reviews the DXF field identity and wire storage only. It does
not yet assign enum, color, lineweight, plot-style, shadow, display, UCS,
units, xref, timer, clipping, default, range, or version-applicability
semantics.

## Normative evidence

Autodesk's HEADER table gives the selected field names and group codes:

<https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-A85E8E67-27CD-4C59-BE61-4DC9FADBE74A.htm>

Autodesk's DXF file value-type table assigns group codes 60-79, 280-289, and
370-389 to signed 16-bit values, and 290-299 to Boolean values:

<https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-2553CF98-44F6-4828-82DD-FE3BC7448113.htm>

The group-280/290 wire-width decision remains backed by the isolated AutoCAD
2027 Binary DXF receipt in
`docs/audits/M5_1A_AUTODESK_BINARY_REFERENCE.md`. No source from an external
CAD parser was copied, translated, or ported.

## Generic decode and exact evidence

All sixteen fields enter the existing schema-driven directory. No
field-specific parser or lookup state machine was added. ASCII values use the
bounded locale-free integer grammar; Binary signed values preserve their exact
two-byte little-endian payload. Boolean accepts only `0` and `1`; other
source values remain typed invalid with exact raw provenance.

The code generator now validates group 370-389 as the same DXF-file `Int16`
wire family as group 60-79 and 270-289. It rejects a schema row whose declared
storage and group-code family disagree.

## AC1009 physical boundary

Pre-R13 Binary DXF has a one-byte group-code header, so it cannot physically
encode group codes 280, 290, 370, or 380. The AC1009 parity fixture therefore
marks `$CELWEIGHT`, `$CEPSNTYPE`, `$CSHADOW`, `$SORTENTS`, `$XCLIPFRAME`, and
`$XEDIT` absent in both standard physical fixtures. Its ten group-62/70 fields
remain explicit. AC1012 through AC1032 exercise all sixteen fields in ASCII and
Binary.

This is a standard-fixture policy, not a claim that arbitrary ASCII input is
version-valid or invalid. Version applicability remains separate schema work.

## Append-only ordinal contract

The prior 101 field ids remain frozen at ordinals 0-100:

| Ordinal | Field id | DXF name | Storage |
| ---: | --- | --- | --- |
| 101 | `celweight` | `$CELWEIGHT` | `Int16` |
| 102 | `cepsntype` | `$CEPSNTYPE` | `Int16` |
| 103 | `cshadow` | `$CSHADOW` | `Int16` |
| 104 | `dispsilh` | `$DISPSILH` | `Int16` |
| 105 | `insunits` | `$INSUNITS` | `Int16` |
| 106 | `interferecolor` | `$INTERFERECOLOR` | `Int16` |
| 107 | `intersectioncolor` | `$INTERSECTIONCOLOR` | `Int16` |
| 108 | `obscolor` | `$OBSCOLOR` | `Int16` |
| 109 | `sortents` | `$SORTENTS` | `Int16` |
| 110 | `ucsorthoview` | `$UCSORTHOVIEW` | `Int16` |
| 111 | `unitmode` | `$UNITMODE` | `Int16` |
| 112 | `usrtimer` | `$USRTIMER` | `Int16` |
| 113 | `visretain` | `$VISRETAIN` | `Int16` |
| 114 | `worldview` | `$WORLDVIEW` | `Int16` |
| 115 | `xclipframe` | `$XCLIPFRAME` | `Boolean` |
| 116 | `xedit` | `$XEDIT` | `Boolean` |

Stable field ids remain the canonical persisted identity.

## Verification coverage

Tests cover the complete AC1009-AC1032 ASCII/Binary matrix, all 117 generated
field rows, append-only ordinals, the AC1009 physical boundary, exact
group-62/70/280/290/370/380 provenance, `i16::MIN`, malformed ASCII range
failure, strict Boolean rejection, cancellation, and deterministic
wire-family validation.

## Verification result

Rust 1.97.1 passed:

- `cargo fmt --all -- --check`;
- deterministic schema generation in `--check` mode;
- Clippy for the workspace and all targets with warnings denied;
- 229 workspace tests: 200 core unit tests, 7 numeric integration tests,
  12 CLI tests, and 10 schema-generator tests;
- `git diff --check`.

The code/schema/generated production diff adds 400 lines and removes 8,
within the approximate 200-500-line production target. Test-only assertions
and fixtures add 150 lines and remove 2. No dependency, unsafe block,
field-specific parser/lookup state machine, `panic!`, `unwrap`, `expect`,
`todo!`, or `unimplemented!` was added.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `schema/dxf/v1/header.bootstrap.json` | `f10ffeca40f57230235feae391f9ddd3b7962830612b61b6e7ed4d40b5a2728f` |
| `crates/seacad-schema-gen/src/main.rs` | `b8fba2633c5c24078307b9770aff5abfbc471c879f41a5b1b11c38ccd21e3f2a` |
| `crates/seacad-dxf-core/src/generated/header_schema.rs` | `fd596db4efc01d63a75ed87e65c2a19a2f06626aea1d63fb4c65780597e6e804` |
| `crates/seacad-dxf-core/src/header_numeric.rs` | `bc683cddda1ce9d833c2dcea9be60c8062df7af1b19ffa6425b6e13d535ee047` |
| `crates/seacad-dxf-core/src/header_schema_directory.rs` | `7ee041a5e84cbb58d005720ee1f7d213148991c60e8ee7440e695eadaaa3b7c9` |
| `crates/seacad-dxf-core/tests/header_numeric_tests.rs` | `8d02a4ddf0440d827b4659b36d853e647624a71c70857a6034ca6fd8c663c424` |
| `docs/IMPLEMENTATION_PLAN.md` | `7f9f2450790d015fc4939564aa876269a4e9400c507269f1fab94df9bdd8753e` |
