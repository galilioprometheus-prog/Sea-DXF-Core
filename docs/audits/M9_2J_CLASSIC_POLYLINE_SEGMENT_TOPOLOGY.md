# M9.2j Classic POLYLINE Segment Topology

Retrieved: 2026-07-30

## Normative boundary

- Autodesk [POLYLINE (DXF)](https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-ABF6B778-BE20-4B49-9B58-A94E64CEFFF3.htm)
  defines parent flag bit `1` as closed for ordinary polylines and bit `8` as
  the 3D family.
- Autodesk [VERTEX (DXF)](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-0741E831-599E-4CBF-91E1-8ADBCFD6556D.htm)
  defines ordered VERTEX records and their 2D/3D family flags.
- M9.2j requires the structural `SEQEND` proof from M9.2a and the fail-closed
  family agreement from M9.2i before emitting topology.

## Implementation contract

`crates/seacad-dxf-core/src/polyline_segment.rs` provides:

- consecutive segments for complete, family-consistent classic 2D/3D paths;
- a last-to-first segment only when usable parent flag bit `1` is set;
- exact parent, start/end VERTEX entries and document/record-local ordinals;
- typed zero-segment states for incomplete sequences, mesh/polyface parents,
  indeterminate parent families, and inconsistent VERTEX evidence;
- empty-path and one-vertex closed-path behavior, lookup, ASCII/Binary adapters,
  source identity, bounded allocation, and cancellation.

## Test evidence

`crates/seacad-dxf-core/tests/polyline_segment_tests.rs` covers all dialects
and both physical formats, open 2D and closed 3D topology, closing ordinals,
each zero-segment state, empty and one-vertex paths, lookup bounds,
cancellation, source identity, and public traits.

## Non-claims

M9.2j does not project endpoint coordinates or effective widths, interpret
bulge geometry, apply elevation/extrusion transforms, build mesh topology,
resolve polyface faces, edit, write, render, or diagnose conformance.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, formatting,
generated-schema drift, workspace Clippy with warnings denied, 395 workspace
tests with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/polyline_segment.rs` | 355 | `8d20ab4fa0137bf07c2873a4c1d0480a553d35622ec92fb9b86672ac61573dd6` |
| `crates/seacad-dxf-core/src/lib.rs` | 363 | `1de82fc4c980c5d954d35bb9275922c68b7a91c4e4d8c4fa9aba6106eabe9892` |
| `crates/seacad-dxf-core/tests/polyline_segment_tests.rs` | 250 | `f36aac4afc7ab2cae18c68bea3f5a8838ae2ca06bbcf7705fe78cdc81fb1610a` |
| `docs/IMPLEMENTATION_PLAN.md` | 603 | `d79a24e38aef065c3666e91f7d7447ad6dbf042800357bf5b02934a4eebd273a` |
| `docs/SUPPORT_MATRIX.md` | 507 | `2086fd4111ea24ce71568250481f033572c2a82fa1eb31f6ded5505e70b73404` |
