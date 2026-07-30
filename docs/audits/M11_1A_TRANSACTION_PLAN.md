# M11.1a Immutable Transaction Plan Audit

## Outcome

M11.1a adds the first transaction-layer primitive described by the architecture
and implementation plan: a bounded mutable construction boundary that freezes
into an immutable raw-byte patch plan. The plan is tied to the opened
`DxfSourceId`, source length, and physical ASCII/Binary format.

## Contract

- Replace, delete, and insertion patches retain exact source spans.
- Replacement payloads are owned by the plan.
- Original source bytes are captured before a patch is admitted and retained
  as inverse evidence.
- Published patches are in deterministic source order even when added out of
  order.
- Out-of-bounds spans fail with stable `DXF-E1101`; nonempty overlaps,
  insertions strictly inside replaced spans, and duplicate insertions at one
  offset fail with stable `DXF-E1102`.
- Insertions exactly at replacement boundaries remain deterministic.
- Failed bounds, conflict, resource, I/O, or cancellation checks do not change
  patch count or projected length.
- Per-patch bytes, patch count, accumulated payloads, and projected snapshot
  size reuse the selected resource profile.
- Payload bytes are excluded from `Debug`.

## Evidence

- ASCII/Binary plan parity covers all nine supported AC1009-AC1032 dialects.
- Integration tests cover out-of-order construction, source-order publication,
  replacement/insertion/deletion length accounting, exact inverse capture,
  boundary insertion, overlap and duplicate rejection, source mismatch,
  cancellation, resource limits, no-op behavior, trait bounds, and debug
  redaction.
- The existing source-backed adapter performs exact bounded span reads; no
  whole-document copy is introduced.

## Non-claims

M11.1a does not apply a plan, calculate a post-image source identity,
materialize a directly executable inverse plan, validate the resulting DXF
syntax or semantics, allocate handles, write a destination, or publish a new
snapshot. Those remain later M11/M12 checkpoints.

## Gates

- `cargo deny --locked check`
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace --quiet` (565 passed, 0 failed, 0 ignored)
- `git diff --check`

All gates passed.

## Artifact receipt

| Artifact | Lines | SHA-256 |
| --- | ---: | --- |
| `crates/seacad-dxf-core/src/transaction_plan.rs` | 435 | `0b60172d99f742d0094030ceae62e46e50f1d12d3be139c5a5297b3dae1ba84d` |
| `crates/seacad-dxf-core/src/error.rs` | 615 | `4bc61e572a417f739d39797661ed4c39ccc641e2a2c39b0dff3acd3fd83adf2f` |
| `crates/seacad-dxf-core/src/lib.rs` | 610 | `5f1bfdd0733bfd957736ee93607926f51210ac93be72b0a3c91bbaedb926c1be` |
| `crates/seacad-dxf-core/tests/transaction_plan_tests.rs` | 345 | `91df13b20edaf5bcf32b71bec1d4017898020113f8f914f28bfdc88241144d47` |
| `docs/IMPLEMENTATION_PLAN.md` | 1143 | `44685a48d82256c94106abc29a2ea43f38601b99bd6e964213bc023dfe448f77` |
| `docs/SUPPORT_MATRIX.md` | 940 | `e17895d48d05e99c26e358d21823ab4396f9a952dabdbf94aa390e96da7f5e75` |
