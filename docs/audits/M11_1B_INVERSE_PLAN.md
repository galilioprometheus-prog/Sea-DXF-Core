# M11.1b Executable Inverse Plan Audit

## Outcome

M11.1b verifies an independently opened post-image against an M11.1a forward
plan and materializes a normal immutable transaction plan that reverses the
exact raw-byte change. The inverse is bound to the post-image `DxfSourceId`,
length, and physical format, so later M12 writers can consume it through the
same plan contract.

## Contract

- The original document must satisfy the forward plan's source precondition.
- The post-image must have the projected length and same ASCII/Binary format.
- Unchanged source ranges and replacement payloads are compared to the
  post-image in fixed 4-KiB chunks.
- Length mismatch is stable `DXF-E1103`; content/format mismatch is stable
  `DXF-E1104` with only the first differing post-image span.
- Forward replacement spans map to inverse source spans without copying the
  whole source or post-image.
- Captured original bytes become inverse replacement bytes.
- Adjacent forward deletions that collapse to one post-image offset are
  coalesced in original source order.
- The inverse projects the original source length and can itself be
  materialized back into an exact redo plan.
- Cancellation and selected resource-profile limits remain fail-closed.

## Evidence

- Forward/inverse byte-identical round trips cover ASCII and Binary across all
  nine supported AC1009-AC1032 dialects.
- Mixed insertion, adjacent deletion, replacement, and boundary insertion
  coverage proves coalescing and exact redo behavior.
- Two adjacent 600,000-byte raw-group deletions prove inverse/redo closure when
  the coalesced derived patch exceeds one Safe-profile raw-value limit while
  remaining inside the snapshot limit.
- A 9,000-byte payload crosses multiple verifier chunks and reports the exact
  changed byte.
- Tests retain distinct source-identity, post-image length, post-image content,
  and cancellation failures.

## Non-claims

M11.1b does not write or apply a plan, accept a post-image that cannot be opened
by the raw-document layer, validate higher-level edit intent, allocate
handles, choose or replace a filesystem destination, or publish a snapshot.

## Gates

- `cargo deny --locked check`
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace --quiet` (571 passed, 0 failed, 0 ignored)
- `git diff --check`

All gates passed.

## Artifact receipt

| Artifact | Lines | SHA-256 |
| --- | ---: | --- |
| `crates/seacad-dxf-core/src/transaction_inverse.rs` | 346 | `42a398f350c86aeceb697e1c6d91b082a945484a45bb5687a0319fbbf8ff126c` |
| `crates/seacad-dxf-core/src/transaction_plan.rs` | 563 | `8b01200a89dc4394adcf300a38f13e134337dd4b4615fd0c04647a5581eb0b29` |
| `crates/seacad-dxf-core/src/error.rs` | 657 | `f425c715307134519c7bf754cb0d3d99facb800b4e7bce6c55f850b31765a10d` |
| `crates/seacad-dxf-core/src/lib.rs` | 611 | `7a7d532b755353c2628c7d8d5c1d57ba8a2e4818d6e703a86bd0aac4c031f8ff` |
| `crates/seacad-dxf-core/tests/transaction_inverse_tests.rs` | 461 | `89a34d02a2d0f09c713dcfa10c4e75b01ab16439021110244b9cbc164beb1a54` |
| `docs/IMPLEMENTATION_PLAN.md` | 1158 | `31e0f3be60f5b9c8e85aa3cbfed128cc8aa267f3cd7cc26f521038e26066ae8b` |
| `docs/SUPPORT_MATRIX.md` | 949 | `54fb2902b8801cbdab36bc8134dc7a6b4cb0105105cb563da84a4d46627d8eb5` |
