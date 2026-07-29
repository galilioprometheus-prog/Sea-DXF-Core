# M9.2f Classic POLYLINE Record Cardinality

Retrieved: 2026-07-30

## Normative boundary

- Autodesk [POLYLINE (DXF)](https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-ABF6B778-BE20-4B49-9B58-A94E64CEFFF3.htm)
  documents the sixteen numeric record roles retained by M9.2b.
- Optionality, defaults, bit meanings, and field applicability do not permit
  SeaCad to discard duplicate, malformed, or inapplicable-looking source
  occurrences before semantic policy is applied.

M9.2f therefore adds neutral occurrence-count evidence without selecting a
canonical value, applying defaults, or classifying the polyline family.

## Implementation contract

`crates/seacad-dxf-core/src/polyline_record_card.rs` provides:

- sixteen fixed cards in stable documented-role order for every M9.2b POLYLINE;
- explicit `Absent`, `Unique`, or `Multiple { occurrence_count }` state;
- compact source-order members referring back to M9.2b value ordinals without
  copying numeric values;
- independent cardinality and lexical validity, so invalid ASCII occurrences
  still count exactly once;
- sixteen absent cards for a recognized POLYLINE with no documented numeric
  groups;
- strict record locality so following VERTEX values cannot enter a POLYLINE
  card;
- retained source, sequence-state, record, role, card, member, and raw-value
  provenance, with ASCII/Binary adapters and bounded cancellation.

## Test evidence

`crates/seacad-dxf-core/tests/polyline_record_card_tests.rs` covers:

- identical ASCII/Binary cards and exact member values across every supported
  AC1009-AC1032 dialect;
- all sixteen roles, stable role order, `Absent`, `Unique`, and `Multiple`;
- invalid unique int16 occurrences and multiple invalid doubles retaining
  cardinality independently from lexical validity;
- strict POLYLINE/VERTEX record locality;
- sixteen absent cards for an empty POLYLINE;
- cards retained for closed, interrupted, and unclosed sequences across
  `BLOCKS` and `ENTITIES`;
- source identity, card/member/record lookup bounds, cancellation, and public
  traits.

The fixtures prove physical cardinality and provenance only. They do not claim
that every field applies to every polyline family or `$ACADVER` dialect.

## Non-claims

M9.2f does not select or decode canonical values; apply defaults; interpret
group `66`, flags, counts, densities, or smooth-surface type; validate dummy
coordinates, widths, extrusion, or count domains; classify 2D, 3D, polygon-
mesh, or polyface polylines; decode vertex integers; resolve faces; transform
coordinates; assemble geometry; edit; write; render; or diagnose conformance.

## Required checkpoint gates

- `cargo deny --locked check`
- `cargo +1.97.1 fmt --all -- --check`
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`
- `cargo +1.97.1 clippy --locked --workspace --all-targets -- -D warnings`
- `cargo +1.97.1 test --locked --workspace`
- `git diff --check`

All checkpoint gates passed on 2026-07-30:

- `cargo deny --locked check`: advisories, bans, licenses, and sources passed;
- `cargo +1.97.1 fmt --all -- --check`: passed;
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`: generated
  schema matched the committed artifacts;
- `cargo +1.97.1 clippy --locked --workspace --all-targets -- -D warnings`:
  passed with warnings denied;
- `cargo +1.97.1 test --locked --workspace`: 383 passed, 0 failed, 0
  ignored, including all four focused M9.2f integration tests;
- `git diff --check`: passed.

No dependency manifest or lockfile changed. A focused production-source scan
also found none of the prohibited `unsafe`, panic, unwrap, expect, todo, or
unimplemented constructs in the new module.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/polyline_record_card.rs` | 286 | `e37d3f7aefbec2548c29088f01f68e43fe591ab838bba5b5098b8744c59e0b36` |
| `crates/seacad-dxf-core/src/lib.rs` | 342 | `21d03312ebc8920800c19c1202416ed0786001a80ef3fc97030df2ce93ace3da` |
| `crates/seacad-dxf-core/tests/polyline_record_card_tests.rs` | 357 | `7f07e610ed1a02a408bb39c6fcfb4f19c9c42f5f7e963251c07fc82d78a01710` |
| `docs/IMPLEMENTATION_PLAN.md` | 554 | `fe6332021440a7ccb4bbbd08407ed5884ceb67c1698fe7b5b1c8b69786239c19` |
| `docs/SUPPORT_MATRIX.md` | 466 | `f1f8d9a772bdbe4621ac381228ac3565ba439afd69d30ce4ed1b2529460b6bb8` |
