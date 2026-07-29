# M9.2h Classic POLYLINE VERTEX Integer Semantics

Retrieved: 2026-07-30

## Normative boundary

- Autodesk [VERTEX (DXF)](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-0741E831-599E-4CBF-91E1-8ADBCFD6556D.htm)
  documents flags `70`, optional polyface indices `71`-`74`, and vertex
  identifier `91`.
- The table assigns meanings to flag bits `1`, `2`, `8`, `16`, `32`, `64`, and
  `128`; bit `4` is explicitly not used.
- It says a negative polyface index makes the edge starting at that vertex
  invisible and the first zero terminates a face. M9.2h preserves signed values
  but deliberately defers that interpretation until vertex family and face
  topology are available.
- Autodesk publishes no default for these six integer fields. Their absence
  therefore remains semantic `Absent`, not an invented zero or required-value
  failure.

## Implementation contract

`crates/seacad-dxf-core/src/polyline_vertex_integer_semantic.rs` provides:

- one lazy semantic directory owning the complete M9.2d card/evidence graph;
- optional source-anchored signed-i16 flags and four polyface indices plus the
  signed-i32 identifier;
- exact explicit values, typed invalid-number and multiple-value states, and
  raw provenance where source evidence exists;
- independent helpers for the seven meaningful flag bits, returning
  unavailable for absent, invalid, or duplicate flags;
- no semantic helper for unused bit `4`;
- raw-record, ASCII-document, and Binary-document adapters plus lookup by raw
  VERTEX ordinal, exact entry, or parent-POLYLINE/sequence vertex ordinal;
- bounded cancellation before semantic directory construction.

## Test evidence

`crates/seacad-dxf-core/tests/polyline_vertex_integer_semantic_tests.rs` covers:

- ASCII/Binary parity across every supported AC1009-AC1032 dialect;
- exact signed-i16 minimum, negative/zero/positive indices, signed-i32 maximum,
  and all seven meaningful flag helpers;
- absent fields without defaults, invalid integers, duplicate cardinality, and
  retained raw provenance;
- semantic availability for closed, interrupted, and unclosed sequences;
- parent/sequence lookup bounds, cancellation, source identity, and public
  traits.

The parity fixtures prove physical decoding only. They do not claim that every
field is semantically applicable to every `$ACADVER` dialect.

## Non-claims

M9.2h does not require flags, indices, or identifiers; assign meaning to bit
`4`; interpret negative-index edge visibility or zero termination; validate
index ranges; classify 2D, 3D, polygon-mesh, mesh-coordinate, or polyface-face
vertices; reconcile parent and vertex flags; resolve faces; transform OCS/WCS;
assemble geometry; edit; write; render; or diagnose conformance.

## Required checkpoint gates

- `cargo deny --locked check`
- `cargo +1.97.1 fmt --all -- --check`
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`
- `cargo +1.97.1 clippy --locked --workspace --all-targets -- -D warnings`
- `cargo +1.97.1 test --locked --workspace`
- `git diff --check`

All checkpoint gates passed on 2026-07-30:

- dependency policy: advisories, bans, licenses, and sources all `ok`;
- formatting and generated-schema drift checks passed;
- workspace Clippy passed with warnings denied;
- workspace tests: 389 passed, 0 failed, 0 ignored, including all three focused
  M9.2h integration tests;
- `git diff --check` passed.

No dependency manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/polyline_vertex_integer_semantic.rs` | 383 | `26493570b66026254ce6114423e54dd336f5a9f4f1feb5ad03a086cf48ed2461` |
| `crates/seacad-dxf-core/src/lib.rs` | 352 | `bd1227eddaaebeb1835869b5238b98bc9adfd84100d1ea37a540692ba70b24d0` |
| `crates/seacad-dxf-core/tests/polyline_vertex_integer_semantic_tests.rs` | 322 | `0ea727bd8886c5f6aee79b0e4f9c7a8366c899e10f488f18008b8eca2c57bd89` |
| `docs/IMPLEMENTATION_PLAN.md` | 582 | `f8f0f5a699a7204c771b67ee819f506696c00a9356e8bd5d3e769be35169bff0` |
| `docs/SUPPORT_MATRIX.md` | 489 | `132692924d187b701673b5993c80b23a682e2ecfbf05bc89a23dbd41d4279752` |
