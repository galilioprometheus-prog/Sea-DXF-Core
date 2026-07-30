# M9.2q Classic POLYLINE Polyface Face Resolution

Retrieved: 2026-07-30

## Normative boundary

Autodesk [VERTEX (DXF)](https://help.autodesk.com/cloudhelp/2020/ENU/AutoCAD-DXF/files/GUID-0741E831-599E-4CBF-91E1-8ADBCFD6556D.htm)
defines optional face indices in groups `71`-`74`, numbers coordinate VERTEX
records from one in their polyline order, makes a negative index hide the edge
beginning at that corner, and makes the first zero terminate the face.
Autodesk [Polyface Meshes (DXF)](https://help.autodesk.com/cloudhelp/2016/ENU/AutoCAD-DXF/files/GUID-96B6288E-F413-46C0-968A-A314171C0AAE.htm)
limits one face record to four vertices and requires readers to tolerate odd
coordinate/face ordering.

## Implementation contract

`crates/seacad-dxf-core/src/polyline_polyface_face.rs` resolves each usable
signed 1-based face index against the exact record-local M9.2p coordinate
partition, including coordinates retained after an oddly ordered face. Each
corner retains its source index, exact coordinate entry, and edge visibility.
Absent slots and explicit zero terminate the face. Invalid values, nonzero
post-terminator values, `i16::MIN`, and out-of-range indices retain typed
zero-corner face states.

## Test evidence

`crates/seacad-dxf-core/tests/polyline_polyface_face_tests.rs` covers
ASCII/Binary parity across all nine dialects, resolution across odd ordering,
positive and negative indices, explicit termination, exact coordinate identity,
edge visibility, every typed failure, an empty face, lookup bounds,
cancellation, source identity, and public traits.

## Non-claims

M9.2q does not assemble coordinate tuples or face geometry, validate geometric
degeneracy, winding, planarity, manifoldness, or normals, triangulate, edit,
write, or render.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, formatting,
generated-schema drift, workspace Clippy with warnings denied, 417 workspace
tests with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/polyline_polyface_face.rs` | 401 | `f8507e4f223f2db9c1604ea3b324a1e4c4e6b54a0c52f3143c9784fd11ba4553` |
| `crates/seacad-dxf-core/src/lib.rs` | 408 | `22be82febbd0548b2546b60fc3bb6d4bd4025636373abdb2d438b8efd9c981da` |
| `crates/seacad-dxf-core/tests/polyline_polyface_face_tests.rs` | 248 | `449ba13ac0aa1e0682683dae6220aee690368f96fa771e04141141c113f23c92` |
| `docs/IMPLEMENTATION_PLAN.md` | 685 | `454c42be20e26d0c1920cd2ef06a77a3008336702d1ee48a1593d36ef2c4d334` |
| `docs/SUPPORT_MATRIX.md` | 577 | `54d638f06d09bcc39ac13fd70913085913499009c0880ddc2611fee999018391` |
