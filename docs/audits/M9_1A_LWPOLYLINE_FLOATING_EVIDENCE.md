# M9.1a LWPOLYLINE Floating-Point Evidence

Retrieved: 2026-07-29

## Normative boundary

- Autodesk [LWPOLYLINE (DXF)](https://help.autodesk.com/cloudhelp/2016/ENU/AutoCAD-DXF/files/GUID-748FC305-F3F2-4F74-825A-61F04D757A50.htm)
  identifies elevation `38`, thickness `39`, constant width `43`, repeated OCS
  vertex coordinates `10/20`, repeated start/end widths `40/41`, repeated
  bulge `42`, and optional extrusion `210/220/230` as floating-point values.
- The same table identifies integer vertex count `90`, flags `70`, and vertex
  identifier `91`. M9.1a leaves those groups exact in the existing raw record;
  typed integer evidence is a separate checkpoint.

The entity table describes per-vertex multiplicity, but raw group ordering and
malformed evidence require an explicit grouping contract. M9.1a therefore
retains each documented floating occurrence in source order without pairing or
selecting it.

## Implementation contract

`crates/seacad-dxf-core/src/lightweight_polyline.rs` provides:

- exact uppercase `LWPOLYLINE` recognition in complete `BLOCKS` and `ENTITIES`;
- one stable record entry per recognized raw record;
- eleven distinct documented floating-point roles;
- source-order occurrence slices with exact group and value spans;
- exact ASCII lexical failure or Binary IEEE-754 bits, including signed zero;
- duplicate and empty slices without canonical selection;
- binary-search lookup by raw-record ordinal and source group occurrence;
- shared raw-document, ASCII-document, and Binary-document adapters;
- explicit cancellation and fallible allocation behavior.

## Test evidence

`crates/seacad-dxf-core/tests/lightweight_polyline_tests.rs` covers:

- ASCII/Binary parity for all supported AC1009-AC1032 physical dialects;
- every one of the eleven floating-point roles and repeated vertex values;
- exact signed-zero preservation;
- coexistence with correctly framed integer groups `90`, `70`, and `91`;
- duplicate, invalid, underflowing, and empty floating evidence;
- exact case-sensitive matching and section scoping;
- record/group lookup, cancellation, and public traits.

The parity fixtures are physical decoding evidence only. M9.1a does not claim
that `LWPOLYLINE` is semantically applicable to every `$ACADVER` value.

## Non-claims

M9.1a does not decode integer count, flags, or vertex identifiers; group values
into vertices; apply defaults; compare declared and observed vertex counts;
validate ordering, cardinality, widths, bulges, extrusion, or version
applicability; transform OCS; assemble segments; edit/write; render; snap; or
infer topology.

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
- workspace tests: 337 passed, 0 failed, 0 ignored, including all 4 focused
  M9.1a integration tests;
- `git diff --check` passed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/lightweight_polyline.rs` | 321 | `bc0a9a41100aed980cd5ca13e51e756ff416eaad9e3496f1c57a27235cd85238` |
| `crates/seacad-dxf-core/src/lib.rs` | 267 | `12fe93fd5573bb3ec45f5564ed0382bc9f79795da68d5c3fdc0cc888deaa83db` |
| `crates/seacad-dxf-core/tests/lightweight_polyline_tests.rs` | 305 | `072d7c9b81aa2da4802524875df02ac21b5684de52a8e4444193f1b6126e6db1` |
| `docs/IMPLEMENTATION_PLAN.md` | 405 | `4261aa7f961659eb320f04adcb8d7c017e74179e96f9e98f1388a1a5f0d47089` |
| `docs/SUPPORT_MATRIX.md` | 315 | `9a47d3e1f2961e0fed467e07afa7177ea7affc61e9cca68bd861aa2d12038768` |
