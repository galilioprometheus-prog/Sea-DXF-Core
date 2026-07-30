# M10.1w BLOCK ATTDEF Topology

Retrieved: 2026-07-30

## Normative boundary

Autodesk [ATTDEF
(DXF)](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-F0EA099B-6F88-4BCC-BEC7-247BA64838A4.htm)
identifies ATTDEF as the attribute-definition entity and publishes its defining
group codes, including prompt, tag, flags, default text, placement, and
extrusion. Autodesk's [DXF ENTITIES
index](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-7D07C886-FD1D-4A0C-A7AB-B4D21F18E484.htm)
lists ATTDEF as a distinct entity type.

M10.1w establishes only the record topology needed before those fields can be
decoded and before an INSERT attribute can be associated with a block
attribute definition.

## Implementation contract

`crates/seacad-dxf-core/src/block_attribute_definition.rs` indexes exact
uppercase `ATTDEF` group-zero records from the retained member slices of the
existing BLOCK-definition directory. Each entry retains:

- the exact raw ATTDEF record;
- its owning BLOCK-definition entry and boundary state;
- its zero-based position among all members of that definition; and
- its zero-based position among ATTDEF members of that definition.

Records outside an indexed BLOCK definition, including orphan ATTDEF records
in BLOCKS or ENTITIES, do not enter the directory. Lowercase, padded, or
otherwise non-exact markers do not match. Interrupted and unclosed definitions
remain representable because this layer does not silently require a closed
parent.

The directory preserves source identity, checks cancellation during both
definition and member traversal, and offers allocation-free exact-record and
per-BLOCK ordinal lookups after construction. It inherits the complete-BLOCKS-
section publication boundary from M10.1a.

## Test evidence

`crates/seacad-dxf-core/tests/block_attribute_definition_tests.rs` covers
ASCII/Binary parity across all nine supported dialects; multiple ATTDEF members
and their two local ordinals; exact marker case; records outside definitions;
closed, interrupted, and unclosed owners; incomplete BLOCKS-section
suppression; cancellation; source identity; bounded misses; and public traits.

## Non-claims

M10.1w does not decode ATTDEF fields, apply defaults, interpret attribute or
text flags, validate tags, compare ATTDEF and ATTRIB tags, associate inserted
attribute sequences, apply INSERT/BLOCK transforms, decode MText extensions,
edit, write, or render.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, generated-schema
drift, formatting, workspace Clippy with warnings denied, 522 workspace tests
with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/block_attribute_definition.rs` | 201 | `681fac8b09a2df4aee9ec34b60e33b5c484e3aeb716a4ba1ebd309bb85afe375` |
| `crates/seacad-dxf-core/src/lib.rs` | 544 | `72c4bb118282c0488134f699b6c320409abe16081a4df7966361988b737096a4` |
| `crates/seacad-dxf-core/tests/block_attribute_definition_tests.rs` | 238 | `159a00e4e347984ba88d5fdd399f3d36ddcbf06df15b9e9f6cf245f08b71ac01` |
| `docs/IMPLEMENTATION_PLAN.md` | 994 | `7f418c7c13ef3715d0319b2540ce220f030e3a22d8644f0ee896ee4dddab37bd` |
| `docs/SUPPORT_MATRIX.md` | 828 | `13b2e87cbb52693ec07cf746ba98de256988ed30560621a17ec7b829bf598b27` |
