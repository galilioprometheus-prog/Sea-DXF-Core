# M6.6a source-anchored HEADER text directory

M6.6a adds one schema-ordered, format-neutral directory for reviewed HEADER
fields whose wire storage is exact text. It appends eleven source rows after
the M6.5q prefix and makes no claim about filesystem paths, symbol resolution,
defaults, version applicability, allowed values, or higher-level meaning.

## Normative basis

The variable names and group-code shapes come from Autodesk's published HEADER
table:

<https://help.autodesk.com/cloudhelp/2026/ENU/AutoCAD-DXF/files/GUID-A85E8E67-27CD-4C59-BE61-4DC9FADBE74A.htm>

The reviewed group-code value-type reference classifies the selected codes as
string/name storage:

<https://help.autodesk.com/cloudhelp/2026/ENU/AutoCAD-DXF/files/GUID-2553CF98-44F6-4828-82DD-FE3BC7448113.htm>

Binary strings retain their documented NUL-terminated wire framing while the
raw document exposes only the exact payload span:

<https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-FC1C3C69-DBC2-49E4-893A-000D6538C0FE.htm>

The normalized Autodesk inventory remains 206 rows with source-facts SHA-256
`d1034c4246758f368ac79982fa1c21d59328851d9fdca8ccebe944f20f71cfaf`.

## Append-only schema slice

Ordinals `0..186` remain unchanged. M6.6a appends:

| Ordinal | Field ID | DXF name | Group code |
| ---: | --- | --- | ---: |
| 187 | `celtype` | `$CELTYPE` | 6 |
| 188 | `clayer` | `$CLAYER` | 8 |
| 189 | `cmlstyle` | `$CMLSTYLE` | 2 |
| 190 | `dimapost` | `$DIMAPOST` | 1 |
| 191 | `dimblk` | `$DIMBLK` | 1 |
| 192 | `dimblk1` | `$DIMBLK1` | 1 |
| 193 | `dimblk2` | `$DIMBLK2` | 1 |
| 194 | `dimldrblk` | `$DIMLDRBLK` | 1 |
| 195 | `dimpost` | `$DIMPOST` | 1 |
| 196 | `dimstyle` | `$DIMSTYLE` | 2 |
| 197 | `dimtxsty` | `$DIMTXSTY` | 7 |

All eleven rows are optional and `shape_only`; applicability and defaults stay
`not_yet_reviewed`. The generator accepts only reviewed exact-text codes
`1`, `2`, `3`, `6`, `7`, and `8`, with explicit negative coverage for
`0`, `4`, `5`, and `9`.

## Public contract

`DxfRawDocumentView`, `DxfAsciiRawDocument`, and `DxfBinaryRawDocument` expose
`header_text_directory(cancellation)`. The directory includes every generated
`ExactText` row, including the earlier `$ACADVER` and `$DWGCODEPAGE`, so M6.6a
contains thirteen entries while retaining each complete-schema ordinal.

Each explicit entry retains document `SourceId`, exact group occurrence and
payload span, document encoding resolution, schema field provenance, and raw
value provenance. `Absent` and structural invalidity are distinct. Wrong group
codes, marker-without-value, multiple value groups, and duplicate exact
variables remain typed failures with bounded evidence.

`DxfHeaderTextValue::decode_to_utf8_without_replacement` reads the original
ASCII or Binary payload through the shared raw-document adapter. It rejects a
different document identity, performs no trimming, case folding, path cleanup,
symbol lookup, or replacement-character recovery, and preserves the existing
bounded output/retry contract.

## Verification scope

Synthetic ASCII and Binary vectors cover all thirteen directory entries,
complete-schema ordering, codes `1/2/3/6/7/8`, exact whitespace/case/path-like
bytes, typed absence and structural failures, cancellation, source-identity
mismatch, replacement-free decoding, public trait bounds, and redacted debug
output. M6.6b's remaining twelve text rows and M6.6c's four handle rows remain
out of scope.

No Cargo manifest, dependency, lockfile, support matrix, filesystem behavior,
writer, or edit API changes in this checkpoint.

## Reviewed implementation artifacts

The production diff adds 437 physical Rust lines and removes 10: 403 lines in
the new text-directory module, 30 additions and 10 removals in the shared text
view, and four export/module lines. Generated code and tests are counted
separately.

| Artifact | SHA-256 |
| --- | --- |
| `schema/dxf/v1/header.bootstrap.json` | `a015a255f83a231451bea54babc263ea7bdf246fc55dc479b1bdcce4ec0ed865` |
| `crates/seacad-dxf-core/src/generated/header_schema.rs` | `60484608561c58e74cd3136c0d878d748a832ae104e7e43e25fa1d139edb21a0` |
| `crates/seacad-dxf-core/src/header_schema_directory.rs` | `4364eeba520ffd43aa1fc6d5554dddb464db99a4327da9b68c0c9acd402a4399` |
| `crates/seacad-dxf-core/src/header_text.rs` | `21e2c02bc683084a8d14fd318fc47680542bfb315ae31cbbc7df938e5bf45ef3` |
| `crates/seacad-dxf-core/src/text_view.rs` | `946f72c54947fb851e33a9465d0979d6f0c5687dd433c8c33122d3dd21938932` |
| `crates/seacad-dxf-core/src/lib.rs` | `668b36bb98352975e4c1f30f15fbd59fd257a886a3da11451bc43e4103b0f233` |
| `crates/seacad-schema-gen/src/main.rs` | `2cd5dc45053fe2b2e3982fd65d110740f4276456474575e707976027ec5bd8eb` |
| `crates/seacad-dxf-core/tests/header_text_tests.rs` | `9920139550dd5837ac042a1ff507302cac4cf3b7fe0d5040e6492fe8460cb49d` |
| `docs/IMPLEMENTATION_PLAN.md` | `3e394fb3d891f1cc3222794387912c47935377448faad380afd18d7931f58604` |

The generated normalized-input SHA-256 is
`d296ca18c3d1c88011e6b017fe4cd3308c916cabde643630355e1b0f594d072f`.

## Required gates

Rust 1.97.1 passed:

- `cargo deny --locked check`;
- `cargo +1.97.1 fmt --all -- --check`;
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`;
- `cargo +1.97.1 clippy --workspace --all-targets -- -D warnings`;
- `cargo +1.97.1 test --workspace`: 252 passed;
- `git diff --check`.

Targeted evidence also passed 202 core unit tests, 12 schema-generator tests,
and four new text-directory integration tests. The reviewed production files
contain no `unsafe`, `panic!`, `unwrap`, `expect`, `todo!`, or
`unimplemented!`. `Cargo.lock` and every Cargo manifest remain unchanged.
