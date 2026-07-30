# M11.2a Handle Allocation Policy Audit

## Outcome

M11.2a adds an immutable, source-bound policy directory that can propose a
bounded consecutive range of new object handles from exact `$HANDSEED` and
record-identity evidence. It fails closed before allocation when either source
of evidence is incomplete, malformed, contradictory, null, duplicated, or
stale.

## Normative evidence

- Autodesk documents HEADER `$HANDSEED` group 5 as the
  [next available handle](https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-A85E8E67-27CD-4C59-BE61-4DC9FADBE74A.htm).
- Autodesk states that object handles are
  [unique within a drawing and constant for an object's lifetime](https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-A0CC85BE-C044-4A4D-B20A-161C5D53FF6E.htm).
- Autodesk's `AcDbHandle` reference says handles are unique within one database
  and that databases
  [start the handseed at 1 and increment it](https://help.autodesk.com/cloudhelp/2018/ENU/OARX-RefGuide/files/OREF-AcDbHandle.html).

## Contract

- The document must contain exactly one parsed, nonzero `$HANDSEED`.
- Every record-local group-code 5/105 object-identity candidate must be
  absent or exactly one parsed, nonzero handle.
- Parsed identities must be globally unique.
- `$HANDSEED` must be strictly greater than the greatest occupied identity.
- Invalid and multiple identities are reported in raw-record source order;
  duplicate identities are reported in handle order.
- Missing, invalid, ambiguous, null, duplicate, and stale evidence remain
  distinct typed policy states.
- A ready policy proposes consecutive handles beginning at `$HANDSEED` and a
  representable next seed without allocating a handle-sized collection.
- A zero-count proposal preserves the current seed. Arithmetic exhaustion is a
  typed outcome, and request counts reuse the selected profile's record limit.
- The directory and every proposal retain the originating `DxfSourceId`.

## Evidence

- ASCII/Binary parity covers all nine supported AC1009-AC1032 dialects.
- Tests distinguish absent, invalid, ambiguous, and null `$HANDSEED` evidence.
- Tests distinguish invalid, multiple, null, duplicate, and seed-not-above-
  occupied identity failures.
- Tests cover an empty identity set, three consecutive handles, zero count,
  out-of-range lookup, `u64` successor exhaustion, resource limits,
  cancellation, source anchoring, and public bounded types.

## Non-claims

M11.2a does not treat pointer, owner, reactor, or arbitrary handle-reference
values as occupied object identities. It does not validate reference topology,
repair a stale seed, reuse handle gaps, reserve a proposal across calls, encode
an identity, update `$HANDSEED`, construct or apply a transaction, write a
destination, or publish a snapshot. Atomic handle and seed editing remains a
later M11 checkpoint.

## Gates

- `cargo deny --locked check`
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace --quiet` (575 passed, 0 failed, 0 ignored)
- `git diff --check`

All gates passed.

## Artifact receipt

| Artifact | Lines | SHA-256 |
| --- | ---: | --- |
| `crates/seacad-dxf-core/src/handle_allocation_policy.rs` | 346 | `4c6b37cba1fea48b2a11e8f8396c566ccf68f585a11550e53b7aa79dfb2ffc6c` |
| `crates/seacad-dxf-core/src/lib.rs` | 616 | `1e728951b7d8cf5050f45e322e7659cbdc37164a2a6a0691fe87c3c9d0e0a2fe` |
| `crates/seacad-dxf-core/tests/handle_allocation_policy_tests.rs` | 336 | `54f291cc3cb7b2fcf68f0a78fe572da5d3938055193c5e3d944e08d127d3a3f2` |
| `docs/IMPLEMENTATION_PLAN.md` | 1170 | `d0ec4f1873ce6e7b29068af45101c5a91ea72360224c4949fa96c380490069c9` |
| `docs/SUPPORT_MATRIX.md` | 959 | `5ccbaeaf86aa662f00bc2fc83ce898e91f1047263925c1b9d3541509290a1e0d` |
