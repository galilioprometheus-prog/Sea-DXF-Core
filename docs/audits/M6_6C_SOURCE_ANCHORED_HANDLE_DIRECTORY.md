# M6.6c source-anchored HEADER handle directory

M6.6c adds one schema-ordered, format-neutral directory for reviewed HEADER
fields whose wire storage is a hexadecimal handle. It appends the final four
handle rows after the M6.6b prefix and reuses SeaCad's bounded exact-handle
parser. The resulting values are identifiers only: this checkpoint does not
resolve object references, infer ownership, validate target existence, or add
pointer semantics.

## Normative basis

The variable names and group-code shapes come from Autodesk's published HEADER
table:

<https://help.autodesk.com/cloudhelp/2026/ENU/AutoCAD-DXF/files/GUID-A85E8E67-27CD-4C59-BE61-4DC9FADBE74A.htm>

The group-code value-type reference classifies `340..349` as hexadecimal object
IDs and `390..399` as hexadecimal handle values:

<https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-2553CF98-44F6-4828-82DD-FE3BC7448113.htm>

Autodesk's numerical group-code reference describes `340..349` as hard-pointer
handles and `390..399` as plot-style handle values:

<https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3F0380A5-1C15-464D-BC66-2C5F094BCFB9.htm>

The normalized Autodesk HEADER inventory remains 206 rows with source-facts
SHA-256
`d1034c4246758f368ac79982fa1c21d59328851d9fdca8ccebe944f20f71cfaf`.

## Append-only schema slice

All 210 M6.6b fields at ordinals `0..209` remain unchanged. M6.6c appends:

| Ordinal | Field ID | DXF name | Group code |
| ---: | --- | --- | ---: |
| 210 | `cepsnid` | `$CEPSNID` | 390 |
| 211 | `dragvs` | `$DRAGVS` | 349 |
| 212 | `interfereobjvs` | `$INTERFEREOBJVS` | 345 |
| 213 | `interferevpvs` | `$INTERFEREVPVS` | 346 |

All four rows are optional and `shape_only`; applicability and defaults remain
`not_yet_reviewed`. The generator accepts only the reviewed HEADER handle codes
`5`, `345`, `346`, `349`, and `390`; adjacent and unrelated codes fail closed.

## Public contract

`DxfRawDocumentView`, `DxfAsciiRawDocument`, and `DxfBinaryRawDocument` expose
`header_handle_directory(cancellation)`. The directory includes all five
generated `Handle` fields: the existing `$HANDSEED` row at ordinal 10 and the
four M6.6c rows at ordinals `210..213`.

Each explicit `DxfHeaderHandleValue` contains the parsed `DxfHandle`, document
`SourceId`, exact group occurrence, and payload span. `read_raw_spelling` reads
the original hexadecimal bytes only from the matching raw document, preserving
leading zeros and letter case. Parsing uses a fixed 16-byte stack buffer and
the existing exact 1-to-16-digit hexadecimal grammar.

Absence, lexical invalidity, wrong group codes, marker-without-value, multiple
value groups, and duplicate variables remain distinct typed states with bounded
raw and schema provenance. The existing specialized `$HANDSEED` report and
`DxfHeaderView::handseed` contract remain unchanged.

No API follows a handle into an entity or object table, classifies a referenced
object, or assigns ownership. Those topology semantics remain scheduled for M7.

## Verification scope

Synthetic ASCII and Binary fixtures cover all five handle-directory entries in
complete-schema order. They preserve null handles, leading zeros, lowercase and
mixed-case hexadecimal spelling while producing the exact parsed 64-bit value.
Negative vectors cover wrong group codes, missing values, multiple value groups,
duplicate variables, invalid hexadecimal digits, values longer than 16 digits,
cancellation, source-identity mismatch, public trait bounds, and redacted
directory debug output.

The complete 214-slot schema directory retains ASCII/Binary parity across every
supported AC1009-through-AC1032 dialect. Version applicability for the four new
rows remains explicitly unreviewed. M6.7 evidence closure remains out of scope.

No Cargo manifest, dependency, lockfile, support matrix, source-facts receipt,
writer, edit API, topology lookup, or support claim changes in this checkpoint.

## Reviewed implementation artifacts

The handwritten production diff adds 429 physical Rust lines: 425 lines in the
new handle-directory module and four module/export lines. The schema adds 48
JSON lines and the generated registry changes by 46 additions and two removals.
Test/checkpoint changes replace three schema-count literals, add the generator's
bounded handle-code coverage, add 334 integration-test lines, and update the
implementation plan by three additions and one removal.

| Artifact | SHA-256 |
| --- | --- |
| `schema/dxf/v1/header.bootstrap.json` | `8a871216ac42ccbccf86d6c13bcb469ba9b7bc55b2a749c6bbe82a5f0de1c5d2` |
| `crates/seacad-dxf-core/src/generated/header_schema.rs` | `1f93ce236436de7bbbebf0a17377b5a8f22f71e99b25c0f2c0960a26ef4915cb` |
| `crates/seacad-dxf-core/src/handle.rs` | `a0cc7438a1cecb6d3e3627345ef92885e357e431c6ff3b017ae40bec88335b1d` |
| `crates/seacad-dxf-core/src/header_handle.rs` | `87397d8efdb4a6b8310b7690caeddf6f7cea7e091aa7f89fc037445ddbb150aa` |
| `crates/seacad-dxf-core/src/header_schema_directory.rs` | `7bca3f49ca20cd18bf3e76fb57e2d81ce6e474a653c948b709f2f5b7d1ebb84c` |
| `crates/seacad-dxf-core/src/lib.rs` | `826717f4f715546b6c993509d465494347ceb6d2dc34d6d11c704bc59bff7848` |
| `crates/seacad-schema-gen/src/main.rs` | `18a30e6b71e3e8a2cfd1a3c9793fd969697ca8700b8d4b690c06ee8f777acb4b` |
| `crates/seacad-dxf-core/tests/header_handle_tests.rs` | `3eefbc1a4c5e8ae98cea784fffd7d149c49bcaa50c5669823ef192b1dfdd0eff` |
| `docs/IMPLEMENTATION_PLAN.md` | `2d09a258c4f9f3db425969b3245605b7ca6cf37ffcc08af5f6347cd39d357a42` |

The generated normalized-input SHA-256 is
`ff28e978910e9b9995afa6c10e6399e8a1e5b532fbc85cf6c25f8ce17da4ea4f`.

## Required gates

Rust 1.97.1 passed:

- `cargo deny --locked check`;
- `cargo +1.97.1 fmt --all -- --check`;
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`;
- `cargo +1.97.1 clippy --workspace --all-targets -- -D warnings`;
- `cargo +1.97.1 test --workspace`: 257 passed;
- `git diff --check`.

Targeted evidence also passed 202 core unit tests, 13 schema-generator tests,
12 numeric-directory tests, four text-directory tests, and four new
handle-directory integration tests. The reviewed production files contain no
`unsafe`, `panic!`, `unwrap`, `expect`, `todo!`, or `unimplemented!`.
`Cargo.lock` and every Cargo manifest remain unchanged.
