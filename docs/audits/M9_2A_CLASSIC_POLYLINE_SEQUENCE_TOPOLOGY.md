# M9.2a Classic POLYLINE Sequence Topology

Retrieved: 2026-07-29

## Normative boundary

- Autodesk [POLYLINE (DXF)](https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-ABF6B778-BE20-4B49-9B58-A94E64CEFFF3.htm)
  documents classic POLYLINE fields and states that group `66`, formerly the
  entities-follow flag, is obsolete, optional, and ignored when present.
- Autodesk [VERTEX (DXF)](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-0741E831-599E-4CBF-91E1-8ADBCFD6556D.htm)
  documents the separate VERTEX entity and its 2D, 3D, mesh, and polyface
  payload families.
- Autodesk [SEQEND (DXF)](https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-FD4FAA74-1F6D-45F6-B132-BF0C4BE6CC3B.htm)
  identifies SEQEND as the marker ending the vertex sequence for a polyline.

M9.2a uses only those exact record markers and their order. It does not infer a
sequence from group `66`, record payloads, case-folding, or nearby orphan
records.

## Implementation contract

`crates/seacad-dxf-core/src/polyline_sequence.rs` provides:

- exact uppercase POLYLINE recognition inside complete `BLOCKS` and
  `ENTITIES` sections only;
- a compact retained slice of every consecutive exact uppercase VERTEX record
  after each recognized POLYLINE;
- `Closed` state with the exact following SEQEND boundary record;
- `Interrupted` state with the first unexpected same-section boundary record;
- `Unclosed` state when the containing section ends before SEQEND;
- restart at an interrupting POLYLINE, so one malformed sequence cannot hide a
  later candidate;
- source identity, raw-record ownership, ordinal lookup, ASCII/Binary adapters,
  and cancellation before, during, and after directory construction.

The sequence directory is linear in retained raw records. It stores compact
raw-record metadata only and never copies a record payload.

## Test evidence

`crates/seacad-dxf-core/tests/polyline_sequence_tests.rs` covers:

- identical ASCII/Binary sequence evidence across all supported
  AC1009-AC1032 dialects;
- POLYLINE sequences in both `BLOCKS` and `ENTITIES`;
- present zero/nonzero obsolete group `66` values having no effect;
- closed sequences with zero, one, and multiple vertices;
- unexpected-record interruption, section-boundary unclosed state, and restart
  at an interrupting POLYLINE;
- orphan VERTEX/SEQEND records, wrong sections, lowercase markers, and trailing
  whitespace remaining unmatched;
- source identity, ordinal lookup bounds, cancellation, and public traits.

The fixtures prove physical record topology only. They do not claim semantic
applicability of every classic polyline form to every `$ACADVER` dialect.

## Non-claims

M9.2a does not decode POLYLINE or VERTEX fields; interpret flags; distinguish
2D, 3D, polygon mesh, or polyface mesh forms; validate group `66`; resolve face
indices; apply defaults; group geometric vertices; infer closure geometry;
transform OCS/WCS; edit; write; render; tessellate; or diagnose conformance.

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
- `cargo +1.97.1 test --locked --workspace`: 363 passed, 0 failed, 0
  ignored, including all three focused M9.2a integration tests;
- `git diff --check`: passed.

No dependency manifest or lockfile changed. A focused production-source scan
also found none of the prohibited `unsafe`, panic, unwrap, expect, todo, or
unimplemented constructs in the new module.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/polyline_sequence.rs` | 311 | `98ce20ccbfd32c8feb8572f6fc5e865ee2a4afc0c3618d75a9c9427a8cfc8f92` |
| `crates/seacad-dxf-core/src/lib.rs` | 320 | `370b12052cae0c8e9c299b40ecd8f3b6d9a93b6c9643208189798f4ccf8f97c9` |
| `crates/seacad-dxf-core/tests/polyline_sequence_tests.rs` | 293 | `308cf5168787f45ef50c8f31f0ce967a4610fd0beced7e66ac7faf16cd171495` |
| `docs/IMPLEMENTATION_PLAN.md` | 496 | `ca3c504e78d9e20fb8d19d64e946dd62631c96b56ce0c852d5f2b7e6ed484425` |
| `docs/SUPPORT_MATRIX.md` | 412 | `abc83162c1ede3b3ae95a79b4adc4e4c30e8f58d6788d5a9cba31f0e53403d30` |
