# M8.4b RAY/XLINE Value Cardinality

Retrieved: 2026-07-29

## Evidence boundary

M8.4b layers deterministic per-role cards over the exact RAY/XLINE values
reviewed and retained by M8.4a. It introduces no new DXF group-code,
coordinate-system, direction, or extent interpretation. Normative facts and
primary Autodesk sources remain recorded in
`docs/audits/M8_4A_RAY_XLINE_EVIDENCE.md`.

Every recognized RAY and XLINE receives six cards even when its M8.4a value
slice is empty: WCS start/first-point X/Y/Z and unit-direction X/Y/Z.

## Implementation contract

`crates/seacad-dxf-core/src/infinite_line_geometry_card.rs` owns:

- one stable source-order card sequence grouped by raw record and documented
  role order;
- `Absent`, `Unique`, or `Multiple { occurrence_count }` state based only on
  the number of matching M8.4a occurrences;
- one compact member range per card and one `u32` evidence ordinal per member;
- exact lookup by card ordinal, raw-record ordinal, role, and member;
- the complete owned M8.4a evidence directory, preserving entity kind, source
  identity, lexical issues, raw groups, values, and original binary64 bits;
- cancellation checks and fallible allocation throughout directory creation.

Cardinality is deliberately independent from numeric validity. A unique invalid
ASCII number is still `Unique`; multiple invalid values are still `Multiple`.

## Test evidence

`crates/seacad-dxf-core/tests/infinite_line_geometry_card_tests.rs` covers:

- ASCII/Binary parity for every supported AC1009-AC1032 dialect;
- the exact six-card role order for both RAY and XLINE;
- absent, unique, and multiple states with exact occurrence counts;
- member order and exact signed-zero/binary64 evidence recovery;
- invalid ASCII values remaining members independently of cardinality;
- missing lookups, cancellation, and public `Copy`/`Send`/`Sync` contracts.

## Non-claims

M8.4b does not choose a canonical or primary value, require a field, validate or
normalize unit-direction vectors, infer ray/xline direction or extent, validate
subclasses or version applicability, transform coordinates, assemble geometry,
edit/write, render, snap, or infer topology.

## Required checkpoint gates

- `cargo deny --locked check`
- `cargo +1.97.1 fmt --all -- --check`
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`
- `cargo +1.97.1 clippy --locked --workspace --all-targets -- -D warnings`
- `cargo +1.97.1 test --locked --workspace`
- `git diff --check`

All checkpoint gates passed on 2026-07-29:

- dependency policy: advisories, bans, licenses, and sources all `ok`;
- formatting and generated-schema drift checks passed;
- workspace Clippy passed with warnings denied;
- workspace tests: 330 passed, 0 failed, 0 ignored, including all 3 focused
  M8.4b integration tests;
- `git diff --check` passed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/infinite_line_geometry_card.rs` | 322 | `5389b74ae1f36b3f434b9d849f4cf8cb82a04bad3dec532bca19898387c1553a` |
| `crates/seacad-dxf-core/src/lib.rs` | 256 | `555e9828ece557cb36bfb9564869c05136baace337b61990a53eca320b425922` |
| `crates/seacad-dxf-core/tests/infinite_line_geometry_card_tests.rs` | 343 | `df8b0293c94bf5d8a0466a351446a466fb0a8e4f4621af85c8228265a8e25fd8` |
| `docs/IMPLEMENTATION_PLAN.md` | 387 | `2328c8299c6d657f6a2b7e2adc03144024d271b3256c9000b99585a48e937fe4` |
| `docs/SUPPORT_MATRIX.md` | 292 | `e423a17f358890dbf21990b24b75b1c697c9f5496317ca96b3a74d73ddc5a6c3` |
