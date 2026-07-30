# M12.2b Canonical Binary Framing Audit

## Outcome

M12.2b emits one deterministic strict Binary framing for every opened Binary
raw document. The exact canonical sentinel and declared dialect's group-code
encoding are emitted, exact non-EOF value wire bytes remain authoritative, and
every reviewed Compatible EOF recovery closes to one terminal group-code `0`
with value `EOF\0`.

## Contract

- Input is an already opened `DxfBinaryRawDocument`; its original strict or
  compatible source identity remains the receipt source identity.
- A bounded metadata preflight computes exact output length and group count
  before file creation.
- The selected profile limits source bytes, every value payload, output groups,
  and projected output bytes.
- Output begins with the exact 22-byte `DXF_BINARY_SENTINEL`.
- AC1009 group codes use one byte for codes `0..=254` and byte `255` followed by
  signed little-endian `i16` for reviewed XDATA codes `1000..=1071`.
- AC1012 through AC1032 group codes use signed little-endian `i16`.
- Non-EOF value wire bytes are copied exactly, including string NULs,
  binary-chunk length prefixes/payloads, Boolean bytes, and numeric bit
  patterns.
- Strict terminal EOF remains terminal EOF; missing EOF is appended and opaque
  trailing bytes after recovered EOF are omitted.
- Streaming reads and hashes every live source byte in original order,
  including the original sentinel, group-code spans, and discarded opaque
  tail evidence.
- The create-new output is hashed while written, flushed, synced, reopened for
  exact length/identity verification, and strictly reparsed with the same
  selected profile and group-code encoding.
- Any cancellation, source mismatch, I/O, resource, verification, or strict
  reparse failure removes the incomplete output.
- The receipt retains source/output identities, byte/group counts, selected
  group-code encoding, and typed envelope action without payload disclosure.

## Stable failures

- Existing source, resource, I/O, cancellation, and identity errors retain
  their stable codes.
- `DXF-E1221` reports canonical-output length mismatch.
- `DXF-E1222` reports canonical-output identity mismatch.

## Evidence

- Exact output and strict reparse cover all nine supported AC1009-AC1032
  dialects and both reviewed group-code encodings.
- A dedicated fixture preserves every Binary value family byte-for-byte:
  NUL-terminated string, `f64`, `i16`, `i32`, `i64`, Boolean byte, and
  length-prefixed binary chunk, plus escaped XDATA.
- Strict, missing EOF, and trailing opaque data produce strictly reopenable
  outputs with exact receipt counts/actions.
- A value larger than two writer chunks proves bounded cancellation cleanup.
- Same-length live mutation is detected by full-source rehash and leaves no
  output.
- Existing destinations remain byte-identical.
- Public receipt/action `Copy`, receipt `Send + Sync`, exact output identity,
  monotonic progress, and selected encoding are covered.

## Non-claims

M12.2b canonicalizes Binary physical framing only. It does not normalize,
decode, select, round, or reinterpret text, numeric, Boolean, handle,
binary-chunk, application, or semantic value payloads. It does not accept
ASCII input, convert physical formats, replace an existing path, provide
crash-atomic rename, or publish a snapshot.

## Gates

- `cargo deny --locked check`
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace --quiet` (591 passed, 0 failed, 0 ignored)
- `git diff --check`

All gates passed.

## Artifact receipt

| Artifact | Lines | SHA-256 |
| --- | ---: | --- |
| `crates/seacad-dxf-core/src/canonical_binary_write.rs` | 606 | `c15287365a64f8b0527c636d94dc9ca169b0dbece4644147769a390267bbdf19` |
| `crates/seacad-dxf-core/src/lib.rs` | 628 | `8fe47d81eaaf7007265ff2e1128e5056913a6c8ab0ebc1a734d6283af6ea962d` |
| `crates/seacad-dxf-core/tests/canonical_binary_write_tests.rs` | 425 | `04e7e1f8414042cbcc05f3d49daddd5ffbdaff0d2f1c620c270d7cb5215965b2` |
| `docs/IMPLEMENTATION_PLAN.md` | 1236 | `9438f60f42110221bcd87332f8a18d2f034ad66c06080f9f6b0aa3474d39dd6b` |
| `docs/SUPPORT_MATRIX.md` | 1009 | `2c62d39a24b29f6c612f06090516b8e3d151a396443d40b036ee81e89fa73c3d` |
