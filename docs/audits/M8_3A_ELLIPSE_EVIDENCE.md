# M8.3a ELLIPSE Defining-Value Evidence

Retrieved: 2026-07-29

## Normative boundary

- Autodesk [ELLIPSE (DXF)](https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-107CB04F-AD4D-4D2F-8EC9-AC90888063AB.htm)
  identifies center `10/20/30` in WCS, major-axis endpoint `11/21/31` relative
  to center in WCS, optional extrusion `210/220/230` with default `(0, 0, 1)`,
  minor-to-major axis ratio `40`, start parameter `41`, and end parameter `42`.
- Autodesk [DXF Group Codes](https://help.autodesk.com/cloudhelp/2027/ENU/OARX-RefGuide/files/OARX-RefGuide-DXF_Group_Codes.html)
  classifies `10..17` and their `20`/`30` companions as point/vector data and
  `40..47` as double-precision values. The ELLIPSE-specific table defines
  `41/42` as parameters, not degree-valued angle fields.
- Autodesk [Object Coordinate Systems](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-D99F1509-E4E4-47A3-8691-92EA07DC88F5.htm)
  distinguishes WCS and OCS storage. M8.3a retains the documented ELLIPSE WCS
  boundary and does not reuse the CIRCLE/ARC OCS directory.

The reviewed entity table supplies field identities and the extrusion default,
but this evidence-only milestone does not apply defaults, decide requiredness,
or validate ratio and parameter ranges.

## Implementation contract

`crates/seacad-dxf-core/src/ellipse_geometry.rs` provides:

- a separate immutable directory for exact uppercase `ELLIPSE` records in
  complete `BLOCKS` and `ENTITIES` sections;
- source-order center, relative major-axis endpoint, ratio, parameter, and
  extrusion-component occurrences with raw group provenance;
- exact ASCII/Binary double evidence, including signed zero, or typed ASCII
  lexical failure without replacement;
- duplicate preservation and explicit empty value slices;
- lookup by raw-record ordinal or group occurrence without copying payloads;
- bounded allocation, cancellation checks, and shared ASCII/Binary adapters.

## Test evidence

`crates/seacad-dxf-core/tests/ellipse_geometry_tests.rs` covers:

- ASCII/Binary parity for all supported AC1009-AC1032 dialects;
- exact signed-zero and end-parameter binary64 bits;
- duplicates, invalid and underflowing ASCII values, and empty slices;
- exact case-sensitive markers and complete-section filtering;
- exclusion of unrelated angle `50` and thickness `39` groups;
- record/group lookup, cancellation, and public trait bounds.

## Non-claims

M8.3a does not select canonical values, materialize cardinality cards, apply the
documented extrusion default, validate required fields, ratio constraints, or
parameter ranges, infer endpoints or sweep, validate subclass/version
applicability, normalize extrusion, transform WCS/OCS/UCS/DCS coordinates,
apply thickness, assemble geometry, edit/write, render, snap, or infer topology.

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
- workspace tests: 317 passed, 0 failed, 0 ignored, including all 4 focused
  M8.3a integration tests;
- `git diff --check` passed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/ellipse_geometry.rs` | 321 | `0cd6a04b36bb8a13dc922fa71c7f98b41b4c81322722d25fb92aeb0a86f734b5` |
| `crates/seacad-dxf-core/src/lib.rs` | 232 | `f5d7b354218eb1a094d966cc994928bbbe4d56172f1e9a334a9f20a9f4792bb1` |
| `crates/seacad-dxf-core/tests/ellipse_geometry_tests.rs` | 296 | `110b117562fe1e7d5ea6b53f9f9140f387241261fdff14698f0de272999baee8` |
| `docs/IMPLEMENTATION_PLAN.md` | 355 | `a1c85f0e5c10895de693a5ca8d7b5a2a472c29fe8b8b89ce1f2601ae673ec72a` |
| `docs/SUPPORT_MATRIX.md` | 254 | `1667929f1bb139302702d12641aea5c1f596f60a4931f2c11965d178c763af41` |
