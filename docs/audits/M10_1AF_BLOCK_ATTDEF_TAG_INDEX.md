# M10.1af Block-Local ATTDEF Tag Index

Retrieved: 2026-07-30

## Normative boundary

Autodesk [ATTDEF
(DXF)](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-F0EA099B-6F88-4BCC-BEC7-247BA64838A4.htm)
defines group `2` as the ATTDEF tag string. Autodesk's [Attribute Definition
dialog](https://help.autodesk.com/cloudhelp/2022/ENU/AutoCAD-Core/files/GUID-92A7CE9D-2E1C-415E-A1BF-5CB35D5E2D38.htm)
says the tag identifies each attribute occurrence, forbids spaces, and
uppercases lowercase input. The command documentation additionally warns that
duplicate tag names cause extraction and dynamic-block problems.

Those authoring rules do not authorize rewriting arbitrary DXF source.
M10.1af therefore indexes exact stored bytes, retains duplicates, and reports
unusable tag semantics rather than normalizing or discarding them.

## Implementation contract

`crates/seacad-dxf-core/src/block_attribute_definition_tag_index.rs` builds
one immutable exact-tag index over M10.1aa:

- only usable required ATTDEF tag values enter lookup;
- every BLOCK receives total, indexed, and unusable ATTDEF-tag counts,
  including blocks with zero ATTDEF records;
- lookup is scoped by exact owning BLOCK and preserves duplicate definitions
  in definition-local order;
- SHA-256 narrows candidates while exact raw-span comparison confirms equality,
  so digest collisions cannot create a match; and
- caller bytes or a same-document source span can query the index through
  fixed 4 KiB buffers without tag-sized allocation.

The directory owns the M10.1aa semantic/evidence chain, preserves source
identity, checks cancellation during construction and lookup, and exposes
bounded misses.

## Test evidence

`crates/seacad-dxf-core/tests/block_attribute_definition_tag_index_tests.rs`
covers ASCII/Binary parity across all nine supported dialects; block-local
separation; duplicate, unique, and missing lookup; definition-local ordering;
unusable tag counts; blocks with zero ATTDEF records; case-exact raw behavior;
same-document ATTRIB tag-span lookup without a payload buffer; cancellation;
source identity; bounded misses; and public traits.

## Non-claims

M10.1af does not validate spaces or other tag syntax, uppercase or decode tags,
collapse duplicates, require closed BLOCK topology, claim a missing exact
match is definitive when unusable tags exist, resolve INSERT targets, associate
ATTRIB records, edit, write, or render.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, generated-schema
drift, formatting, workspace Clippy with warnings denied, 557 workspace tests
with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/block_attribute_definition_tag_index.rs` | 581 | `f18cf6480faf02f2962b6b42c3bd4a6485ebf4a59be1e11f5022edf0c425ea44` |
| `crates/seacad-dxf-core/src/lib.rs` | 601 | `975c43e92bc368614c2387e7635737a179a161f942af7923f6d749f87b112aa3` |
| `crates/seacad-dxf-core/tests/block_attribute_definition_tag_index_tests.rs` | 369 | `f015a01c7bd9b0526567b9186890cef41d6140492a2b3b832d5d85162af1c5d6` |
| `docs/IMPLEMENTATION_PLAN.md` | 1116 | `b346420b937dd07b8df21b710b56d470a51633018306272da25b4e1f2bf2332c` |
| `docs/SUPPORT_MATRIX.md` | 925 | `77d934740230114ddd8ad973671d390875d1771895a317da297e5474a94a2a64` |
