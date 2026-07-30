# M14.1c Planar-Face Value Semantics

Retrieved: 2026-07-31

## Normative boundary

Autodesk's [3DFACE
reference](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-747865D5-51F0-45F2-BEFE-9572DBC5B151.htm)
defines WCS corners, a fourth-corner fallback to the third corner, and optional
invisible-edge flags defaulting to zero. Autodesk's [SOLID
reference](https://help.autodesk.com/cloudhelp/2018/ENU/AutoCAD-DXF/files/GUID-E0C5F04E-D0C5-48F5-AC09-32733E8848F2.htm)
defines a fourth-corner fallback, zero thickness, and (0,0,1) extrusion
defaults. Autodesk's [TRACE
reference](https://help.autodesk.com/cloudhelp/2018/ENU/AutoCAD-DXF/files/GUID-EA6FBCA8-1AD6-4FB2-B149-770313E93511.htm)
defines OCS corners plus the same thickness and extrusion defaults, but does
not publish a fourth-corner fallback.

M14.1c applies only these documented defaults. It treats undocumented missing
corner components as unavailable rather than importing assumptions from other
entity families.

## Legacy evidence boundary

`D:\SeaCad\cad_2026-07-23_source` remains a read-only behavioral oracle. Its
normalized-view tests identify missing coordinates, edge flags, corner order,
and zero extrusion as later compatibility risks. No legacy parser
implementation was copied, translated, vendored, or linked. Autodesk
documentation remains normative.

## Implementation contract

`planar_face_geometry_semantic.rs` exposes one lazy semantic directory and one
compact record value containing:

- four typed double corner tuples;
- optional signed-16-bit invisible-edge flags for 3DFACE;
- optional double thickness and extrusion values for SOLID/TRACE; and
- exact field provenance plus raw provenance for every explicit or invalid
  source occurrence.

Unique values become explicit. Missing required values, invalid ASCII numbers,
duplicates, and an unusable third-corner default source remain distinct typed
failures. Defaults have field provenance but invent no raw bytes. An entirely
absent 3DFACE/SOLID fourth tuple may inherit the third tuple; a partial tuple
does not. TRACE requires its fourth tuple.

`planar_face_geometry_semantic_value.rs` isolates scalar selection and
provenance mechanics so both new production files remain below 400 lines.
Construction owns the M14.1b card directory, checks source identity, observes
cancellation, and supports bounded raw-record lookup.

## Test evidence

`planar_face_geometry_semantic_tests.rs` covers ASCII/Binary parity across all
nine AC1009--AC1032 dialects; explicit and defaulted corners; all documented
metadata defaults; partial explicit extrusion; exact binary64 bits; missing,
invalid, multiple, partial, and unavailable-default states; TRACE's required
fourth tuple; raw/field provenance behavior; absent lookup; cancellation; and
public `Copy`/`Send`/`Sync` bounds.

## Non-claims

M14.1c does not default undocumented missing corner components, interpret edge
bits, reject unknown edge bits, validate finite coordinates or nonzero normals,
reorder SOLID/TRACE corners, transform OCS to WCS, apply thickness, assemble
faces, expand blocks, edit, write, render, or tessellate.

## Required checkpoint gates

All required gates passed on 2026-07-31: dependency policy, generated-schema
drift, release-evidence drift, formatting, workspace Clippy with warnings
denied, 612 workspace tests with zero failures/ignored tests, and
`git diff --check`. No dependency manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 78 | `adc24736b8decef5cf7ba287dcd0eac98174714a2cf8ac0488d8f6aabad6fdd7` |
| `crates/seacad-dxf-core/src/lib.rs` | 644 | `be08a84d06369c1a06527004d3a1ec2be4f00285e08d5bd219d22a757eb25e58` |
| `crates/seacad-dxf-core/src/planar_face_geometry_semantic.rs` | 372 | `a407031894721db96e48f62533b91ba909159245877c7b1df3b609791937f292` |
| `crates/seacad-dxf-core/src/planar_face_geometry_semantic_value.rs` | 170 | `be59f64e085c1d577c5f2e517778f35e0ce30bf444e91379f2ee3f6433d0d746` |
| `crates/seacad-dxf-core/tests/planar_face_geometry_semantic_tests.rs` | 349 | `529a7b379f3b226e1cb2cf5c07e98d7640717d0dedde4ade688255a8d94eb1da` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 97 | `4186a13c2c11452ee74439a268da6f3d3d3e646def20b2d8d5a399becd9a1273` |
| `docs/IMPLEMENTATION_PLAN.md` | 1371 | `8ce96d8f6fc41ec51bc617cf791beb67aad367bf71739353e3d12756dc352134` |
| `docs/SUPPORT_MATRIX.md` | 1094 | `e80f098781f8cb97cd76c0e31eb619aa447c9bbc0f73e055f02af17de5032c3b` |
