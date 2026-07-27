# M5.2b Binary DXF envelope/index receipt

Status: local gates passed 2026-07-27; checkpoint CI receipt is the annotated
tag and GitHub Actions run.

Base checkpoint:

- commit: `636d384e528b1d9984a6a846570f69b14fbf0505`;
- tag: `m5.2a-binary-raw-document`.

## Accepted result

M5.2b makes the exact Binary group-code 0 payload `EOF` the terminal envelope.
Strict returns stable `DXF-E0219` for a missing marker and `DXF-E0220` for any
physical tail. Compatible never changes case, trims text, or guesses a marker:
it only records `DXF-W0210` for a completely framed stream without EOF or
`DXF-W0211` plus one exact opaque span for bytes following the first exact EOF.
The complete accepted source, including a compatible tail, remains covered by
the document SHA-256 identity.

The ASCII structure tracker now consumes one internal format-neutral raw-group
observation. Binary calls the same state machine and exposes public Binary type
aliases for group ranges, section kinds/names/closures, section metadata, and
the structure index. Every accepted group is accounted inside or outside
non-overlapping sections; every numeric group code 0 is indexed. Unknown names
retain exact spans, while malformed section topology remains openable with the
existing bounded diagnostics.

Production delta is 168 lines: 20 shared tracker, 102 Binary document, 12
diagnostic, 22 fatal-error, and 12 public alias/export lines. This is slightly
below the 200-line guideline because the checkpoint deliberately reuses the
reviewed state machine instead of duplicating a Binary implementation. No
dependency, `Cargo.toml`, or `Cargo.lock` change is present.

## Oracle evidence

The committed audit reopens the three external AutoCAD Core Console 2027
fixtures and requires strict conformance, exact source/dialect/group evidence,
EOF as the final occurrence, no tail or diagnostic, complete inside/outside
accounting, exact section-range sums, and Closed closure for every section.

- AC1009: 535 groups, 4 sections, 41 group-zero records.
- AC1015: 10,281 groups, 6 sections, 225 group-zero records.
- AC1032: 5,898 groups, 7 sections, 171 group-zero records.
- Combined: 16,714 groups, 17 sections, and 437 group-zero records.

All 16,711 non-EOF groups are inside the 17 closed sections; the three exact
terminal EOF groups are outside. Checker, build, log, AutoCAD executable, and
CAD fixtures remain outside the repository with hashes frozen in the audit.

## Verification

- Eleven focused Binary document tests pass; three are new for M5.2b.
- Synthetic evidence covers known and unknown section names, exact name spans,
  every accounting total, group-zero index, Interrupted/Unclosed/orphan
  topology diagnostics, strict missing/trailing EOF, Compatible missing/tail
  receipts, opaque tail readback, and case-sensitive non-matching `eof`.
- Six ASCII index regression tests pass through the shared observation path.
- `cargo test -p seacad-dxf-core binary_document`: 11 passed.
- `cargo test -p seacad-dxf-core ascii_index`: 6 passed.
- `cargo test --workspace`: 176 passed (165 core, 11 CLI).
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
| `crates/seacad-dxf-core/src/ascii_index.rs` | `dc0f6158047be7b05f0a13e406768ebd386754380724f43849fa8249e712d466` |
| `crates/seacad-dxf-core/src/binary_document.rs` | `fcba488d7ac290002c08902a0daba268e45a3672dfe8091e3ca5f0a74b692710` |
| `crates/seacad-dxf-core/src/diagnostic.rs` | `ec22736a8aa9bd8817556fa54a744a11c392dd3e98bd7ced54b5c96e12560387` |
| `crates/seacad-dxf-core/src/error.rs` | `ed29157dd7e4cfb52abd9053888e5c222cc531798f5138d9424ab0ee151ad221` |
| `crates/seacad-dxf-core/src/lib.rs` | `0e690341608e162ff648785a7f837e2f7df5bd726e342f55666379cdd7b4ee2e` |
| `docs/M5_2B_BINARY_ENVELOPE_INDEX_CONTRACT.md` | `4241ec2814792b3b8d3837cc9c4b81b3cac9bc6c14f9492a334c9a20d1c55acd` |
| `docs/audits/M5_2B_AUTOCAD_BINARY_ENVELOPE_ORACLE.md` | `c56274044d05a6cb8dcc5e0a4b1443f9ad265ae43e4319b199674ed4f0e42e02` |
| `docs/IMPLEMENTATION_PLAN.md` | `d84444922e0f9b447ed85a5a15479c68205799f20014ffcad856c5076d490fdf` |
| `docs/SUPPORT_MATRIX.md` | `4e6290a0f849ca3e2fe327e4060473dfecb4d66d86055d0057701429f04e8421` |

The receipt does not hash itself. The annotated checkpoint tag binds this
receipt and every listed artifact to the final commit.
