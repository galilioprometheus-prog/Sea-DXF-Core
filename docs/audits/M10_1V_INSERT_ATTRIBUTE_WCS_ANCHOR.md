# M10.1v Classic ATTRIB WCS Anchor

Retrieved: 2026-07-30

## Normative boundary

Autodesk [ATTRIB
(DXF)](https://help.autodesk.com/cloudhelp/2020/ENU/AutoCAD-DXF/files/GUID-7DD8B495-C3F8-48CD-A766-14F9D7D0DD9B.htm)
defines classic text-start/alignment points in OCS and the optional extrusion
direction. Autodesk's [arbitrary-axis
algorithm](https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-E19E5B42-0CC7-4EBA-B29F-5E1D595149EE.htm)
defines the normalized OCS basis and exact `1/64` polar-cap branch.

M10.1v projects the already-selected M10.1u placement anchor. It reuses the
same internal OCS basis implementation established by M10.1l rather than
duplicating that algorithm.

## Implementation contract

`crates/seacad-dxf-core/src/insert_attribute_wcs_anchor.rs` lazily maps each
usable selected OCS anchor through its usable ATTRIB extrusion. Successful
entries retain:

- a finite WCS point;
- the normalized finite WCS normal;
- canonical positive zero for deterministic bit-level evidence; and
- the complete underlying M10.1u placement evidence.

Placement or extrusion unavailability, non-finite Binary placement/extrusion
values, zero-length extrusion, non-finite basis derivation, and transformed
overflow remain distinct typed failures.

`crates/seacad-dxf-core/src/insert_transform.rs` exposes only its existing OCS
basis `transform` and `normal` methods at crate scope. This changes no public
API and keeps one reviewed arbitrary-axis implementation.

Directory construction preserves source identity and reuses the cancellable,
bounded M10.1u directory. Raw-record, exact-entry, and INSERT-sequence-local
lookups do not allocate payload-sized buffers.

## Test evidence

`crates/seacad-dxf-core/tests/insert_attribute_wcs_anchor_tests.rs` covers
ASCII/Binary parity across all nine supported dialects; identity, non-polar,
and polar-cap bases; non-unit extrusion normalization; exact WCS points and
normals; positive-zero canonicalization; unavailable placement/extrusion;
zero extrusion; Binary NaN/infinity; derived overflow; raw-record, exact-entry,
and sequence-local lookups; cancellation; source identity; bounds; and public
traits.

## Non-claims

M10.1v does not apply text rotation, oblique/width/generation flags, text-style
metrics, INSERT/BLOCK transforms, or ATTDEF association. It does not
recalculate stored points, decode MText extensions, edit, write, or render.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, formatting,
generated-schema drift, workspace Clippy with warnings denied, 518 workspace
tests with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/insert_attribute_wcs_anchor.rs` | 213 | `c3fead3405c47ea451cb553a180432923fbce1d20d69c3823b4ea5c8ae06cce8` |
| `crates/seacad-dxf-core/src/insert_transform.rs` | 423 | `b35f117c42e290450661353aa105e8d7ade419cdd984830a84b1b32b77e7acd2` |
| `crates/seacad-dxf-core/src/lib.rs` | 540 | `217f7248efbd0b340d88e23247f680d6d67eabdb47c6b6312968c119ca9e128d` |
| `crates/seacad-dxf-core/tests/insert_attribute_wcs_anchor_tests.rs` | 321 | `1d5add1f9c4839fd80b4a41e0bd3761c99dd8dd323393bb9b7554ea47e26c525` |
| `docs/IMPLEMENTATION_PLAN.md` | 982 | `470af5f5014cb5ce8884f604b1219b19a94a8fb55d5a05af675b2b2ed6525065` |
| `docs/SUPPORT_MATRIX.md` | 819 | `c49c551854dab8e5dd3e150c20cce001589001777dd3ed7ea81f6bb7ba775155` |
