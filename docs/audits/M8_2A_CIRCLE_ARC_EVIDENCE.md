# M8.2a CIRCLE/ARC Evidence Receipt

Retrieved: 2026-07-29

## Primary references

- Autodesk, [CIRCLE (DXF)](https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-8663262B-222C-414D-B133-4A8506A27C18.htm)
- Autodesk, [ARC (DXF)](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-0B14D8F1-0EBA-44BF-9108-57D8CE614BC8.htm)
- Autodesk, [Object Coordinate Systems (OCS) in DXF](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-D99F1509-E4E4-47A3-8691-92EA07DC88F5.htm)
- Autodesk ObjectARX 2027, [DXF Group Codes](https://help.autodesk.com/cloudhelp/2027/ENU/OARX-RefGuide/files/OARX-RefGuide-DXF_Group_Codes.html)

## Reviewed facts

- CIRCLE and ARC use groups `10`, `20`, and `30` for the center point in OCS.
- Both use group `40` for radius and optional `210`, `220`, and `230` for the
  extrusion direction; the documented omitted extrusion default is `(0, 0, 1)`.
- ARC additionally uses group `50` for the start angle and `51` for the end
  angle.
- Autodesk classifies CIRCLE and ARC as planar entities whose points are
  expressed in OCS, unlike the WCS points exposed for POINT and LINE.
- ObjectARX classifies `10..17` plus their `20..27` and `30..37` companions as
  point/vector doubles, `40..47` as general doubles, and `50..59` as angles.

## Implementation contract

`crates/seacad-dxf-core/src/circular_geometry.rs` provides a separate immutable,
format-neutral evidence directory for exact uppercase `CIRCLE` and `ARC`
records in complete `BLOCKS` or `ENTITIES` sections. It:

- indexes center, radius, ARC angle, and extrusion roles only by numeric group
  code without depending on group order;
- decodes through the shared bounded ASCII/Binary double path;
- retains source order, duplicates, invalid ASCII numbers, signed zero, exact
  binary64 bits, group occurrence, raw spans, source identity, and empty record
  slices;
- exposes record and group-occurrence lookups over compact immutable metadata;
- checks cancellation before and throughout source-dependent work.

The dedicated directory prevents CIRCLE/ARC OCS evidence from weakening the
existing POINT/LINE WCS contract.

## Test evidence

`crates/seacad-dxf-core/tests/circular_geometry_tests.rs` covers:

- ASCII/Binary parity for every supported AC1009-AC1032 dialect;
- source-order center, radius, angle, and extrusion roles with exact bits;
- signed zero, invalid syntax, underflow, duplicates, empty value slices, and
  exact group lookup;
- exact uppercase matching in complete `BLOCKS` or `ENTITIES` sections only;
- CIRCLE angle codes, thickness, unrelated sections, and case variants staying
  outside the claim;
- cancellation, bounds, and public `Copy`/`Send`/`Sync` contracts.

## Non-claims

M8.2a does not select canonical values, create per-role cardinality cards,
apply extrusion or thickness defaults, require fields, validate a positive or
finite geometry radius, attach angle units or normalize angle ranges, determine
ARC sweep, validate subclass/version applicability, transform OCS to WCS,
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
- workspace tests: 310 passed, 0 failed, 0 ignored, including all 4 focused
  M8.2a integration tests;
- `git diff --check` passed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/circular_geometry.rs` | 343 | `dbe302bcaf677924a0d72bde7b79ec95881d30b8d9f12a96f192c37ad2ebca1a` |
| `crates/seacad-dxf-core/src/lib.rs` | 216 | `ae9ac6a86ec85c8e3c9499d528ed4d4981bd1b76e3db53c60f8f845838da8b60` |
| `crates/seacad-dxf-core/tests/circular_geometry_tests.rs` | 316 | `ae1c8c622ad6fcdd1019348efe3ac53ee18884e365a8264e310dc8515c7cfeb9` |
| `docs/IMPLEMENTATION_PLAN.md` | 332 | `b82fb8df6dc8b90e3e35fd41a4f42d8a06dfc098d771cfc395e249f5583ddd0f` |
| `docs/SUPPORT_MATRIX.md` | 222 | `757a751964bfa375397ca271e2a65dec2fbdccb143d4a0d6c408b09c419eb9ff` |
