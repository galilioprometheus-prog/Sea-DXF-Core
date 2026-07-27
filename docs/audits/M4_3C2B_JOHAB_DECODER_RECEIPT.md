# M4.3c2b exact CP1361/Johab decoder receipt

Status: local gates passed 2026-07-27; checkpoint CI receipt is the annotated
tag and GitHub Actions run.

Base checkpoint:

- commit: `81a385bd07a86291186cdabfeb1f8638a0b18a54`;
- tag: `m4.3c2a-mif-selector-map`.

## Accepted contract

M4.3c2b adds deterministic, replacement-free CP1361/Johab decoding to the
exact `ANSI_1361` storage registry and to MIF selector 4. The runtime performs
constant-time lookup in a fixed 65,536-slot table, allocates nothing, calls no
platform API, and retains at most one lead byte between source chunks.

Malformed and undefined input remains typed and fail-closed. The implementation
does not add an encoder, best-fit conversion, semantic string cache, MTEXT
formatting, binary framing, edit path, writer path, CLI command, or JSON field.

## Primary data and mapping evidence

| Evidence | Receipt |
| --- | --- |
| Unicode-hosted Microsoft `bestfit1361.txt` | 1,223,827 bytes; SHA-256 `7dcda2d5d2cfc5ddf43757d589a0106e020ef0d9b84de47f08e18297ae0fe1ec` |
| source decode map | 17,395 records; canonical SHA-256 `74e0dd256041e42198bcc5a17d509e9bd4ccedbed82f1d4f8e2a276eecdac61e` |
| excluded `Undefined -> EUDC` singles | 11 records: D4-D7, DF, FA-FF |
| strict source map | 132 single + 17,252 double = 17,384 records |
| strict canonical map | SHA-256 `5f038ab2832fc3b597115b3138480142d5dccc97a39edcf61567b0dff8385a84` |
| exhaustive current Windows NLS | 17,384 records; 0 extra; 0 value mismatch; same canonical SHA-256 |
| frozen direct table | 131,072 bytes; SHA-256 `d04a1a13d5f4706df6fa46394cda98a817570e774d601acd042e0fa57249f7ea` |
| committed audit-tool regeneration | byte-identical direct table; all frozen counts and hashes passed |

The committed tool checks every `u16` payload against
`MultiByteToWideChar(1361, MB_ERR_INVALID_CHARS, ...)` on Windows. Platform
FFI belongs only to that audit tool; production Rust has no FFI. The source
data is not vendored. Unicode License v3 review and the required notice are
committed with this checkpoint.

## AutoCAD 2027 evidence

Oracle executable:

- version: `26.0.60.0.0`;
- SHA-256:
  `fc6b59c29fea759d556083cf43d9de3657974ef0bf57962d49a19892f729fa77`.

Isolated `/readonly` and `/safemode` runs established:

| Fixture | Input SHA-256 | Exact UTF-8 output SHA-256 |
| --- | --- | --- |
| MIF selector 4 families | `a12d7912f72b820d949e7cc1623b02139553bc1a13d8eaa5546e6d122d4d2172` | `ba8027f27edb11c7d24211a9600daac5338682ea93e0df09643c530ddfcb5cbb` |
| raw `ANSI_1361` storage | `2f33c8769cf1143a258764f41690a082a2e2d9c699b4f4932d5a3b4a084210ad` | `b909b0fe7efa01a5cfe6a640390d289f84edcaecacaee9f38b0c4375d2ed0912` |

AutoCAD supplies semantic evidence only. It is not used to claim verbatim
preservation, and no Autodesk artifact is stored or distributed.

## Verification

- Exact registry, representative family, malformed, `OutputFull`, streaming
  boundary, and AutoCAD storage-vector tests pass.
- An exhaustive in-repository test checks all 65,536 MIF selector 4 payloads
  against the frozen table or the typed `MifInvalidCode` result.
- A source-backed test splits a valid Johab pair across the core's 4 KiB input
  boundary and retains exact encoding/source provenance.
- `cargo test -p seacad-dxf-core`: 126 passed.
- `cargo test --workspace`: 137 passed.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- Production forbidden-construct scan: zero unsafe blocks/functions or
  `panic!`, `unwrap()`, `expect()`, `todo!`, and `unimplemented!` hits;
  crate-level `#![forbid(unsafe_code)]` remains active.
- `git diff --check`: passed.
- No `Cargo.toml` or `Cargo.lock` change.

The GitHub Actions matrix must pass on Windows x64, macOS ARM64, and Linux x64
before the checkpoint is reported complete.

## Artifact hashes

| Artifact | SHA-256 |
| --- | --- |
| `.gitattributes` | `38329331656195ad733b1574617fe7ac5406155346b9bae4cbb6f62cac7c96d1` |
| `crates/seacad-dxf-core/src/encoding.rs` | `5948083503dae78586cc03e6bd4ac193fe31b2e5e5b2903ceba89219c6f36a8d` |
| `crates/seacad-dxf-core/src/johab.rs` | `a4063be98e7ab692afc8a092d1c462eba93b847eff9a207fd0a08196c4c43d1a` |
| `crates/seacad-dxf-core/src/johab_decode_le.bin` | `d04a1a13d5f4706df6fa46394cda98a817570e774d601acd042e0fa57249f7ea` |
| `crates/seacad-dxf-core/src/lib.rs` | `38b8d277bd718f94e07954d1fd168a335ad176644a68ed8b8426b27395a7010d` |
| `crates/seacad-dxf-core/src/text_decoder.rs` | `d695771208213fee123f0d45d03bfe4021b91f19f8383b0da017cca9f7f7c9be` |
| `crates/seacad-dxf-core/src/text_escape.rs` | `5e1b5b84d8c05e653e6605a5ad7e861ab4073963dfa1d08fbf9f9cd043f2cdbf` |
| `crates/seacad-dxf-core/src/text_view.rs` | `7518db0d45615a533c09048c1c54994ab67fa7dbaa586ab48d458864dd62df41` |
| `docs/DEPENDENCY_POLICY.md` | `8cde6099e5cb680da85db2fa50291b9b29b2317c9045d25b223eca6b0da39aa2` |
| `docs/IMPLEMENTATION_PLAN.md` | `f303d65e677fa532a455fde0de4c1e148b33431861694d484ca3d2deb73d5806` |
| `docs/SUPPORT_MATRIX.md` | `3a6a67da174322e9c900d97ca29eae9e454713e5946e2d86d6aeb5ba8867d48c` |
| `docs/M4_3C2B_JOHAB_DECODER_CONTRACT.md` | `7f29deaab4e498ea721add52c59b454365f791803183c09460c8e7810b899345` |
| `docs/audits/M4_3C2B_JOHAB_MAPPING_ORACLE.md` | `5b3bbfd3300bb69f77be8af3a2f60c0749deb54f4b8b0ba516b982cc66245097` |
| `docs/audits/M4_3C2B_UNICODE_DATA_LICENSE_REVIEW.md` | `4942837afb0f972913d9c3d169d11951a4471e8df23f7ea3a5dc9bf5c97f2ea8` |
| `THIRD_PARTY_NOTICES.md` | `0398284555eaebaa08c076c1739c6a832669f5d97c8955d52878d46cbb521e16` |
| `tools/audit-johab-table.ps1` | `d161663c37d55d47976b4fdc405b8f6a2277ae9e1459a66b0dd72f0768cc0450` |

The receipt does not hash itself. The annotated checkpoint tag binds this
receipt and every listed artifact to the final commit.
