# M12.2a Canonical ASCII Framing Audit

## Outcome

M12.2a emits one deterministic strict ASCII framing for every opened ASCII raw
document. Group codes use minimal signed decimal spelling, every code and value
is LF-terminated, exact non-envelope value payload bytes remain authoritative,
and every reviewed Compatible EOF recovery closes to one terminal `0`/`EOF`.

## Contract

- Input is an already opened `DxfAsciiRawDocument`; its original strict or
  compatible source identity remains the receipt source identity.
- A bounded metadata preflight computes exact output length and group count
  before file creation.
- The selected profile limits source bytes, each retained value payload,
  output groups, and projected output bytes.
- Group codes use minimal signed decimal spelling with no padding.
- Each code and value is followed by exactly one LF.
- Non-EOF value payload bytes are copied exactly, including leading/trailing
  horizontal whitespace and source encoding.
- Strict terminal EOF remains terminal EOF.
- Recovered BOM/padded EOF is emitted as exact `EOF`; a missing EOF appends
  `0\nEOF\n`; opaque trailing bytes after recovered EOF are omitted.
- Streaming reads and hashes every live source byte in original order,
  including discarded BOM, line framing, and opaque tail evidence.
- The create-new output is hashed while written, flushed, synced, reopened for
  exact length/identity verification, and strictly reparsed.
- Any cancellation, source mismatch, I/O, resource, verification, or strict
  reparse failure removes the incomplete output.
- The receipt retains source/output identities, byte/group counts, and typed
  envelope action without payload disclosure.

## Stable failures

- Existing source, resource, I/O, cancellation, and identity errors retain
  their stable codes.
- `DXF-E1221` reports canonical-output length mismatch.
- `DXF-E1222` reports canonical-output identity mismatch.

## Evidence

- Deterministic exact output covers all nine supported AC1009-AC1032 ASCII
  dialects with mixed CR, LF, CRLF, padded group codes, and preserved
  whitespace-bearing value payloads.
- Strict, recovered padded/BOM EOF, missing EOF, and trailing opaque data all
  produce strictly reopenable outputs with exact receipt counts/actions.
- A value larger than two writer chunks proves bounded cancellation cleanup.
- Same-length live mutation of a source group-code/value region is detected by
  full-source rehash and leaves no output.
- Existing destinations remain byte-identical.
- Public receipt/action `Copy`, exact output identity, monotonic progress, and
  stable error codes are covered.

## Non-claims

M12.2a canonicalizes ASCII physical framing only. It does not normalize,
decode, select, round, or reinterpret text, numeric, handle, binary-chunk,
application, or semantic value payloads. It does not accept Binary input,
convert physical formats, replace an existing path, provide crash-atomic
rename, or publish a snapshot.

## Gates

- `cargo deny --locked check`
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace --quiet` (587 passed, 0 failed, 0 ignored)
- `git diff --check`

All gates passed.

## Artifact receipt

| Artifact | Lines | SHA-256 |
| --- | ---: | --- |
| `crates/seacad-dxf-core/src/canonical_ascii_write.rs` | 559 | `d98805a0ad68dc0fbf990976c604de15e0247086df8a708971743c2b76547d37` |
| `crates/seacad-dxf-core/src/error.rs` | 741 | `a78972861eec39917b0adfb3d4d4a65ba341e1e7f1f92aa62ac25fdd33ad0928` |
| `crates/seacad-dxf-core/src/lib.rs` | 624 | `56b23bd2733c27abee63053fab24b38edcd98b11b8e7fa618778b56eb4aac985` |
| `crates/seacad-dxf-core/tests/canonical_ascii_write_tests.rs` | 327 | `c2e2da94ba3d873e193164139cdbc270b80bb0430489a3a2002060f9369dafc0` |
| `docs/IMPLEMENTATION_PLAN.md` | 1222 | `5c6f83156ff8a8885c105901a2476fe6f996aadcc844f4b489acd3d243c2c121` |
| `docs/SUPPORT_MATRIX.md` | 999 | `567afdf2b7e6881041760ae695eb6df853d40ad7b8ea894457113ea84bf495ab` |
