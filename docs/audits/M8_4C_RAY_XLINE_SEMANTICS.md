# M8.4c RAY/XLINE Value Semantics

Retrieved: 2026-07-29

## Normative boundary

- Autodesk [RAY (DXF)](https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-638B9F01-5D86-408E-A2DE-FA5D6ADBD415.htm)
  identifies start point `10/20/30` in WCS and unit direction vector
  `11/21/31` in WCS.
- Autodesk [XLINE (DXF)](https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-55080553-34B6-40AA-9EE2-3F3A3A2A5C0A.htm)
  identifies first point `10/20/30` in WCS and unit direction vector
  `11/21/31` in WCS.
- Autodesk [DXF Formatting Conventions](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-E50DB779-69AE-43C6-B004-85653A983AC0.htm)
  states that optional codes are explicitly marked optional. The reviewed RAY
  and XLINE defining-value rows are not marked optional.

M8.4c therefore requires one unique valid value for each point and direction
component. Although Autodesk names the second triple a unit direction vector,
this milestone does not silently reject, repair, or normalize its numeric
values; vector conformance remains a distinct future contract.

## Implementation contract

`crates/seacad-dxf-core/src/infinite_line_geometry_semantic.rs` provides:

- one lazy semantic projection over the retained M8.4b card directory;
- distinct RAY and XLINE field namespaces with retained entity kind;
- `Explicit` or typed `Invalid` states with field provenance;
- `MissingRequiredValue` without invented raw provenance;
- `InvalidAsciiNumber` with the unique raw occurrence and exact span;
- `MultipleValues { occurrence_count }` anchored to the first raw occurrence,
  while every occurrence remains available from M8.4b;
- WCS point and unit-direction tuple helpers that succeed only when every
  component is explicit;
- borrowed record entries and semantic construction only on query.

## Test evidence

`crates/seacad-dxf-core/tests/infinite_line_geometry_semantic_tests.rs` covers:

- ASCII/Binary parity for all supported AC1009-AC1032 dialects;
- exact signed-zero and deliberately non-unit direction binary64 bits;
- missing, invalid, underflowing, and duplicate required evidence;
- separate RAY/XLINE field namespaces and raw/field provenance;
- lazy record lookup, cancellation, and public traits.

## Non-claims

M8.4c does not validate or normalize unit-direction vectors, reject zero
vectors, infer ray/xline direction or extent, validate subclass/version
applicability, transform coordinates, assemble geometry, edit/write, render,
snap, or infer topology.

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
- workspace tests: 333 passed, 0 failed, 0 ignored, including all 3 focused
  M8.4c integration tests;
- `git diff --check` passed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/infinite_line_geometry_semantic.rs` | 330 | `4866865d2c8e52fd45ce8444e561d6d50f8b56244fae9b12c5d5361682077dc1` |
| `crates/seacad-dxf-core/src/lib.rs` | 261 | `f8898496724623376f7b0c0d68fa8184a86cbb7a6f021a355474a1235f89fbe0` |
| `crates/seacad-dxf-core/tests/infinite_line_geometry_semantic_tests.rs` | 335 | `8c523d65346b70033b988ec89bab37dff85ba8c47a684918d7ec03bf5a269307` |
| `docs/IMPLEMENTATION_PLAN.md` | 394 | `ac8cc62cf7bfe83b1e72b09342e9efbf1d19fd6fb4a359b57929c53a7c0c9f25` |
| `docs/SUPPORT_MATRIX.md` | 302 | `1208d375ff57f74a43498861514f2a0584a137df00a0b0443f9ef6e508be9f1a` |
