# M14.4h HATCH Boundary Path Topology

## Scope

M14.4h groups the opaque M14.4f boundary span into source-stable path envelopes
using exact group-92 anchors and compares their count with group 91. It does not
interpret path flags or payloads.

## Normative evidence

The Autodesk HATCH table defines group 91 as the number of repeated boundary
paths:

`https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-C6C71CED-CE0F-4184-82A5-07AD6241F15B.htm`

The boundary-path table defines group 92 as the path type flag beginning each
boundary path and documents the nested payload that follows:

`https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-DC5215D6-E73F-4DFF-8BE9-01CA9610FAEE.htm`

No external or legacy implementation, fixture, data, or dependency was copied,
translated, vendored, linked, or consulted.

## Contract

- Every exact HATCH subclass receives one topology entry. An unavailable M14.4f
  partition remains typed and publishes no paths or orphan slice.
- Every group 92 in an available boundary span begins one path. Its exact raw
  marker is retained; payload fields continue to the next anchor or group 75.
- Fields before the first group 92 remain an explicit orphan range. Empty path,
  payload, and orphan slices are valid and source stable.
- Group 91 decodes as signed Int32 and reports matched or mismatched observed
  anchors, malformed ASCII, or a negative declaration with exact provenance.
- Duplicate subclasses remain independent. Construction is cancellation-aware,
  fallibly allocated, source-identity checked, and bounded by M14.4a evidence.

## Nonclaims

M14.4h does not decode group-92 bits, select polyline versus edge grammar,
validate payload cardinality or source handles, decode vertices/arcs/ellipses/
splines, derive geometry, establish applicability, add CRUD/write behavior,
qualify a corpus, or claim `Complete` support.

## Verification

The focused boundary-path suite passes 4/4 tests and the workspace passes
1,121/1,121 tests. Dependency policy, formatting, schema, release evidence,
workspace Clippy with warnings denied, safety/link/protected-surface scans, and
whitespace checks pass. No dependency, schema, license, release, corpus, or CI
configuration changes. The checkpoint adds 483 physical production lines: 476
in the boundary-path module and seven module/export lines. This audit
intentionally omits its own self-referential hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 210 | `19aa21184a9998906ea0fd286a6b4a01df3ffd34140d68cce17e02f458bfe6d6` |
| `README.vi.md` | 208 | `2727800588ae493a208c0f14b5c75ff4835eee8dbf3c9aab2c7e5e0e474a1eeb` |
| `crates/seacad-dxf-core/src/hatch_boundary_path.rs` | 476 | `7d5557d42e3b208badd81ef9678c3c959c7fdc3595388b1696c2e72e47034d87` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,314 | `04414c977b49dfaded65df8063c57b03f3930e867f32fbf2c3913440b38bd280` |
| `crates/seacad-dxf-core/tests/hatch_boundary_path_tests.rs` | 411 | `71adb9b4690fc2cd2f0ac81e74158bcfb987410d69dced21e11567b94d148118` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `8d6dac3fa90a386ead0bd9ca9f33114abb0bd3db14d5e069890daddcaed126f9` |
| `docs/SUPPORT_MATRIX.md` | 3,093 | `62e046722a44805de24e615753594773adfe23d7b66ebb240dbb41502fe64ca4` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,011 | `9116d3ac753d66e3169ad54e378aff762227ce2c77c4ea9678320c3e21c429ed` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,432 | `c2d6097cde468c7b58b3f11471112b841707325eee9e3091a4d18614c688483d` |
