# M6.5l Extended Integer and Boolean HEADER Audit

## Scope

M6.5l appends ten shape-reviewed HEADER fields:

- group 280, decoded as exact signed 16-bit DXF file values: `$ENDCAPS`,
  `$HALOGAP`, `$INDEXCTL`, `$JOINSTYLE`, and `$OBSLTYPE`;
- group 290, decoded as strict Boolean flags: `$EXTNAMES`, `$HIDETEXT`,
  `$INTERSECTIONDISPLAY`, `$LWDISPLAY`, and `$PSTYLEMODE`.

The checkpoint preserves the generated field identity, four-state semantic
value, group occurrence, byte span, and exact raw bytes. It does not interpret
the enum, bit-field, display, xref, indexing, or default meanings described by
the individual variables.

## Normative and oracle evidence

Autodesk's HEADER table gives the selected field names and group codes:

<https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-A85E8E67-27CD-4C59-BE61-4DC9FADBE74A.htm>

Autodesk's DXF file value-type table assigns 280-289 to 16-bit integer values
and 290-299 to Boolean flags:

<https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-2553CF98-44F6-4828-82DD-FE3BC7448113.htm>

Some ObjectARX API tables describe 280-289 as 8-bit integers. SeaCad does not
silently merge those contracts. The M5.1a AutoCAD 2027 Binary DXF oracle found
two payload bytes after group 280 and one payload byte after group 290 in
AutoCAD-generated files. That receipt is frozen in:

`docs/audits/M5_1A_AUTODESK_BINARY_REFERENCE.md`.

SeaCad follows the DXF file wire evidence: group 280 uses the existing
little-endian `i16` decoder; group 290 uses one Boolean byte. No external
parser source was copied, translated, or ported.

## Boolean contract

ASCII and Binary Boolean values accept only the exact numeric domain `0` and
`1`. SeaCad does not treat an arbitrary nonzero value as true. Other values
produce `BooleanOutOfDomain` while retaining source provenance and raw bytes.
ASCII lexical and `i16` range errors remain `InvalidAsciiNumber`.

Binary Boolean framing already bounds the payload to one byte. The semantic
decoder performs no allocation and keeps cancellation checks before and after
source reads.

## Dialect boundary

Pre-R13 Binary DXF uses a one-byte group-code header and therefore cannot
represent group codes 280 or 290. The AC1009 standard parity fixture records
the new fields as absent in both physical formats. AC1012 through AC1032
fixtures exercise all ten fields in ASCII and Binary. This fixture policy does
not invent version applicability for arbitrary noncanonical ASCII input;
applicability remains separately reviewable schema metadata.

## Append-only ordinal contract

The prior 91 field ids remain frozen at ordinals 0-90:

| Ordinal | Field id | DXF name | Storage |
| ---: | --- | --- | --- |
| 91 | `endcaps` | `$ENDCAPS` | `Int16` |
| 92 | `extnames` | `$EXTNAMES` | `Boolean` |
| 93 | `halogap` | `$HALOGAP` | `Int16` |
| 94 | `hidetext` | `$HIDETEXT` | `Boolean` |
| 95 | `indexctl` | `$INDEXCTL` | `Int16` |
| 96 | `intersectiondisplay` | `$INTERSECTIONDISPLAY` | `Boolean` |
| 97 | `joinstyle` | `$JOINSTYLE` | `Int16` |
| 98 | `lwdisplay` | `$LWDISPLAY` | `Boolean` |
| 99 | `obsltype` | `$OBSLTYPE` | `Int16` |
| 100 | `pstylemode` | `$PSTYLEMODE` | `Boolean` |

Stable field ids remain the canonical persisted identity.

## Verification coverage

Tests cover the full AC1009-AC1032 ASCII/Binary dialect matrix, schema counts,
append-only ordinals, exact 2-byte group-280 and 1-byte group-290 provenance,
true and false values, malformed ASCII, Boolean domain rejection, invalid
Binary byte 255, cancellation, and generated wire-family boundaries.

Final gate results and artifact SHA-256 receipts are appended after the
checkpoint verification run.

## Verification result

Rust 1.97.1 passed:

- `cargo fmt --all -- --check`;
- deterministic schema generation in `--check` mode;
- Clippy for the workspace and all targets with warnings denied;
- 229 workspace tests: 200 core unit tests, 7 numeric integration tests,
  12 CLI tests, and 10 schema-generator tests;
- `git diff --check`.

The code/schema/generated production diff adds 368 lines and removes 16,
within the approximate 200-500-line production target. Test-only assertions
and fixtures add 162 lines and remove 9. No dependency, unsafe block,
field-specific lookup state machine, `panic!`, `unwrap`, `expect`, `todo!`, or
`unimplemented!` was added.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `schema/dxf/v1/header.bootstrap.json` | `353399ff19d6b31e8d7c3c4de0b40d3202dfdced1ded8f8dac3ef8ef23dae4cb` |
| `crates/seacad-schema-gen/src/main.rs` | `046c2e5fdfe392d6e61d44598fe96050df38d427d5d5b752235d69ac96341dc6` |
| `crates/seacad-dxf-core/src/generated/header_schema.rs` | `a3561d3e29210058d0f7c306b0a6ade1c4b73709f6d61bc9a0516346d39b0909` |
| `crates/seacad-dxf-core/src/header_numeric.rs` | `bc683cddda1ce9d833c2dcea9be60c8062df7af1b19ffa6425b6e13d535ee047` |
| `crates/seacad-dxf-core/src/header_schema_directory.rs` | `1704b057d0592daf1ccd885205164c8dd6de14dc58e4b60b333575ffd9ca5669` |
| `crates/seacad-dxf-core/tests/header_numeric_tests.rs` | `b32dde606dcb9afacf9f0b3ddbcdfe5903194fcc6e35a6de4e53730d9e06cf91` |
| `docs/IMPLEMENTATION_PLAN.md` | `26aaf8d8bb93de1aa890f282c9ae59e0f48facb44be0f89bae4b0a879ed7f694` |
