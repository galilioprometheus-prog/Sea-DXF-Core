# M4.3b1 codepage registry and decoder receipt

Status: completed 2026-07-27

This receipt records deterministic implementation and vector evidence for the
exact registry and bounded replacement-free decoder. No private CAD data or
external source code is included.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `crates/seacad-dxf-core/src/text_decoder.rs` | `b6632568cec08cc615e2385655fbeba0a52cfe163eeacd09c29c986c1c71057c` |
| `crates/seacad-dxf-core/src/lib.rs` | `39def32bbe632549bc40656422990c5a3b3fd9177ea77a2261f2583f48d88e8f` |
| `crates/seacad-dxf-core/Cargo.toml` | `17412ed5c169b5a7c3dff0f36ba5c339bcf54816add8e86cf6995bc225ac5703` |
| `Cargo.lock` | `cf943f92617468bdf222309c5a5105673bc5e7a10501a8a7802d1b21d9871424` |
| `THIRD_PARTY_NOTICES.md` | `187cd8cac66cc926445e4ab519e1be92ca77308cd5047cae3df7638af4769925` |

Cargo.lock adds only `encoding_rs 0.8.35`; its `cfg-if 1.0.4` dependency was
already locked. The package checksum is
`75030f3c4f45dafd7586dd6780965a8c7e8e285a5ecb86713e63a79c5b2766f3`.

## Registry decode vectors

Each source byte sequence is passed as one complete value. Tests require exact
UTF-8 output and `Complete`, with no replacement character.

| Token | Source hex | Expected scalar | Source SHA-256 |
| --- | --- | --- | --- |
| `ANSI_874` | `A1` | `U+0E01` | `8a8950f7623663222542c9469c73be3c4c81bbdf019e2c577590a61f2ce9a157` |
| `ANSI_932` | `82 A0` | `U+3042` | `73170355cd1bc70b8cacb33a972fbdbafc98b9db122d8c9176dba98bfd6c8d09` |
| `ANSI_936` | `C4 E3` | `U+4F60` | `0335c0c49b3f53c973169ab9ec6433ee3e5c477ee7f6075ae7e5b7be19a5a706` |
| `ANSI_949` | `B0 A1` | `U+AC00` | `78fbe360deed3fd09565464bcfd57912e597015cee3fcf5ee0e256265a81b876` |
| `ANSI_950` | `A4 40` | `U+4E00` | `d4df43b260cdc9847460c5026c92da9b30aced30ec1c01b305a2c97d676bb2f8` |
| `ANSI_1250` | `8C` | `U+015A` | `9defb0a9e163278be0e05aa01b312ec78cfa3726869503385e76e3a4b7950648` |
| `ANSI_1251` | `C0` | `U+0410` | `e4ff5e7d7a7f08e9800a3e25cb774533cb20040df30b6ba10f956f9acd0eb3f7` |
| `ANSI_1252` | `E9` | `U+00E9` | `de2e331d891ae267a7009cb45b4e8830f170e0c937288ea2731a1941c7a53b0d` |
| `ANSI_1253` | `C1` | `U+0391` | `d1bbd73bb09190bfb883056771e22e997541ed20079793bf33975fe1654581c3` |
| `ANSI_1254` | `D0` | `U+011E` | `d4b0c0a4a8cc6c257aed34d16d39dd3c2d3539ed67fd4badd40aef16c1591715` |
| `ANSI_1255` | `E0` | `U+05D0` | `7d8c5da7fd418379048e430b33dc8ffcda739e44326b8a5d647dc0ad81ed2157` |
| `ANSI_1256` | `C7` | `U+0627` | `3340883aad3038dd993b3c94d2d32c3b20e07859969aca411f7f93ab8847c746` |
| `ANSI_1257` | `C0` | `U+0104` | `e4ff5e7d7a7f08e9800a3e25cb774533cb20040df30b6ba10f956f9acd0eb3f7` |
| `ANSI_1258` | `D0` | `U+0110` | `d4b0c0a4a8cc6c257aed34d16d39dd3c2d3539ed67fd4badd40aef16c1591715` |

Identical one-byte hashes in different rows are expected: the selected
codepage, not the byte alone, determines the Unicode scalar.

## Negative and resource-bound evidence

Tests reject case variants, padded tokens, web aliases, `DOS437`,
`ANSI_1361`, UTF-8 labels, empty values, and unknown tokens. UTF-8 malformed
input and an incomplete Shift_JIS lead byte return `Malformed` without U+FFFD.
A destination smaller than four bytes returns zero-progress `OutputFull`;
larger bounded output stops only between complete Unicode scalars. A test
retries the full source with a larger buffer and completes exactly; `read` is
not treated as a fresh-decoder resume cursor. Empty input completes with an
empty destination.

## Review size and gates

The new module contains 211 production lines and 188 test/helper lines.
Together with four module/export lines, M4.3b1 adds 215 Rust production lines,
within the 200-500-line micro-milestone target.

The checkpoint requires Rustfmt, workspace Clippy with warnings denied, all 105
workspace tests, forbidden-production-pattern scan, `git diff --check`,
artifact-hash verification, and the three-platform GitHub Actions matrix.
