# M4.3c2a evidence-backed MIF mapping receipt

Status: completed 2026-07-27

This receipt records selector-registry, strict decode, malformed-input,
AutoCAD-oracle, boundary, and resource evidence. All test vectors are
synthetic and contain no private CAD data.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `crates/seacad-dxf-core/src/lib.rs` | `3ee092145789e0148f692b03ae467b7723a4f14fb65db2d861659254492f6989` |
| `crates/seacad-dxf-core/src/text_escape.rs` | `5794685f7dc6a89abf5a191f04774114528d2b044969905cbcd8655453692856` |
| `docs/IMPLEMENTATION_PLAN.md` | `6d498bf1768b8448203439c2fcc65b9e1bc660148e30dbdbc4b21c635f3adb18` |
| `docs/SUPPORT_MATRIX.md` | `2f706e6742e138672da784ba8bb7e174b0a5ae9bd4e2288f3186130dcee39701` |
| `docs/M4_3C2A_MIF_MAPPING_CONTRACT.md` | `517cb52aed362bba52105cae8ad4277db1ae248dd9298e07d38ca3bef7693113` |
| `docs/audits/M4_3C2A_MIF_SELECTOR_ORACLE.md` | `6f83b6883db5fe0ceac08e56d1e7675a395a096d40fedffbf7b77704fcd362ac` |

M4.3c2a changes no Cargo manifest, lockfile, feature, dependency, generated
table, FFI, or platform-specific production path. It reuses the reviewed
replacement-free `encoding_rs` pages already pinned by M4.3b1.

## Synthetic vector receipts

Hashes cover exact UTF-8 input bytes passed to the escape layer.

| Case | Bytes | SHA-256 | Required result |
| --- | ---: | --- | --- |
| selectors 1/2/3/5 plus `00xx` | 42 | `117f9aeb22f874f62f1b839ab6b0e575bd18b1c53a4e0dba31581be17bd14d5a` | `Aあ一가你AZ`, complete |
| lowercase `m` | 10 | `16755745d6fb667a0722d91e6c26e25256e2592f612b263419b90bb18f3b1eda` | `AあB`, complete |
| selector 4, `88 61` | 8 | `5491149409c5982bbacbe31029606360c9c6e5ef2f6b890a70931f4a989ade49` | typed CP1361/Johab unsupported; no output |
| selectors 0, 6, and non-decimal | 26 | `b1d8cc8a01c6e0b69de812a81611d3892c50411001d0a887aaeb26ec071a8c15` | unchanged literal UTF-8 |
| truncated valid selector | 8 | `149ce69f41dd1cbdefff1ca8a44ab80e4334c7a3dec335b407a7fd42970b1d17` | typed `MifTruncated` at offset 1 |
| invalid hexadecimal | 9 | `1924e5051477e0fb0b3864b6f128783e670f5b5b764926e33ab8d1fc6ab5ac33` | typed `MifInvalidHex` at offset 1 |
| invalid CP932 byte pair | 9 | `b2b64a42e477d8a0f80a0b7923464b51e2d49d01bceb281a356e9eb23fb65580` | typed `MifInvalidCode` at offset 1 |
| two independent single bytes | 9 | `190cf5900f06c7a79690045e2eb663418c1e37272f9c973ab52c6bb41f8f29af` | typed `MifInvalidCode`; never emits two scalars |
| bounded MIF output | 9 | `16d5377e8eed0e0217bc0673485f045a2ad52c9c4158af553eb73a20705c5c8e` | three-byte destination stops before `あ`; four completes |

Every successful MIF decode consumes one complete token and emits exactly one
Unicode scalar. Every error retains the already written prefix and inserts no
replacement scalar.

## Oracle and provenance evidence

`M4_3C2A_MIF_SELECTOR_ORACLE.md` records the exact AutoCAD Core Console binary
hash, nine selector/case fixtures, one two-single-byte negative fixture,
semantic output hashes, isolated invocation, and CP1361 boundary. The selector
table is corroborated by an Autodesk-authored historical header at an exact
commit/blob and Microsoft numeric codepage references. No external source or
mapping table was copied into SeaCad.

## Review size and gates

`text_escape.rs` grows from 254 to 354 physical production lines and from 260
to 316 test/helper lines. The 100-line production increase is deliberately
narrower than the nominal checkpoint size because the full CP1361 table is
split into M4.3c2b rather than being guessed or padded into this review.

The checkpoint passes:

- `cargo fmt --all -- --check`;
- `cargo clippy --workspace --all-targets -- -D warnings`;
- `cargo test --workspace`: 128 passed, 0 failed;
- forbidden production pattern scan: only crate-level
  `#![forbid(unsafe_code)]`, no forbidden call sites;
- `git diff --check`;
- artifact SHA-256 verification.

The repository CI matrix must independently pass Windows x64, macOS ARM64,
and Linux x64 before the checkpoint is reported complete.
