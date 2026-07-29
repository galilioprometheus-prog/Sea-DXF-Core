# M8.1c POINT/LINE Component Semantics

Retrieved: 2026-07-29

## Normative boundary

- Autodesk [POINT (DXF)](https://help.autodesk.com/cloudhelp/2025/ENU/AutoCAD-DXF/files/GUID-9C6AD32D-769D-4213-85A4-CA9CCB5C5317.htm)
  identifies groups `10/20/30` as the WCS point location and documents optional
  extrusion direction `210/220/230` with default `(0, 0, 1)`.
- Autodesk [LINE (DXF)](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-FCEF5726-53AE-4C43-B4EA-C84EB8686A66.htm)
  identifies `10/20/30` as the WCS start, `11/21/31` as the WCS endpoint, and
  the same optional extrusion default.
- Autodesk [Common Group Codes for Entities (DXF)](https://help.autodesk.com/cloudhelp/2025/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm)
  warns that consumers must not depend on table order and explains that listed
  omitted defaults are applied on input.

The location, start, and endpoint rows are not marked optional. M8.1c therefore
requires one unique valid component for each reviewed WCS role. It applies the
documented extrusion default component-wise as X=`0`, Y=`0`, Z=`1` only when
the corresponding card is absent.

## Implementation contract

- `crates/seacad-dxf-core/src/basic_geometry_semantic.rs` owns compact
  source-order POINT/LINE semantic entries and lazy typed projections over the
  retained M8.1b card directory.
- Required-card absence becomes `MissingRequiredComponent` with no invented raw
  provenance.
- A unique valid card becomes `Explicit` and retains the exact raw group span.
- A unique invalid ASCII card remains `InvalidAsciiNumber` with raw provenance.
- A multiple card remains `MultipleComponents { occurrence_count }` and keeps
  the first raw occurrence as a navigation anchor while every member remains
  available through M8.1b.
- Only absent extrusion components become `Defaulted`; present invalid or
  duplicate evidence never falls back to a default.
- POINT location, LINE start, LINE endpoint, and extrusion expose tuple helpers
  only when every component has a usable explicit or defaulted value.
- The directory stores only compact entries and card evidence. Large semantic
  component arrays are created lazily for the queried record rather than
  materialized for every entity.

## Test evidence

`crates/seacad-dxf-core/tests/basic_geometry_semantic_tests.rs` covers:

- ASCII/Binary parity for every supported AC1009-AC1032 dialect;
- exact signed-zero and binary64 component bits;
- all-default POINT extrusion and partially explicit LINE extrusion;
- missing, invalid, underflowing, and duplicate required evidence;
- invalid explicit extrusion not being hidden by defaults;
- source/field/raw provenance, compact lookup, cancellation, and trait bounds.

## Non-claims

M8.1c does not validate subclass markers or version applicability, normalize an
extrusion vector, convert WCS/OCS/UCS/DCS coordinates, apply thickness or POINT
display angle, select duplicate values, edit/write, render, snap, infer
topology, or implement CIRCLE/ARC and later geometry families.

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
- workspace tests: 306 passed, 0 failed, 0 ignored, including all 3 focused
  M8.1c integration tests;
- `git diff --check` passed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/basic_geometry_semantic.rs` | 496 | `d54254cf01515e5c7362606297e9e346c3f9c048b3c455ab171bdef4ac4a2b3a` |
| `crates/seacad-dxf-core/src/lib.rs` | 210 | `73c44551bce988811040423f3d3fb20b5af6dcd2659fa33b56ab1ce63833e471` |
| `crates/seacad-dxf-core/tests/basic_geometry_semantic_tests.rs` | 342 | `633aec567eeb4104225bcc058783e9170d04e42e81ce6a03eac401d4220033e3` |
| `docs/IMPLEMENTATION_PLAN.md` | 325 | `d29950cd12dfc12955a5e48a1578c6ea594d023997ce24a24ff0f1c7ade4ffdf` |
| `docs/SUPPORT_MATRIX.md` | 211 | `7bdba1ef44a743cac75a8504faf9d2f5683c7971e6ce6defdb18003855cf72c0` |
