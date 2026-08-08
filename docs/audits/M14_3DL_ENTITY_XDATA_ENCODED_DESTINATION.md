# M14.3dl Entity XDATA Encoded Destination Groups

Retrieved: 2026-08-08

## Contract

`DxfEntityXDataEncodedDestinationDirectory` owns M14.3dk logical destination
projection and canonically encodes every available occurrence as one complete
destination-format group. The shared entity group encoder supplies ASCII
framing, Binary wire widths, uppercase handle/chunk spelling, and the AC1009
extended-data group-code escape. No encoded group is inserted or applied.

Destination APPID/LAYER text comes from the exact validated destination record.
Source strings, controls, and chunks are read through bounded provenance;
transformed doubles, remapped handles, and integer values come from M14.3dk.
ASCII-only text is portable across reviewed storage policies. Non-ASCII source
text requires matching reviewed source/destination decoders or remains typed
`TextTranscodingRequired`. Logical, dialect, transcoding, and group-encoding
blockers publish no bytes.

## Verification boundary

Three focused tests cover all four ASCII/Binary source-destination pairings for
all nine Core dialects plus AC1009/AC1032 cross-dialect pairs. Exact checks cover
ASCII/Binary framing, AC1009 extended-data escapes, binary chunks, handles,
transformed binary64 values, text/control/scalar/integer groups, matching and
mismatched legacy decoders, unavailable byte suppression, foreign destination
entries, cancellation, compact metadata, bounds, and non-disclosing debug
output.

This checkpoint does not interpret application-specific meaning, perform text
transcoding, group encoded entries by application or entity, insert XDATA,
mutate either document, complete a cross-container clone, or advance POINT or
another entity to `Complete`.

## Gate receipts

Focused encoded-destination tests passed 3/3; entity encoder/handle replacement/
logical/encoded regression suites passed 16/16. The workspace passed exactly
1,037 tests. Cargo-deny, formatting, schema and release-evidence checks,
workspace Clippy with warnings denied, production safety scan, protected-
surface diff, and `git diff --check` passed. The complete gate run finished in
about 276 seconds. No manifest, dependency, lockfile, schema, corpus, legal, or
release surface changed.

Production adds 562 lines across the encoded directory and public exports;
focused tests add 596 lines. This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 155 | `987abecfbe05336d61a7329cd4f4ffdee4978b38a4e2e11894fd8de728f3bb4d` |
| `README.vi.md` | 154 | `9b019e565350690509aa7093c562968b89352f0c4df8e0abf98a72e8c7d3320d` |
| `crates/seacad-dxf-core/src/entity_xdata_encoded_destination.rs` | 557 | `0fbc6fc07fa32680d501c19595b9aceb0429683e86b9b2c39200681b5204769b` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,209 | `0b1fc913b1aea3a4c5f8f45b79908c02a4945683125eb906ad5a4eaf087c87f9` |
| `crates/seacad-dxf-core/tests/entity_xdata_encoded_destination_tests.rs` | 596 | `5e37a0db8660e1adff5215fdf46703b9159801f8072396802f3c4594c20b5291` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `5b28eaa94f6d4d699f8a1051d108c3e37f659ccc8a07e832157290b99a97ce6e` |
| `docs/SUPPORT_MATRIX.md` | 2,732 | `b58deadcc8b57a719fc36259e9b8973d6f64d31a69131fb861697e1377eb0ac2` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,098 | `1b2fd4b5bf6e1c7784eee5849b236bab3b01bc77050dad29385aa58574b8a561` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 1,675 | `ce2f769802135f2a138ea5ce7540b9e5281f3693ba8a7baa6fb2fdef1e1c5e58` |
