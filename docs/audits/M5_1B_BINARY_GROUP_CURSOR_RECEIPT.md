# M5.1b Binary DXF group-cursor receipt

Status: local gates passed 2026-07-27; checkpoint CI receipt is the annotated
tag and GitHub Actions run.

Base checkpoint:

- commit: `570bfc1591ccae78e0ea23d961c6bab260434ec6`;
- tag: `m5.1a-binary-wire-registry`.

## Accepted result

M5.1b adds an 8 KiB buffered cursor over immutable file-backed or memory-backed
Binary DXF sources. The caller supplies the M5.1a physical encoding. Every
successful pair retains occurrence, typed code/family, exact borrowed code and
value wire bytes, and exact code/value/payload/full source spans.

NUL strings and length-prefixed chunks remain lossless while their payload
spans exclude framing bytes. Fixed, string, and chunk values obey the selected
value budget; records and sources obey the selected profile. Partial reads,
cancellation, premature source end, malformed values, and reserved group codes
fail closed with stable typed errors. No semantic recovery or guessing was
added.

Production scope is 428 lines before `#[cfg(test)]` in the new cursor module,
56 production error-contract lines, and two module/export lines: 486 production
lines total. No dependency, `Cargo.toml`, or `Cargo.lock` change is present.

## Oracle evidence

The committed oracle audit frames three external files produced by AutoCAD
Core Console 2027 using only the current SeaCad path dependency. For every
group, the checker rereads the original full span and requires exact equality
with `raw_group_code || raw_value`; spans must be continuous from byte 22 to
physical source end.

- AC1009 one-byte encoding: 535 groups, 3,926 bytes.
- AC1015 two-byte encoding: 10,281 groups, 112,153 bytes.
- AC1032 two-byte encoding: 5,898 groups, 42,235 bytes.
- Combined: 16,714 groups and 158,314 bytes with no gap, overlap, unsupported
  family, or raw mismatch; each stream begins with `SECTION` and ends with
  `EOF`.

All fixture, AutoCAD executable, checker, runner, and result-log SHA-256 values
are frozen in the audit. External fixtures and tools remain outside the repo.
This is physical framing evidence, not a Binary document, semantic, CLI, or
unchanged-writer claim.

## Verification

- Ten focused cursor tests pass.
- Every value family, both group-code encodings, XDATA escapes, raw
  reconstruction, exact spans, invalid sentinel/header, reserved gaps,
  truncated fixed/chunk values, unterminated strings, record/value limits, an
  exact-limit NUL string, an 8 KiB boundary crossing, partial one-byte source
  reads, buffered read count, cancellation-after-read, hostile declared length,
  non-canonical Boolean and maximum XDATA-chunk framing without semantic
  guessing, empty stream, and debug redaction are tested.
- `cargo test -p seacad-dxf-core binary_group`: 10 passed.
- `cargo test --workspace`: 165 passed (154 core, 11 CLI).
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
| `crates/seacad-dxf-core/src/binary_group.rs` | `fc6d85f1d3c219845f15acb8c41431ce48cc9c8bb8406d2818a20f159469e53b` |
| `crates/seacad-dxf-core/src/error.rs` | `b3109354c469e73c7c7d925aad6669d17c95966d3f0dfc3e2b8cbb300751257d` |
| `crates/seacad-dxf-core/src/lib.rs` | `71e4c9c1852dc63246bb83a522c8c3234b73a69bbcae4ee023d0ef0f18cef114` |
| `docs/M5_1B_BINARY_GROUP_CURSOR_CONTRACT.md` | `7776a3ffc2623f16be44dd6f6a73e23482b3d21541e43c9ba73323113f5351db` |
| `docs/audits/M5_1B_AUTOCAD_BINARY_FRAMING_ORACLE.md` | `75dc9e853ee909fa83551dadbf849d58d2ae35fed7e016f6ddc08e0f6c9457cd` |
| `docs/IMPLEMENTATION_PLAN.md` | `1cd12d4313b9b2f3ba13bf4b299b68968bfcf22929c0c82b6209c758eaf3ac72` |
| `docs/SUPPORT_MATRIX.md` | `42961c5952c81a0ad1188bcf0786ffaea5299062649449c148784b23b89449bb` |

The receipt does not hash itself. The annotated checkpoint tag binds this
receipt and every listed artifact to the final commit.
