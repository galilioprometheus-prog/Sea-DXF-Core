# M10.1s Classic ATTRIB Integer Semantics

Retrieved: 2026-07-30

## Normative boundary

Autodesk [ATTRIB
(DXF)](https://help.autodesk.com/cloudhelp/2020/ENU/AutoCAD-DXF/files/GUID-7DD8B495-C3F8-48CD-A766-14F9D7D0DD9B.htm)
documents attribute flags `70`, optional field length `73`, text-generation
flags `71`, and horizontal/vertical justification `72/74`. Autodesk [TEXT
(DXF)](https://help.autodesk.com/cloudhelp/2020/ENU/AutoCAD-DXF/files/GUID-62E5383D-8A14-47B4-BFC4-35824CAE8363.htm)
defines the two text-generation flag bits referenced by ATTRIB.

M10.1s projects only these five unambiguous roles from M10.1p. Neutral group
`280` stays unselected because Autodesk assigns the same wire code to version
and lock-position meanings and group order is not authoritative.

## Implementation contract

`crates/seacad-dxf-core/src/insert_attribute_integer_semantic.rs` lazily
projects one typed integer-semantic view for every retained classic ATTRIB:

- attribute flags are required;
- field length, text-generation flags, and both justification values receive
  only their documented zero defaults; and
- helpers expose invisible, constant, verification-required, preset, backward,
  and upside-down bits without discarding the exact signed value or unknown
  bits.

Invalid ASCII, missing required flags, and duplicate occurrences remain typed;
raw provenance is retained whenever a source occurrence exists. Present
invalid or duplicate values never fall back to a default.

Directory construction preserves source identity, observes cancellation, and
reuses the bounded M10.1p card directory. Raw-record, exact-entry, and
INSERT-sequence-local lookups do not allocate payload-sized buffers.

## Test evidence

`crates/seacad-dxf-core/tests/insert_attribute_integer_semantic_tests.rs`
covers ASCII/Binary parity across all nine supported dialects; all five roles;
required/defaulted states; all six flag helpers; exact signed values; invalid
ASCII and duplicate provenance; retained neutral group-280 multiplicity;
raw-record, exact-entry, and sequence-local lookups; cancellation; source
identity; bounds; and public traits.

## Non-claims

M10.1s does not validate field length, reject unknown flag bits, classify
justification codes, determine alignment-point applicability, distinguish
group-280 meanings, decode MText extensions, associate ATTDEF definitions,
transform attributes, edit, write, or render.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, formatting,
generated-schema drift, workspace Clippy with warnings denied, 506 workspace
tests with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/insert_attribute_integer_semantic.rs` | 406 | `2b1a579c702a4a260962b8b855c96b83bcbeac695ea6ad022c4ec5793764b82f` |
| `crates/seacad-dxf-core/src/lib.rs` | 523 | `5626d4f236d08512a1c32761e41c4cc1919725c65b009bc08d03326570775ed7` |
| `crates/seacad-dxf-core/tests/insert_attribute_integer_semantic_tests.rs` | 314 | `8d5c06472e790ba50fbdaf5b91e7432fa68715d79ae0749d9db7ebd6e2295a11` |
| `docs/IMPLEMENTATION_PLAN.md` | 942 | `19bf454edb46c54119c94a8ef31d3404558e33cfc7e279ac930d688aa90c5ec7` |
| `docs/SUPPORT_MATRIX.md` | 791 | `61c152815929a174a9a06d6aa48dce9d6d57c4ea7f041cf4abc9555f28121b6c` |
