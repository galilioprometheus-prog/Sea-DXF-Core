# M12.1a Verified Transaction Writer Audit

## Outcome

M12.1a applies an immutable M11 transaction plan to a path that must not
already exist. Application is bounded and streaming; the complete live source
and projected output are independently hashed, and the synced output is
reopened for exact length and identity verification before a receipt escapes.

## Contract

- The plan validates source identity, source length, and physical format before
  destination creation.
- The destination uses create-new semantics and an existing path is never
  modified.
- Unchanged source ranges and patch replacement bytes are emitted in exact
  source order.
- Replaced/deleted source ranges are still read and hashed, so the complete
  live source is checked rather than trusting open-time identity alone.
- Reads and verification use fixed 64-KiB buffers independent of source size.
- The projected output is hashed while written, then flushed and synced.
- Reopen verification requires exact projected length and the same output
  identity observed during writing.
- Progress covers source application plus output verification and remains
  monotonic from zero through the exact combined work.
- Cancellation and failures after creation attempt to remove incomplete
  output. Cleanup failure replaces the primary error.
- The receipt retains source identity, output identity, projected byte count,
  and patch count without payload disclosure.

## Stable failures

- Existing I/O, cancellation, source-identity, and offset failures retain their
  stable error codes.
- `DXF-E1201` reports transaction-output length mismatch.
- `DXF-E1202` reports transaction-output identity mismatch.

## Evidence

- Verified create-new application of M11.2b handle-assignment plans covers
  ASCII and Binary across all nine supported AC1009-AC1032 dialects.
- Every output strictly reopens and its `DxfSourceId` matches the write receipt.
- Empty plans reproduce exact source bytes with a zero-patch receipt.
- Existing destinations remain byte-identical and mismatched source documents
  do not create an output.
- A source larger than two chunks proves observer cancellation removes the
  incomplete file.
- Public receipt bounds and stable-code coverage remain enforced.

## Non-claims

M12.1a does not replace an existing path, provide crash-atomic rename over a
destination, retain a filesystem journal, automatically open the result or
materialize its inverse, canonicalize ASCII or Binary, infer edit intent, or
publish a snapshot.

## Gates

- `cargo deny --locked check`
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace --quiet` (582 passed, 0 failed, 0 ignored)
- `git diff --check`

All gates passed.

## Artifact receipt

| Artifact | Lines | SHA-256 |
| --- | ---: | --- |
| `crates/seacad-dxf-core/src/transaction_write.rs` | 387 | `a42a2e3fc3ddc8a16f652fd927faf850e42c6271930c40f6d4f027493e583456` |
| `crates/seacad-dxf-core/src/error.rs` | 699 | `519d1e27a0ff1df0802bb693b6d798c698589e2a8596ad1016cb08ebe2ba4300` |
| `crates/seacad-dxf-core/src/lib.rs` | 622 | `170af6f1563f25784622ed5e86699fad87ab5590c8d12011a6142a74412e3375` |
| `crates/seacad-dxf-core/tests/transaction_write_tests.rs` | 323 | `d6329d6ffa249c4674246fda6fd6ca76aecb5146898b488242e9e1b0cc6f06dd` |
| `docs/IMPLEMENTATION_PLAN.md` | 1197 | `883f5d278557a0cb5e372909550ba6a657e0f3d3ef996dd7b3bb567ba423eb30` |
| `docs/SUPPORT_MATRIX.md` | 981 | `c7342a566134f19fafe58b7d4f954bd7dceeb680b2b3f1657c0490209a8c3963` |
