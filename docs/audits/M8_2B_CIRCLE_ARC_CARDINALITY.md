# M8.2b CIRCLE/ARC Value Cardinality

Retrieved: 2026-07-29

## Evidence boundary

M8.2b layers deterministic per-role cards over the exact CIRCLE/ARC values
reviewed and retained by M8.2a. It introduces no new DXF group-code or
coordinate-system interpretation. Normative facts and primary Autodesk sources
remain recorded in `docs/audits/M8_2A_CIRCLE_ARC_EVIDENCE.md`.

Every recognized record receives its full applicable role set even when its
M8.2a value slice is empty:

- CIRCLE: OCS center X/Y/Z, radius, extrusion X/Y/Z — seven cards;
- ARC: OCS center X/Y/Z, radius, start angle, end angle, extrusion X/Y/Z — nine
  cards.

## Implementation contract

`crates/seacad-dxf-core/src/circular_geometry_card.rs` owns:

- one stable source-order card sequence grouped by raw record and documented
  role order;
- `Absent`, `Unique`, or `Multiple { occurrence_count }` state based only on
  the number of matching M8.2a occurrences;
- one compact member range per card and one `u32` evidence ordinal per member;
- exact lookup by card ordinal, raw-record ordinal, role, and member;
- the complete owned M8.2a evidence directory, preserving source identity,
  lexical issues, raw groups, values, and original binary64 bits;
- cancellation checks and fallible allocation throughout directory creation.

Cardinality is deliberately independent from numeric validity. A unique invalid
ASCII number is still `Unique`; multiple invalid values are still `Multiple`.

## Test evidence

`crates/seacad-dxf-core/tests/circular_geometry_card_tests.rs` covers:

- ASCII/Binary parity for every supported AC1009-AC1032 dialect;
- the exact seven-card CIRCLE and nine-card ARC role order;
- absent, unique, and multiple states with exact occurrence counts;
- member order and exact signed-zero/binary64 evidence recovery;
- invalid ASCII values remaining members independently of cardinality;
- inapplicable CIRCLE angle roles producing no card;
- missing lookups, cancellation, and public `Copy`/`Send`/`Sync` contracts.

## Non-claims

M8.2b does not choose a canonical or primary value, apply extrusion or thickness
defaults, require a field, validate radius or angles, infer an ARC sweep,
normalize values, validate subclasses or version applicability, transform OCS,
assemble geometry, edit/write, render, snap, or infer topology.

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
- workspace tests: 313 passed, 0 failed, 0 ignored, including all 3 focused
  M8.2b integration tests;
- `git diff --check` passed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/circular_geometry_card.rs` | 339 | `78021252580f0051dbf0f2d24d05d59547a6d1e33f512f08f29862cb6d7e988b` |
| `crates/seacad-dxf-core/src/lib.rs` | 222 | `fe29d16280819887aee9d524c19e544e6c445765d8805085ae4e56ad4f118a94` |
| `crates/seacad-dxf-core/tests/circular_geometry_card_tests.rs` | 373 | `4cfdfedc1224e5ade9d3421db12df3cfeab7d62bc6bb60029d49d557289743de` |
| `docs/IMPLEMENTATION_PLAN.md` | 338 | `30ee103a98fb4b4e9e2d5d74fcaae193cba675fe3528e008ff2e4894c470ac68` |
| `docs/SUPPORT_MATRIX.md` | 231 | `f476971a28e07ee87563476e93f3520eff5d01a6b9633d38680d470bd3365ed3` |
