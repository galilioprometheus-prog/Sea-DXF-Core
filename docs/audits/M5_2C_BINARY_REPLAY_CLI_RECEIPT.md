# M5.2c Binary DXF replay and CLI receipt

Status: local gates passed 2026-07-27; checkpoint CI receipt is the annotated
tag and GitHub Actions run.

Base checkpoint:

- commit: `5404bc02d89ff69f89061ac8f3fe056c305781ba`;
- tag: `m5.2b-binary-envelope-index`.

## Accepted result

`DxfBinaryRawDocument::write_verbatim_to_new_file` now uses the same reviewed
implementation and `DxfVerbatimWriteReceipt` as ASCII. A private document trait
shares only source length, SHA-256 identity, and exact span reads; there is no
format-dependent re-encoding path. The writer copies the complete source in
64 KiB chunks, including Compatible missing-EOF or opaque trailing bytes,
requires the source identity observed during copying to match the immutable
snapshot, flushes and synchronizes a create-new destination, then reopens and
verifies output length and SHA-256. Existing destinations are never modified,
and incomplete new outputs are removed on failure when possible.

`seacad inspect` and `seacad verify` now open Binary physical input through the
same stable JSON v1 report shape used by ASCII. Strict Binary reports `ok` or
`verified`. Compatible recovery is inspect/verbatim-only: inspect reports
`recovered`, while verify reports `not_verified` and `CLI-E0004`. Fatal Binary
framing/envelope errors preserve their stable `DXF-E0210` through `DXF-E0220`
codes. English/Vietnamese help now describes ASCII/Binary support, Vietnamese
human output covers every currently reachable Binary fatal code, and paths
remain hidden unless explicitly requested.

The two production files change 359 production lines before their test modules
(240 additions and 119 removals, net 121). Most core churn moves the reviewed
ASCII writer into one shared private implementation. No dependency,
`Cargo.toml`, or `Cargo.lock` change is present.

## Oracle evidence

The committed oracle audit opens and replays three external Binary files made
by AutoCAD Core Console 2027: AC1009, AC1015, and AC1032. All 158,314 bytes have
identical input/output SHA-256, independent whole-file byte comparisons pass,
and a second write to each existing destination is refused without changing
it. The actual CLI executable returns JSON v1 `ok`/`verified`, exact group and
EOF counts, empty diagnostics, physical `binary`, and a redacted path for all
three files.

Synthetic committed tests cover Strict replay, Compatible missing EOF and
trailing bytes, JSON shape, stable recovery/fatal codes, Vietnamese fatal text,
and path redaction. This evidence is raw preservation and inspection only; it
does not claim Binary semantic values, edits, conversion, PreservePatch, or
canonical writing.

## Verification

- `cargo test -p seacad-dxf-core verbatim --quiet`: 10 passed.
- `cargo test -p seacad-cli --quiet`: 12 passed.
- `cargo test --workspace --all-targets`: 178 passed (166 core, 12 CLI).
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- Production forbidden-construct scan: zero unsafe blocks/functions or
  `panic!`, `unwrap()`, `expect()`, `todo!`, and `unimplemented!` hits; the
  crate-level `#![forbid(unsafe_code)]` remains active.
- `git diff --check`: passed.
- External checker: three files, 158,314 bytes, byte-identical replay and
  overwrite refusal passed.
- Actual CLI: six commands, three inspect plus three verify, passed.

The GitHub Actions matrix must pass on Windows x64, macOS ARM64, and Linux x64
before the checkpoint is reported complete.

## Artifact hashes

| Artifact | SHA-256 |
| --- | --- |
| `crates/seacad-dxf-core/src/verbatim.rs` | `1399cf5cd7c81f9996e1ec7b73e5f12bb15d94ff9cdc377277d15f0c24f8d146` |
| `crates/seacad-cli/src/main.rs` | `4ed35b07b8884d017d5d84fd6b62a52822d333e04820a3948ae894b479c264a1` |
| `docs/M5_2C_BINARY_REPLAY_CLI_CONTRACT.md` | `13c3008e5f61ce39705db7b82f845a5b1198628de30d1e4b4fc830cbfcdbeb72` |
| `docs/audits/M5_2C_BINARY_REPLAY_CLI_ORACLE.md` | `54774922cdd4a7f07c645f934c0962200f5984832ad02095c8bf378efaa3d613` |
| `docs/CLI_JSON_V1.md` | `6694adce28dbe313b2ee68dc912bd98f210ed2b4d3e1cf3841c5f5f50e6f3398` |
| `docs/IMPLEMENTATION_PLAN.md` | `e75cf0c6fb58c8cd4e9191f2836c821013fa8dd449766f59b4f2a1bb23fa97b0` |
| `docs/SUPPORT_MATRIX.md` | `7c1212b5d419edf4da991ae26b4e807deeb27e2fe2e36a08100eaa36c9801f75` |

The receipt does not hash itself. The annotated checkpoint tag binds this
receipt and every listed artifact to the final commit.
