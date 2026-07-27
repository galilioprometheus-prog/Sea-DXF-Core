# M5.2a Binary DXF raw-document receipt

Status: local gates passed 2026-07-27; checkpoint CI receipt is the annotated
tag and GitHub Actions run.

Base checkpoint:

- commit: `15a42f303200a030c828eb03a7b061aeff66c351`;
- tag: `m5.1b-binary-group-cursor`.

## Accepted result

M5.2a adds `DxfBinaryRawDocument::open`. A bounded exact canonical-opening
probe selects the pre-R13 one-byte/XDATA-escape or R13-and-later two-byte
little-endian group-code encoding. The complete stream is then framed and
hashed in one sequential pass. Exactly one supported HEADER `$ACADVER` must
agree with that physical encoding; failure is typed and no alternate parse is
guessed.

The immutable document retains `SourceId`, source length, verified encoding,
the existing provenance-rich dialect report, and compact metadata for every
physical group. Exact code/value/payload/full spans remain source-backed.
Metadata is at most 40 bytes per record and is stored in independent
4,096-record chunks, avoiding gigabyte-scale contiguous reallocation. The
25-million-record profile therefore bounds raw group payload storage to at
most 1,000,000,000 bytes before small chunk/allocation metadata; this is a
design bound, not the later 2 GiB release benchmark receipt.

The shared ASCII/Binary dialect observation path keeps all existing ASCII
behavior and tests unchanged. Stable fatal codes `DXF-E0216` through
`DXF-E0218` cover invalid canonical opening, unusable Binary `$ACADVER`, and
physical encoding/dialect disagreement. Strict and Compatible remain
identical for Binary input.

Production scope is 365 lines in the new document module plus net production
deltas of 31 dialect, 49 error, five span-constructor, and two module/export
lines: 452 production lines. Five ASCII helper lines change visibility only
and add no production lines. No dependency, `Cargo.toml`, or `Cargo.lock`
change is present.

## Oracle evidence

The committed oracle audit opens three external files created by AutoCAD Core
Console 2027 without passing an encoding hint. It checks exact encoding,
supported dialect, group count, full-source SHA-256 identity, continuous raw
accounting, and first `SECTION`/last `EOF` payloads.

- AC1009: one-byte encoding, 535 groups, 3,926 bytes.
- AC1015: two-byte encoding, 10,281 groups, 112,153 bytes.
- AC1032: two-byte encoding, 5,898 groups, 42,235 bytes.
- Combined: 16,714 groups and 158,314 bytes, all checks passed.

The checker, build output, result log, AutoCAD executable, and CAD fixtures
remain outside the repository; their SHA-256 values are frozen in the audit.
Synthetic tests cover all nine supported AC1009-AC1032 dialect tokens. This is
raw opening/dialect evidence, not the deferred section/EOF, replay, CLI,
semantic, edit, or writer claim.

## Verification

- Eight focused Binary document tests pass.
- All nine supported dialects, both physical encodings, exact SHA-256/source
  spans, file/memory parity, progress, cancellation, malformed opening,
  missing/invalid/unsupported/duplicate `$ACADVER`, encoding disagreement,
  truncated payload, every wire value shape, 40-byte metadata cap, and access
  across the 4,096-record chunk boundary are tested.
- `cargo test -p seacad-dxf-core binary_document`: 8 passed.
- `cargo test --workspace`: 173 passed (162 core, 11 CLI).
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- Production forbidden-construct scan: zero unsafe blocks/functions or
  `panic!`, `unwrap()`, `expect()`, `todo!`, and `unimplemented!` hits;
  crate-level `#![forbid(unsafe_code)]` remains active.
- `git diff --check`: passed.

The GitHub Actions matrix must pass on Windows x64, macOS ARM64, and Linux x64
before the checkpoint is reported complete.

## Artifact hashes

| Artifact | SHA-256 |
| --- | --- |
| `crates/seacad-dxf-core/src/binary_document.rs` | `155c6991a794d5379e2d2931c995e07a743fdffe476a4d7a62d1a1bed16a3b71` |
| `crates/seacad-dxf-core/src/ascii_document.rs` | `77cc37fd4c73cf41d1d23da60bdb9d2ec5cfc1aa1342a29163e0b36283c3f0d3` |
| `crates/seacad-dxf-core/src/diagnostic.rs` | `8cfff18e4289f75a5ab36a2ccb74327d1d92159a07f3ab0dde5c3135cb3db5a3` |
| `crates/seacad-dxf-core/src/dialect.rs` | `65238d147f2b553d1346026b9705ebf427e85b3316f75d5d769ddf20e2ed5357` |
| `crates/seacad-dxf-core/src/error.rs` | `9cb4faf092e3d4d4335a9664ca5e672187cd3716e30c3597a77ff33741b0ae51` |
| `crates/seacad-dxf-core/src/lib.rs` | `4aab180302a0655147069669fbca511d3df6c4bb609e6b663c90a2b2330b3614` |
| `docs/M5_2A_BINARY_RAW_DOCUMENT_CONTRACT.md` | `b0f2d261667caadcb1ae729cc7275c33bd57560b71858a8a606de3a4ad8a4523` |
| `docs/audits/M5_2A_AUTOCAD_BINARY_DOCUMENT_ORACLE.md` | `0800294566a02f3122eb47f49214d7dc9d44011c7e89b8127b40c12bff7592bf` |
| `docs/IMPLEMENTATION_PLAN.md` | `ea3538b49125969890afe72c93e764b2a1bed567120bffbc16e1d151e6d54918` |
| `docs/SUPPORT_MATRIX.md` | `832acc2cf58c76147b8633c415e546e17ca458f2d11758b5babf14cdc9886b73` |

The receipt does not hash itself. The annotated checkpoint tag binds this
receipt and every listed artifact to the final commit.
