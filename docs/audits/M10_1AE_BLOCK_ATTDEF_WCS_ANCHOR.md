# M10.1ae Classic ATTDEF WCS Anchor

Retrieved: 2026-07-30

## Normative boundary

Autodesk [ATTDEF
(DXF)](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-F0EA099B-6F88-4BCC-BEC7-247BA64838A4.htm)
defines classic text-start/alignment points in OCS and the optional extrusion
direction. Autodesk's [arbitrary-axis
algorithm](https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-E19E5B42-0CC7-4EBA-B29F-5E1D595149EE.htm)
defines the normalized OCS basis and exact `1/64` polar-cap branch.

M10.1ae projects the already-selected M10.1ad placement anchor. It reuses the
same internal OCS basis implementation established by M10.1l rather than
duplicating that algorithm.

## Implementation contract

`crates/seacad-dxf-core/src/block_attribute_definition_wcs_anchor.rs` lazily
maps each usable selected OCS anchor through its usable ATTDEF extrusion.
Successful entries retain:

- a finite WCS point;
- the normalized finite WCS normal;
- canonical positive zero for deterministic bit-level evidence; and
- the complete underlying M10.1ad placement evidence.

Placement or extrusion unavailability, non-finite Binary placement/extrusion
values, zero-length extrusion, non-finite basis derivation, and transformed
overflow remain distinct typed failures.

The implementation reuses the existing crate-scope OCS basis `transform` and
`normal` methods. This changes no public transform API and keeps one reviewed
arbitrary-axis implementation.

Directory construction preserves source identity and reuses the cancellable,
bounded M10.1ad directory. Raw-record, exact-entry, and BLOCK-local ATTDEF
lookups do not allocate payload-sized buffers.

## Test evidence

`crates/seacad-dxf-core/tests/block_attribute_definition_wcs_anchor_tests.rs`
covers ASCII/Binary parity across all nine supported dialects; identity,
non-polar, and polar-cap bases; non-unit extrusion normalization; exact WCS
points and normals; positive-zero canonicalization; unavailable
placement/extrusion; zero extrusion; Binary NaN/infinity; derived overflow;
raw-record, exact-entry, and BLOCK-local lookups; cancellation; source identity;
bounds; and public traits.

## Non-claims

M10.1ae does not apply text rotation, oblique/width/generation flags, text-style
metrics, INSERT/BLOCK transforms, or ATTRIB association. It does not
recalculate stored points, decode MText extensions, edit, write, or render.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, generated-schema
drift, formatting, workspace Clippy with warnings denied, 553 workspace tests
with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/block_attribute_definition_wcs_anchor.rs` | 219 | `73085725aae6e334d0a01a0b0e049788af279a689dcd742d0065e3f452e63b02` |
| `crates/seacad-dxf-core/src/lib.rs` | 596 | `d7c65e96b403cd543f531cc41a05a5d2d982a7e4d337d5e8f22ca37448d5c1a3` |
| `crates/seacad-dxf-core/tests/block_attribute_definition_wcs_anchor_tests.rs` | 322 | `dc4e35b6a2061208e127febab6e0676bd7523576778b1ca1f1d7af736bad0e4c` |
| `docs/IMPLEMENTATION_PLAN.md` | 1104 | `5e903b665b1a9d1cd0ebf44bb104b65ecf02bd5b9e039aa7e79ecf9ba563a2e7` |
| `docs/SUPPORT_MATRIX.md` | 915 | `27c690015a2124942f70b56cc3958fe47c34c5a9b7389a11d49a78365b2a40b0` |
