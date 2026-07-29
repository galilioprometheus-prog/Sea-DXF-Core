# M9.2b Classic POLYLINE Record Values

Retrieved: 2026-07-29

## Normative boundary

- Autodesk [POLYLINE (DXF)](https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-ABF6B778-BE20-4B49-9B58-A94E64CEFFF3.htm)
  documents dummy/elevation groups `10/20/30`, thickness `39`, default widths
  `40/41`, obsolete entities-follow `66`, flags `70`, mesh counts/densities and
  surface type `71`-`75`, and extrusion `210/220/230`.
- Autodesk [Group Code Value Types Reference](https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-2553CF98-44F6-4828-82DD-FE3BC7448113.htm)
  defines `10`-`59` and `210`-`239` as double-precision families and `60`-`79`
  as signed 16-bit integers.

M9.2b retains those sixteen roles exactly as numeric wire evidence. Their
published meanings and defaults are recorded for provenance but are not yet
applied as semantic policy.

## Implementation contract

`crates/seacad-dxf-core/src/polyline_record_value.rs` provides:

- one entry for every M9.2a classic POLYLINE sequence, including interrupted
  and unclosed sequences;
- nine exact binary64 roles and seven exact signed-16-bit roles without numeric
  domain merging;
- source-order occurrences, duplicates, raw groups, raw spans, exact Binary
  values, and typed ASCII syntax/range failures;
- an empty slice for recognized POLYLINE records without documented numeric
  groups;
- strict record locality so same-numbered VERTEX groups never enter a POLYLINE
  value slice;
- retained M9.2a topology, source identity, raw-ordinal and group-occurrence
  lookup, ASCII/Binary adapters, and bounded cancellation checks.

Group `66` is retained only as signed-16-bit evidence. Sequence discovery
continues to ignore it and depends exclusively on exact record topology.

## Test evidence

`crates/seacad-dxf-core/tests/polyline_record_value_tests.rs` covers:

- all sixteen roles with exact ASCII/Binary parity across every supported
  AC1009-AC1032 dialect;
- signed-zero and ordinary binary64 bit preservation plus signed-int16 values;
- duplicate doubles, double syntax/range failures, int16 syntax/range failures,
  and the signed-int16 minimum;
- exclusion of same-numbered groups from following VERTEX records;
- empty POLYLINE value slices;
- retained values for closed, interrupted, and unclosed sequences, including a
  later valid POLYLINE in another section after an unclosed sequence;
- source identity, lookup bounds, cancellation, and public traits.

The fixtures prove physical decoding and record locality only. They do not
claim that every field is semantically applicable to every `$ACADVER` dialect.

## Non-claims

M9.2b does not select canonical occurrences; apply defaults; require dummy X/Y
zero; use group `66`; interpret flags, counts, densities, or smooth-surface
type; validate widths, extrusion, or other domains; classify 2D, 3D, polygon
mesh, or polyface mesh records; decode VERTEX payloads; resolve faces; transform
OCS/WCS; assemble geometry; edit; write; render; or diagnose conformance.

## Required checkpoint gates

- `cargo deny --locked check`
- `cargo +1.97.1 fmt --all -- --check`
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`
- `cargo +1.97.1 clippy --locked --workspace --all-targets -- -D warnings`
- `cargo +1.97.1 test --locked --workspace`
- `git diff --check`

All checkpoint gates passed on 2026-07-29:

- `cargo deny --locked check`: advisories, bans, licenses, and sources passed;
- `cargo +1.97.1 fmt --all -- --check`: passed;
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`: generated
  schema matched the committed artifacts;
- `cargo +1.97.1 clippy --locked --workspace --all-targets -- -D warnings`:
  passed with warnings denied;
- `cargo +1.97.1 test --locked --workspace`: 367 passed, 0 failed, 0
  ignored, including all four focused M9.2b integration tests;
- `git diff --check`: passed.

No dependency manifest or lockfile changed. A focused production-source scan
also found none of the prohibited `unsafe`, panic, unwrap, expect, todo, or
unimplemented constructs in the new module.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/polyline_record_value.rs` | 371 | `2e4f0df824677e26694fff0971efa3ebe21557f443d2e72a2cddadb4025ec8ff` |
| `crates/seacad-dxf-core/src/lib.rs` | 326 | `9e75b26b77440e15dbbea353e2d30cafdee7fee77556f9e8cc0ddc92680a6018` |
| `crates/seacad-dxf-core/tests/polyline_record_value_tests.rs` | 343 | `8b8d16decc3c7768c252da7c3bb03bf899630ee6a80a01b72460eb87a075732b` |
| `docs/IMPLEMENTATION_PLAN.md` | 509 | `28d6aefa22ff6b0b84ebd19d3f52997d36046c22034d8f4e8771d293d3bfdaff` |
| `docs/SUPPORT_MATRIX.md` | 425 | `38f8664497d353c90dc83f8da4e69a5d42da85c3f66bf4c910d13e72c31f9ad0` |
