# M9.2p Classic POLYLINE Polyface Partition Topology

Retrieved: 2026-07-30

## Normative boundary

Autodesk [Polyface Meshes (DXF)](https://help.autodesk.com/cloudhelp/2016/ENU/AutoCAD-DXF/files/GUID-96B6288E-F413-46C0-968A-A314171C0AAE.htm)
defines parent flag bit `64`, uses groups `71/72` for vertex and face counts,
warns that applications need not write correct counts, identifies coordinate
and face VERTEX records by their flag combinations, and requires readers to
tolerate odd coordinate/face ordering. Autodesk
[POLYLINE (DXF)](https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-ABF6B778-BE20-4B49-9B58-A94E64CEFFF3.htm)
independently documents parent flag bit `64` and groups `71/72`.

## Implementation contract

`crates/seacad-dxf-core/src/polyline_polyface_topology.rs` partitions every
complete family-consistent polyface sequence into exact coordinate and
face-definition VERTEX ranges. Reported coordinate/face counts and independent
observed counts remain visible even when they disagree or are unusable.
Coordinates appearing after a face are retained with typed `Odd` ordering.
Incomplete, unsupported, indeterminate, and inconsistent records retain typed
zero-member state.

## Test evidence

`crates/seacad-dxf-core/tests/polyline_polyface_topology_tests.rs` covers
ASCII/Binary parity across all nine dialects, deliberately inaccurate reported
counts, odd and conventional ordering, unusable count evidence, exact partition
membership and record-local ordinals, every typed failure, lookup bounds,
cancellation, source identity, and public traits.

## Non-claims

M9.2p does not interpret face groups `71`-`74`, resolve signed coordinate
indices or edge visibility, validate index ranges, assemble face geometry,
project coordinates, edit, write, or render.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, formatting,
generated-schema drift, workspace Clippy with warnings denied, 414 workspace
tests with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/polyline_polyface_topology.rs` | 499 | `305f95b0b602bfb2c106fdfc0798a254d3f50f2e8ac8335472486dc79cd386a3` |
| `crates/seacad-dxf-core/src/lib.rs` | 402 | `daf926e39b02d17b016236741a78528ed97098c3b0e16689da9c0f8ae3b18862` |
| `crates/seacad-dxf-core/tests/polyline_polyface_topology_tests.rs` | 280 | `b8d0e7f71597a72b2c93ad647cf793530969a95145c4619f50af76020b6f978a` |
| `docs/IMPLEMENTATION_PLAN.md` | 673 | `af55c28f70eafb5a8455b8d92cd5679f4265edd97dde0c846a0c65b519a70017` |
| `docs/SUPPORT_MATRIX.md` | 568 | `31ae1930dfc9dc46ebd89977e206f71e9fa66a5a8e8d0dbb583583099fdbb25d` |
