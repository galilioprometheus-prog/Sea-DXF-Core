# M4.3b1 `encoding_rs` adoption receipt

Status: approved and added 2026-07-27

This receipt completes the conditional review begun in M4.3a. It applies only
to SeaCad's replacement-free byte-to-UTF-8 wrapper and the exact registry
frozen in `M4_3B1_CODEPAGE_DECODER_CONTRACT.md`.

## Locked package and feature tree

| Field | Adopted value |
| --- | --- |
| crate | `encoding_rs` |
| exact version | `0.8.35` |
| Cargo checksum / archive SHA-256 | `75030f3c4f45dafd7586dd6780965a8c7e8e285a5ecb86713e63a79c5b2766f3` |
| declaration | `default-features = false`, `features = ["alloc"]` |
| license | `(Apache-2.0 OR MIT) AND BSD-3-Clause` |
| new transitive packages | none |
| resolved dependency | existing `cfg-if 1.0.4` |

`cargo tree -p seacad-dxf-core -e features` shows only
`encoding_rs feature "alloc"` and its existing `cfg-if` dependency. SIMD,
serde, fast encoding tables, and every encode-only optional feature remain
disabled. Cargo.lock pins the package and checksum above.

## Called surface and safety boundary

SeaCad calls only:

- reviewed static `Encoding` values;
- `new_decoder_without_bom_handling()`;
- `decode_to_utf8_without_replacement(..., last = true)`.

SeaCad never calls replacement-producing or allocating convenience decode
methods. The wrapper accepts caller-owned source/destination slices, returns
read/written counts, treats `Malformed` as terminal, and exposes only the
logically written prefix.

The dependency contains internal `unsafe` optimized code, while SeaCad's own
production module remains covered by `#![forbid(unsafe_code)]`. Focused tests
cover every selected mapping, malformed UTF-8 and Shift_JIS, small output
buffers, and empty input. Fuzz coverage remains required at the M5/M13 gates;
this adoption is not an audit of every internal dependency path.

## License and provenance

The package archive was obtained through Cargo and is not vendored. Its
`COPYRIGHT` identifies the Mozilla Foundation for `encoding_rs` and WHATWG
(Apple, Google, Mozilla, Microsoft) for derived encoding data. The applicable
Apache-2.0-or-MIT notice and WHATWG BSD-3-Clause terms are recorded in
`THIRD_PARTY_NOTICES.md`; distributable license-file packaging remains an M13
release gate.

## Point-in-time advisory check

On 2026-07-27:

- GitHub Advisory Database query for Rust package
  `encoding_rs@0.8.35` returned zero advisories;
- RustSec advisory-db code search returned one textual hit,
  `RUSTSEC-2021-0153`, which concerns the different unmaintained crate
  `encoding` and names `encoding_rs` as an alternative.

These are point-in-time results and must be rerun during supply-chain and
release gates.

## Decision

Approved for the M4.3b1 decoder subset. This does not approve automatic
fallbacks, lossy decoding, encoding, DOS/OEM emulation, or a complete
historical-codepage claim.
