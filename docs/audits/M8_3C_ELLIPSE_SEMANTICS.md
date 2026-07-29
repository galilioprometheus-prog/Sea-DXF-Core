# M8.3c ELLIPSE Value Semantics

Retrieved: 2026-07-29

## Normative boundary

- Autodesk [ELLIPSE (DXF)](https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-107CB04F-AD4D-4D2F-8EC9-AC90888063AB.htm)
  identifies center `10/20/30` in WCS, major-axis endpoint `11/21/31` relative
  to center in WCS, ratio `40`, start/end parameters `41/42`, and optional
  extrusion `210/220/230` with default `(0, 0, 1)`.
- Autodesk [DXF Formatting Conventions](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-E50DB779-69AE-43C6-B004-85653A983AC0.htm)
  states that optional codes are explicitly marked optional. The reviewed
  ELLIPSE table marks extrusion optional and does not mark the defining center,
  axis, ratio, or parameter values optional.
- Autodesk [Object Coordinate Systems](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-D99F1509-E4E4-47A3-8691-92EA07DC88F5.htm)
  distinguishes WCS and OCS storage. M8.3c preserves the ELLIPSE WCS boundary
  and performs no coordinate transformation.

M8.3c therefore requires one unique valid value for each defining center,
relative major-axis endpoint, ratio, and parameter card. It applies the
documented extrusion default component-wise only when that card is absent. The
reviewed references do not justify silent repair or normalization, so this
milestone preserves out-of-range numeric values exactly.

## Implementation contract

`crates/seacad-dxf-core/src/ellipse_geometry_semantic.rs` provides:

- one lazy semantic projection over the retained M8.3b card directory;
- `Explicit`, `Defaulted`, or typed `Invalid` states with field provenance;
- `MissingRequiredValue` without invented raw provenance;
- `InvalidAsciiNumber` with the unique raw occurrence and exact span;
- `MultipleValues { occurrence_count }` anchored to the first raw occurrence,
  while every occurrence remains available from M8.3b;
- WCS center, relative major-axis endpoint, and extrusion tuple helpers that
  succeed only when every component is explicit or defaulted;
- scalar ratio and parameter helpers that preserve exact binary64 bits;
- borrowed record entries and semantic construction only on query.

## Test evidence

`crates/seacad-dxf-core/tests/ellipse_geometry_semantic_tests.rs` covers:

- ASCII/Binary parity for all supported AC1009-AC1032 dialects;
- exact signed-zero, negative-ratio, and unnormalized parameter bits;
- partially explicit extrusion with the component-wise `0/0/1` defaults;
- missing, invalid, underflowing, and duplicate required evidence;
- invalid explicit extrusion not being hidden by defaults;
- field/raw provenance, lazy record lookup, cancellation, and public traits.

## Non-claims

M8.3c does not validate ratio or parameter ranges, normalize parameters, infer
ellipse endpoints or sweep, validate subclass/version applicability, normalize
extrusion, transform WCS/OCS/UCS/DCS coordinates, apply thickness, assemble
geometry, edit/write, render, snap, or infer topology.

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
- workspace tests: 323 passed, 0 failed, 0 ignored, including all 3 focused
  M8.3c integration tests;
- `git diff --check` passed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/ellipse_geometry_semantic.rs` | 419 | `404389edeff1dff90c6ce027eac24613189f5a9fa0f6c1f4b36b5a71eff50edb` |
| `crates/seacad-dxf-core/src/lib.rs` | 243 | `9b21882e838086a36bb9876ffac19dfbecfb70c53c6d57d96f973d12f0c6232f` |
| `crates/seacad-dxf-core/tests/ellipse_geometry_semantic_tests.rs` | 363 | `4486b9777ce446777b0eab11cbe5204fa9d9f7a21a9de17043903b0686ac2799` |
| `docs/IMPLEMENTATION_PLAN.md` | 371 | `2755487675e534d6ae072b85e9b5407ec16a6e61665c070c98175c6760ec08b6` |
| `docs/SUPPORT_MATRIX.md` | 274 | `e1ac642be05891ea9907f617e13cc768c4e59dba97a23da562ec1e0d471b3195` |
