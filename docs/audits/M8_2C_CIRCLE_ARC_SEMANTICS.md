# M8.2c CIRCLE/ARC Value Semantics

Retrieved: 2026-07-29

## Normative boundary

- Autodesk [CIRCLE (DXF)](https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-8663262B-222C-414D-B133-4A8506A27C18.htm)
  identifies center `10/20/30` in OCS, radius `40`, and optional extrusion
  `210/220/230` with default `(0, 0, 1)`.
- Autodesk [ARC (DXF)](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-0B14D8F1-0EBA-44BF-9108-57D8CE614BC8.htm)
  adds start angle `50` and end angle `51` to the same center/radius/extrusion
  contract.
- Autodesk [DXF Formatting Conventions](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-E50DB779-69AE-43C6-B004-85653A983AC0.htm)
  states that optional codes are explicitly marked optional. Center, radius,
  and ARC angles are not marked optional in the entity tables.
- Autodesk [Binary DXF Files](https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-FC1C3C69-DBC2-49E4-893A-000D6538C0FE.htm)
  states that angle translation to degrees applies to Binary as well as ASCII
  DXF files.
- Autodesk [Object Coordinate Systems](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-D99F1509-E4E4-47A3-8691-92EA07DC88F5.htm)
  classifies CIRCLE and ARC as planar entities whose points use OCS.

M8.2c therefore requires one unique valid center component and radius for both
families, plus one unique valid degree-valued start and end angle for ARC. It
applies the documented extrusion default component-wise only when that card is
absent. The reviewed references do not state a radius range or require angle
normalization, so this milestone does neither.

## Implementation contract

`crates/seacad-dxf-core/src/circular_geometry_semantic.rs` provides:

- one lazy semantic projection over the retained M8.2b card directory;
- `Explicit`, `Defaulted`, or typed `Invalid` states with field provenance;
- `MissingRequiredValue` without invented raw provenance;
- `InvalidAsciiNumber` with the unique raw occurrence and exact span;
- `MultipleValues { occurrence_count }` anchored to the first raw occurrence,
  while every occurrence remains available from M8.2b;
- OCS center and extrusion tuple helpers that succeed only when every component
  is explicit or defaulted;
- scalar radius and degree-named ARC angle helpers with exact binary64 bits;
- `None` angle fields for CIRCLE to distinguish inapplicability from an invalid
  ARC angle;
- borrowed record entries and semantic construction only on query, with no
  semantic array materialized for every entity.

## Test evidence

`crates/seacad-dxf-core/tests/circular_geometry_semantic_tests.rs` covers:

- ASCII/Binary parity for all supported AC1009-AC1032 dialects;
- exact signed-zero, negative-radius, and unnormalized `450`-degree bits;
- all-default CIRCLE extrusion and partially explicit ARC extrusion;
- missing, invalid, underflowing, and duplicate required evidence;
- invalid explicit extrusion not being hidden by defaults;
- angle inapplicability for CIRCLE versus typed invalidity for ARC;
- field/raw provenance, lazy record lookup, cancellation, and public traits.

## Non-claims

M8.2c does not validate positive radius, normalize angles, infer ARC sweep or
endpoints, validate subclass/version applicability, normalize extrusion,
implement the arbitrary-axis algorithm, transform OCS to WCS/UCS/DCS, apply
thickness, assemble geometry, edit/write, render, snap, or infer topology.

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
- workspace tests: 316 passed, 0 failed, 0 ignored, including all 3 focused
  M8.2c integration tests;
- `git diff --check` passed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/circular_geometry_semantic.rs` | 434 | `32e1e3bbe15c0ac9e04964741c1164f6d0062580ebf7f39e3b58c471fe47313f` |
| `crates/seacad-dxf-core/src/lib.rs` | 227 | `79cc498605a5ef78dbae9a1344a3c36652460d66ecd196286db05c2a524e7bad` |
| `crates/seacad-dxf-core/tests/circular_geometry_semantic_tests.rs` | 351 | `9950573624d001e60c12baee0b466fe9aa3651f8706a7c1a05b12c250714db42` |
| `docs/IMPLEMENTATION_PLAN.md` | 345 | `b5df5565feb883f6303e4728626a67d79d62515c5ea89c4a517790dd9ae948a9` |
| `docs/SUPPORT_MATRIX.md` | 242 | `6d8e22bf08a40bdd8a538491a2f7a4774e4ec16e69455d6cdc224c7b373479df` |
