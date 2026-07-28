# M6.6b remaining source-anchored HEADER text fields

M6.6b appends the remaining twelve reviewed HEADER rows whose wire storage is
exact text. It reuses the M6.6a schema-ordered, format-neutral directory and
adds no field-specific parser branch. Filesystem paths, symbol names, and GUID
spellings remain source text; this checkpoint makes no claim about resolution,
validation, normalization, defaults, version applicability, or higher-level
meaning.

## Normative basis

The variable names and group-code shapes come from Autodesk's published HEADER
table:

<https://help.autodesk.com/cloudhelp/2026/ENU/AutoCAD-DXF/files/GUID-A85E8E67-27CD-4C59-BE61-4DC9FADBE74A.htm>

The reviewed group-code value-type reference classifies codes `1`, `2`, and `7`
as string/name storage:

<https://help.autodesk.com/cloudhelp/2026/ENU/AutoCAD-DXF/files/GUID-2553CF98-44F6-4828-82DD-FE3BC7448113.htm>

Binary strings retain their documented NUL-terminated wire framing while the
raw document exposes only the exact payload span:

<https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-FC1C3C69-DBC2-49E4-893A-000D6538C0FE.htm>

The normalized Autodesk inventory remains 206 rows with source-facts SHA-256
`d1034c4246758f368ac79982fa1c21d59328851d9fdca8ccebe944f20f71cfaf`.

## Append-only schema slice

All 198 M6.6a fields at ordinals `0..197` remain unchanged. M6.6b appends:

| Ordinal | Field ID | DXF name | Group code |
| ---: | --- | --- | ---: |
| 198 | `fingerprintguid` | `$FINGERPRINTGUID` | 2 |
| 199 | `hyperlinkbase` | `$HYPERLINKBASE` | 1 |
| 200 | `menu` | `$MENU` | 1 |
| 201 | `projectname` | `$PROJECTNAME` | 1 |
| 202 | `pucsbase` | `$PUCSBASE` | 2 |
| 203 | `pucsname` | `$PUCSNAME` | 2 |
| 204 | `pucsorthoref` | `$PUCSORTHOREF` | 2 |
| 205 | `textstyle` | `$TEXTSTYLE` | 7 |
| 206 | `ucsbase` | `$UCSBASE` | 2 |
| 207 | `ucsname` | `$UCSNAME` | 2 |
| 208 | `ucsorthoref` | `$UCSORTHOREF` | 2 |
| 209 | `versionguid` | `$VERSIONGUID` | 2 |

All twelve rows are optional and `shape_only`; applicability and defaults stay
`not_yet_reviewed`. They use the exact-text wire-code set already bounded by
the generator to `1`, `2`, `3`, `6`, `7`, and `8`.

## Public contract

No public API is added or changed. `DxfRawDocumentView`,
`DxfAsciiRawDocument`, and `DxfBinaryRawDocument` continue to expose
`header_text_directory(cancellation)`. The directory now contains all 25
generated `ExactText` fields: `$ACADVER`, `$DWGCODEPAGE`, the eleven M6.6a
rows, and the twelve M6.6b rows. Every entry retains its complete-schema
ordinal.

The existing `DxfHeaderTextValue::decode_to_utf8_without_replacement` contract
continues to read the original ASCII or Binary payload by source identity,
group occurrence, value span, and document encoding. It performs no trimming,
case folding, path cleanup, filesystem access, symbol lookup, GUID parsing, or
replacement-character recovery.

## Verification scope

Synthetic ASCII and Binary fixtures cover all 25 text-directory entries in
schema order. Exact decoding checks preserve whitespace, mixed case, path
separators and `..` segments, project-name spelling, and GUID-like spelling.
Existing coverage keeps absence, wrong group code, marker-without-value,
multiple value groups, duplicate variables, cancellation, source-identity
mismatch, replacement-free decoding, public trait bounds, and redacted debug
output typed and bounded.

The complete schema directory retains ASCII/Binary parity across every
supported AC1009-through-AC1032 dialect. M6.6c's four remaining HEADER handle
rows remain out of scope.

No Cargo manifest, dependency, lockfile, support matrix, source-facts receipt,
filesystem behavior, writer, edit API, or support claim changes in this
checkpoint.

## Reviewed implementation artifacts

There is no new handwritten runtime parsing path. The schema adds 144 JSON
lines and the generated registry changes by 134 additions and two removals.
Handwritten Rust changes are test-only checkpoint invariants: three count
replacements in the schema directory, 18 additions and one removal in the
generator tests, and 42 additions and one removal in the text integration
tests. The implementation plan changes by two additions and one removal.

| Artifact | SHA-256 |
| --- | --- |
| `schema/dxf/v1/header.bootstrap.json` | `10f988343249671eab0e01cea9e7d2e48fb2cf7809ac9235f48db4dd8969ffdf` |
| `crates/seacad-dxf-core/src/generated/header_schema.rs` | `e9af52512da6dcf5a2ea7d3e3841870ba75fe83005752fc40aa300645084f66f` |
| `crates/seacad-dxf-core/src/header_schema_directory.rs` | `aebc741a0da84087d7235e2cc5c5d9b8ca4fd206d40496eed37806eb9187f03e` |
| `crates/seacad-dxf-core/src/header_text.rs` | `21e2c02bc683084a8d14fd318fc47680542bfb315ae31cbbc7df938e5bf45ef3` |
| `crates/seacad-schema-gen/src/main.rs` | `e235e858cec546d5a9e966ce31c73df8e90304e01f1b9f8f3845b1f71c4cb8a3` |
| `crates/seacad-dxf-core/tests/header_text_tests.rs` | `0bd195321d22006a6c77acc73ebfa0da8092c6244dbda981e00134daca387720` |
| `docs/IMPLEMENTATION_PLAN.md` | `a80f1f91f042674b0aa202297593b6d3846101ad1b07049576c4c412fde40f4c` |

The generated normalized-input SHA-256 is
`45c891b9f3999eb317fee41950a8c82216575082fc4d57d09da01eb27f7cb51a`.

## Required gates

Rust 1.97.1 passed:

- `cargo deny --locked check`;
- `cargo +1.97.1 fmt --all -- --check`;
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`;
- `cargo +1.97.1 clippy --workspace --all-targets -- -D warnings`;
- `cargo +1.97.1 test --workspace`: 252 passed;
- `git diff --check`.

Targeted evidence also passed 202 core unit tests, 12 schema-generator tests,
and four text-directory integration tests. The reviewed production files
contain no `unsafe`, `panic!`, `unwrap`, `expect`, `todo!`, or
`unimplemented!`. `Cargo.lock` and every Cargo manifest remain unchanged.
