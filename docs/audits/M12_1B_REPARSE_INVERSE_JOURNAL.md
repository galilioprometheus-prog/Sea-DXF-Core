# M12.1b Strict Reparse and Inverse Journal Audit

## Outcome

M12.1b closes the preserve-patch write loop for create-new files. After M12.1a
streams and verifies an output, the new API strictly reparses that exact
physical format and materializes an executable M11.1b inverse plan. A journal
escapes only when the write receipt, reparse identity, full post-image
verification, and inverse construction all agree.

## Contract

- M12.1a source preconditions, create-new behavior, live-source hashing,
  output verification, sync, cancellation, and cleanup remain unchanged.
- Reparse uses the plan's exact ASCII or Binary format, strict framing, and the
  caller-selected resource profile.
- An internal no-op observer isolates reparse progress from the public M12.1a
  source-application plus output-verification interval.
- The reparsed `DxfSourceId` must equal the independently verified write
  receipt before inverse materialization.
- M11.1b compares every unchanged range and replacement byte against the
  reparsed post-image before constructing the inverse.
- The returned journal owns one `DxfTransactionWriteReceipt` and one immutable
  inverse `DxfTransactionPlan`.
- Applying that inverse through M12.1b strictly restores the original bytes and
  returns an exact redo journal.
- Any strict-open, identity, post-image, inverse, resource, I/O, or
  cancellation failure after creation removes the output and returns no
  journal.

## Evidence

- Forward write, strict reparse, inverse materialization, inverse write, strict
  restoration reparse, and redo-journal construction cover ASCII and Binary
  across all nine supported AC1009-AC1032 dialects.
- Restored files are byte-identical to the original source.
- Inverse source identity equals the forward receipt output identity; restored
  receipt output identity equals the original source identity.
- Redo source identity equals the restored original and its projected length
  equals the forward post-image.
- A transaction whose raw output passes M12.1a length/hash verification but
  replaces terminal `EOF` with `EOG` fails strict reparse and leaves no file.
- Public journal ownership, accessors, `into_parts`, and `Send + Sync` bounds
  are exercised.

## Non-claims

M12.1b does not replace an existing path, provide crash-atomic destination
rename, retain an external filesystem recovery journal, accept compatible
framing recoveries on output, canonicalize ASCII or Binary, infer edit intent,
or publish a snapshot.

## Gates

- `cargo deny --locked check`
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace --quiet` (583 passed, 0 failed, 0 ignored)
- `git diff --check`

All gates passed.

## Artifact receipt

| Artifact | Lines | SHA-256 |
| --- | ---: | --- |
| `crates/seacad-dxf-core/src/transaction_write.rs` | 493 | `6f24cf5eaec2b7c4365cf5d0c033af86aa761b0f8b8b6fa83f8e61bfaa4b02c4` |
| `crates/seacad-dxf-core/src/lib.rs` | 622 | `345517f16dc5edbda8427401803f4b80699c7067021681a1fc7da8e48d332ccc` |
| `crates/seacad-dxf-core/tests/transaction_write_tests.rs` | 418 | `2cb27d574894f9130669731d6efccffb71f5da2d1227683709754addbbe8ecf7` |
| `docs/IMPLEMENTATION_PLAN.md` | 1209 | `e5ce0b4838ec28f43bda50bf25bea9a45121999be85ef18f98ebc8eca20ccc84` |
| `docs/SUPPORT_MATRIX.md` | 990 | `a18b144fd5556c2d55ecf931141d2be56a16ad6d0b5b7344e335b3e9289b3cfd` |
