# M10.1r Classic ATTRIB Text Semantics

Retrieved: 2026-07-30

## Normative boundary

Autodesk [ATTRIB
(DXF)](https://help.autodesk.com/cloudhelp/2020/ENU/AutoCAD-DXF/files/GUID-7DD8B495-C3F8-48CD-A766-14F9D7D0DD9B.htm)
documents text/default value `1`, attribute tag `2`, and the optional text-style
name `7` whose omitted default is `STANDARD`. M10.1r projects only these three
classic roles from the M10.1p cards.

## Implementation contract

`crates/seacad-dxf-core/src/insert_attribute_text_semantic.rs` lazily projects
one typed text-semantic view for every retained classic ATTRIB:

- text/default value and attribute tag are required source-anchored values;
- an explicit style name remains source-anchored; and
- an absent style name becomes typed default `STANDARD` without invented raw
  provenance.

Duplicate roles and missing required roles remain typed; duplicate state keeps
the first occurrence's raw provenance. Explicit text retains the exact M10.1o
source span and encoding resolution. Decoding is caller-buffered, bounded,
same-document, and replacement-free through the existing text-value API.

Directory construction preserves source identity, observes cancellation, and
reuses the bounded M10.1p card directory. Raw-record, exact-entry, and
INSERT-sequence-local lookups do not allocate payload-sized buffers.

## Test evidence

`crates/seacad-dxf-core/tests/insert_attribute_text_semantic_tests.rs` covers
ASCII/Binary parity across all nine supported dialects; exact text/tag/style
bytes; required failures; the provenance-free `STANDARD` default; duplicate
provenance; explicit replacement-free decoding; raw-record, exact-entry, and
sequence-local lookups; cancellation; source identity; bounds; and public
traits.

## Non-claims

M10.1r does not reject empty text, reject spaces in tags, compare style names
against a style table, interpret text formatting or escape sequences, project
numeric ATTRIB roles, distinguish group-280 meanings, decode MText extensions,
associate ATTDEF definitions, transform attributes, edit, write, or render.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, formatting,
generated-schema drift, workspace Clippy with warnings denied, 502 workspace
tests with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/insert_attribute_text_semantic.rs` | 373 | `f465b0f6a440bc4559a4ceb4a9012a1be52a22e0b3618bbb0a2ad84aa48fba29` |
| `crates/seacad-dxf-core/src/lib.rs` | 518 | `3fb6b166c7a808ce0da1e5628d380af8f0652ad57897a2a6693c1e9905b8939a` |
| `crates/seacad-dxf-core/tests/insert_attribute_text_semantic_tests.rs` | 323 | `19305b1b651caf08f3d6ad5ef5a9321f3db1b1b785534e146398a4644c82cbc6` |
| `docs/IMPLEMENTATION_PLAN.md` | 926 | `ab3a5fd44669ff7fa7cf8d5bca7418709080a8aae301b6611cf4c9385663305a` |
| `docs/SUPPORT_MATRIX.md` | 779 | `e20226e4441f148806461592866bf2a9553773571d46c28c13894bf7ef541068` |
