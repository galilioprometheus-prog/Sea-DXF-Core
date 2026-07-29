# M8.4a RAY/XLINE Defining-Value Evidence

Retrieved: 2026-07-29

## Normative boundary

- Autodesk [RAY (DXF)](https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-638B9F01-5D86-408E-A2DE-FA5D6ADBD415.htm)
  identifies start point `10/20/30` in WCS and unit direction vector
  `11/21/31` in WCS.
- Autodesk [XLINE (DXF)](https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-55080553-34B6-40AA-9EE2-3F3A3A2A5C0A.htm)
  identifies first point `10/20/30` in WCS and unit direction vector
  `11/21/31` in WCS.
- Autodesk [DXF Group Codes](https://help.autodesk.com/cloudhelp/2018/ENU/OARX-RefGuide/files/OREF-DXF_Group_Codes.html)
  classifies `10..17` and their `20`/`30` companions as double-precision point,
  vector, or scale components.
- Autodesk [DXF ENTITIES Section](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-7D07C886-FD1D-4A0C-A7AB-B4D21F18E484.htm)
  lists RAY and XLINE as graphical entity types.

The entity tables call the direction values unit vectors, but M8.4a is an exact
evidence milestone. It does not normalize or validate vector length and does
not infer version applicability.

## Implementation contract

`crates/seacad-dxf-core/src/infinite_line_geometry.rs` provides:

- a separate immutable directory for exact uppercase `RAY` and `XLINE` records
  in complete `BLOCKS` and `ENTITIES` sections;
- typed RAY/XLINE kind plus source-order WCS point and unit-direction
  occurrences with raw group provenance;
- exact ASCII/Binary double evidence, including signed zero, or typed ASCII
  lexical failure without replacement;
- duplicate preservation and explicit empty value slices;
- lookup by raw-record ordinal or group occurrence without copying payloads;
- bounded allocation, cancellation checks, and shared ASCII/Binary adapters.

## Test evidence

`crates/seacad-dxf-core/tests/infinite_line_geometry_tests.rs` covers:

- ASCII/Binary parity for all supported AC1009-AC1032 dialects;
- exact RAY/XLINE kind, role order, signed-zero, and binary64 bits;
- duplicates, invalid and underflowing ASCII values, and empty slices;
- exact case-sensitive markers and complete-section filtering;
- exclusion of unrelated scalar and extrusion groups;
- record/group lookup, cancellation, and public trait bounds.

## Non-claims

M8.4a does not select canonical values, materialize cardinality cards, validate
required components or unit-vector length, normalize direction, infer ray or
xline extent, validate subclass/version applicability, transform coordinates,
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
- workspace tests: 327 passed, 0 failed, 0 ignored, including all 4 focused
  M8.4a integration tests;
- `git diff --check` passed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/infinite_line_geometry.rs` | 334 | `80779a40066953ae9077601c9e87205be15c16fdaf550279826da23875512856` |
| `crates/seacad-dxf-core/src/lib.rs` | 250 | `d5d49dd6b1fd806737563b7b662a0d13a0e51fe7ab8c44cf528bce8c0b4ee2f3` |
| `crates/seacad-dxf-core/tests/infinite_line_geometry_tests.rs` | 331 | `096988e99201c3417735ea99c06b383acb193ead176835bf48fb4f777befdcd1` |
| `docs/IMPLEMENTATION_PLAN.md` | 380 | `bd01aa0d53feb65b0e39b409b3369986abbec1a495d3377905f4ad9412cc1ba1` |
| `docs/SUPPORT_MATRIX.md` | 285 | `b11a1bbc994536bbc45c06481f2d6172aee65bdd4a5aff58ad75787d578df474` |
