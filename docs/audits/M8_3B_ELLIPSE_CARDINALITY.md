# M8.3b ELLIPSE Value Cardinality

Retrieved: 2026-07-29

## Evidence boundary

M8.3b layers deterministic per-role cards over the exact ELLIPSE values
reviewed and retained by M8.3a. It introduces no new DXF group-code,
coordinate-system, ratio, or parameter interpretation. Normative facts and
primary Autodesk sources remain recorded in
`docs/audits/M8_3A_ELLIPSE_EVIDENCE.md`.

Every recognized ELLIPSE receives twelve cards even when its M8.3a value slice
is empty: WCS center X/Y/Z, relative major-axis endpoint X/Y/Z,
minor-to-major axis ratio, start parameter, end parameter, and extrusion X/Y/Z.

## Implementation contract

`crates/seacad-dxf-core/src/ellipse_geometry_card.rs` owns:

- one stable source-order card sequence grouped by raw record and documented
  role order;
- `Absent`, `Unique`, or `Multiple { occurrence_count }` state based only on
  the number of matching M8.3a occurrences;
- one compact member range per card and one `u32` evidence ordinal per member;
- exact lookup by card ordinal, raw-record ordinal, role, and member;
- the complete owned M8.3a evidence directory, preserving source identity,
  lexical issues, raw groups, values, and original binary64 bits;
- cancellation checks and fallible allocation throughout directory creation.

Cardinality is deliberately independent from numeric validity. A unique invalid
ASCII number is still `Unique`; multiple invalid values are still `Multiple`.

## Test evidence

`crates/seacad-dxf-core/tests/ellipse_geometry_card_tests.rs` covers:

- ASCII/Binary parity for every supported AC1009-AC1032 dialect;
- the exact twelve-card ELLIPSE role order;
- absent, unique, and multiple states with exact occurrence counts;
- member order and exact signed-zero/binary64 evidence recovery;
- invalid ASCII values remaining members independently of cardinality;
- missing lookups, cancellation, and public `Copy`/`Send`/`Sync` contracts.

## Non-claims

M8.3b does not choose a canonical or primary value, apply extrusion or thickness
defaults, require a field, validate ratio or parameter ranges, infer ellipse
endpoints or sweep, normalize values, validate subclasses or version
applicability, transform coordinates, assemble geometry, edit/write, render,
snap, or infer topology.

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
- workspace tests: 320 passed, 0 failed, 0 ignored, including all 3 focused
  M8.3b integration tests;
- `git diff --check` passed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/ellipse_geometry_card.rs` | 324 | `40b584eacacb65a0138ba98c354304ad36a520e2d70fa769114cbbfd5621cece` |
| `crates/seacad-dxf-core/src/lib.rs` | 238 | `629664cf9e7387cb63705ba5d9b02bfaffb7d53c3cffa6e3414fa3badde3b0b4` |
| `crates/seacad-dxf-core/tests/ellipse_geometry_card_tests.rs` | 328 | `edbda3f2c59dd4b4ea953bb747327b1939c85f51d88b752cfc995344ebacaf2a` |
| `docs/IMPLEMENTATION_PLAN.md` | 362 | `5da895e41039677daa34f8796635c5704c49f00d9fe696e617576c550f3f5946` |
| `docs/SUPPORT_MATRIX.md` | 263 | `3c6076af3ff1d59594ce86c2285f343af67ba3a9b7242cd8dfb8c1945c0becc0` |
