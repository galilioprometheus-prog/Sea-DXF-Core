# M10.1q Classic ATTRIB Double Semantics

Retrieved: 2026-07-30

## Normative boundary

Autodesk [ATTRIB
(DXF)](https://help.autodesk.com/cloudhelp/2020/ENU/AutoCAD-DXF/files/GUID-7DD8B495-C3F8-48CD-A766-14F9D7D0DD9B.htm)
documents the classic double fields and their omitted-value defaults. M10.1q
projects only those 14 roles from the M10.1p cards. It does not infer
alignment-point applicability from justification fields.

## Implementation contract

`crates/seacad-dxf-core/src/insert_attribute_double_semantic.rs` lazily
projects one typed double-semantic view for every retained classic ATTRIB:

- text-start X/Y/Z and text height are required;
- absent thickness, rotation, relative X scale, oblique angle, and extrusion
  use only the documented `0`, `0`, `1`, `0`, and `(0,0,1)` defaults; and
- alignment-point X/Y/Z remain independently optional.

Unique ASCII and Binary values preserve exact binary64 bits. Invalid ASCII,
missing required values, and duplicate occurrences remain typed; raw
provenance is retained whenever a source occurrence exists. Present invalid or
duplicate values never fall back to a default. Tuple helpers return a value
only when all three components are usable.

Directory construction preserves source identity, observes cancellation, and
reuses the bounded M10.1p card directory. Raw-record, exact-entry, and
INSERT-sequence-local lookups do not allocate payload-sized buffers.

## Test evidence

`crates/seacad-dxf-core/tests/insert_attribute_double_semantic_tests.rs`
covers ASCII/Binary parity across all nine supported dialects; all 14 roles;
exact binary64 values; required, defaulted, and optional absence; invalid ASCII
and duplicate provenance; partial optional tuples; fail-closed tuple helpers;
raw-record, exact-entry, and sequence-local lookups; cancellation; source
identity; bounds; and public traits.

## Non-claims

M10.1q does not validate finiteness, positive height/scale, angle ranges,
justification-dependent alignment requirements, or extrusion length. It does
not project classic ATTRIB text or integer fields, distinguish group-280
meanings, decode MText extensions, associate ATTDEF definitions, transform
attributes, edit, write, or render.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, formatting,
generated-schema drift, workspace Clippy with warnings denied, 498 workspace
tests with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/insert_attribute_double_semantic.rs` | 428 | `22beb3a640e5ac2aa1ecd4f7308840af0d8390258cd4f05f232c5415d5e4c363` |
| `crates/seacad-dxf-core/src/lib.rs` | 512 | `33c2934bdfc7da2150888828d088ed1fa0caa04305768a433072b4280be0e8c8` |
| `crates/seacad-dxf-core/tests/insert_attribute_double_semantic_tests.rs` | 411 | `66ede8a57d421705ff55a9f11d935629dbe01e445d6a3340f0b8a771d6ae93b3` |
| `docs/IMPLEMENTATION_PLAN.md` | 914 | `5c46c96019cafd5033bb2a538813c256616e9274984ce3814c72b4ecadb7b75a` |
| `docs/SUPPORT_MATRIX.md` | 769 | `6985678f66801882335661d44daa9e436013f1fcfc9500830cfc414c8f3db87a` |
