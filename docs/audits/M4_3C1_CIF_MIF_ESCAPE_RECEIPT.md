# M4.3c1 CIF decode and MIF recognition receipt

Status: completed 2026-07-27

This receipt records deterministic syntax, Unicode, integration, negative, and
resource-bound evidence. All vectors are synthetic and contain no private CAD
data.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `crates/seacad-dxf-core/src/lib.rs` | `4bfd605a4fecb3bc02f543d9331375104bf7cb59a777e475a6e1f999e826ad02` |
| `crates/seacad-dxf-core/src/text_escape.rs` | `f524801527651e7be1d54f67b4aa226790d8cfef0cb5d1cd7b6054b296ff1430` |
| `docs/IMPLEMENTATION_PLAN.md` | `a9d9825b8a8c17cedb8588b1304ad6edb41bca785aa8f5814dc31c8463b6aa24` |
| `docs/M4_3C1_CIF_MIF_ESCAPE_CONTRACT.md` | `3741d725492b3226a0999d30acccd4032ab5258b44677f91dfb5976a6201a964` |
| `docs/SUPPORT_MATRIX.md` | `a1ace80ab6ad10f4ee6616b5da3aaf18a1283bc3eaee6cd753551f53aa3c9547` |
| `docs/audits/M4_3C1_AUTODESK_ESCAPE_REFERENCE.md` | `b1343b91ba378fb01f38515576a6ff32ef8fe010f7f9dcc26444430e20aeff9b` |

M4.3c1 changes no Cargo manifest, lockfile, feature, dependency, or external
source. It composes with the reviewed M4.3b storage decoder and adds no runtime
lookup table.

## Synthetic vector receipts

Hashes cover the exact UTF-8 bytes passed to the escape layer, except the final
row, which hashes the complete LF-framed ASCII DXF integration fixture.

| Case | Bytes | SHA-256 | Required result |
| --- | ---: | --- | --- |
| `A\U+00E9\U+4f60Z` | 16 | `c59c2915c846ae48ebe7244e5644c3ee8718ea3675dddb97723eac1781d21734` | `Aé你Z`; exact upper/lower hex decode |
| `\U+D83D\U+DE00` | 14 | `a3927155f8ea3fc38f29b8139d95203c74666b940e36513153dd710f78718ea7` | `U+1F600`; exact four-byte UTF-8 output |
| `x\U+12G4` | 8 | `eadf10d0f68d0f856ac5913ec84efe199f40e055bf3979b9d0dcf16c4f8d4cdc` | typed `CifInvalidHex`; prefix `x` only |
| `x\U+D83D` | 8 | `16ecd06fd0c1fa5fd8daf38ce36d51af56a0a15a5dec20b22946a7b2904dcd74` | typed unpaired high surrogate; no replacement |
| `\M+182A0` | 8 | `5afd30f7aa7de3d4ea7084dc9695b8725ed36e049997cccb533c069e5aa885c3` | `UnsupportedMif { selector: 1, code: 0x82A0 }` |
| `x\M+A82A0` | 9 | `646c6693fec76e5168eba755b30ff66842f5e75083ea39ca0cf70ee09e72759a` | typed invalid MIF selector at offset 1 |
| `x\U+4F60` | 8 | `8e3b606a2b6acc48926ef0da9bd271c9e6ebcaf89934bd3d230538ad6503b2ee` | exact four-byte destination completes; three bytes stop before scalar |
| AC1021 group value `Bien\U+0111` | 105 | `76fdd49ccf1433ebdf724fc75b99bc2b1ff3423ce7936201e24885fc5c6cecb1` | source receipt retained; composed output `Bienđ` |

## Negative and bounded evidence

Tests distinguish truncated CIF, invalid CIF hex, isolated high/low
surrogates, a high surrogate followed by a non-low unit, invalid second-token
hex, truncated MIF, invalid MIF selector, and invalid MIF hex. Every defined
output prefix is replacement-free.

Lowercase `\u+` and unrelated backslash sequences remain literal because MTEXT
quoting is outside this layer. Plain UTF-8 and CIF output stop only at complete
scalar boundaries. `OutputFull` is retried from the complete source. Valid MIF
examples with selectors 0 through 4 return typed metadata and leave the token
unconverted.

The integration test opens a strict AC1021 source, decodes one group value by
document occurrence, validates the source-anchored receipt, then passes only
the defined UTF-8 prefix into the escape layer. The implementation allocates
nothing and uses fixed four-byte scalar storage.

AutoCAD 2027 produced no usable oracle evidence for this checkpoint because
the isolated process entered its error reporter before opening the first
fixture. No failed-oracle artifact or result is used for a support claim.

## Review size and gates

The new module contains 254 physical production lines and 260 test/helper
lines. Five module/export lines make 259 Rust production lines for the
checkpoint, within the 200-500-line target.

The checkpoint requires Rustfmt, workspace Clippy with warnings denied, all
123 workspace tests, forbidden-production-pattern scan, `git diff --check`,
artifact-hash verification, and the Windows x64/macOS ARM64/Linux x64 CI
matrix.
