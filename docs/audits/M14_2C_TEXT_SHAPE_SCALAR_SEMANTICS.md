# M14.2c TEXT/SHAPE Scalar Semantics

Retrieved: 2026-07-31

## Normative boundary

Autodesk's [TEXT
reference](https://help.autodesk.com/cloudhelp/2016/ENU/AutoCAD-DXF/files/GUID-62E5383D-8A14-47B4-BFC4-35824CAE8363.htm)
defines required first-alignment coordinates and text height; optional second
alignment; zero thickness, rotation, oblique, generation, and justification
defaults; width-factor default one; and extrusion default `(0, 0, 1)`.

Autodesk's [SHAPE
reference](https://help.autodesk.com/cloudhelp/2025/ENU/AutoCAD-DXF/files/GUID-0988D755-9AAB-4D6C-8E26-EC636F507F2C.htm)
defines required insertion coordinates and size with the same thickness,
rotation, width-factor, oblique, and extrusion defaults.

The documentation does not authorize choosing an effective TEXT alignment
point, accepting unsupported layout codes, validating numeric ranges, or
deriving geometry at this checkpoint.

## Legacy evidence boundary

`D:\SeaCad\cad_2026-07-23_source` was inspected only as a read-only behavioral
oracle. Its Q4.2 notes and semantic diagnostics identify missing coordinates,
partial optional alignment tuples, invalid numeric input, duplicate fields,
generation/justification codes, and extrusion defaults as compatibility risks.
No parser, semantic implementation, validator, test fixture, dependency, or
source text was copied, translated, vendored, or linked. Autodesk documentation
remains normative.

## Implementation contract

`DxfTextShapeScalarDirectory` retains the complete M14.2b card directory and
constructs lazy public scalar semantics only for exact TEXT and SHAPE records:

- required fields become `Invalid(MissingRequiredValue)` when absent;
- documented defaults become `Defaulted` without invented raw provenance;
- optional TEXT second-alignment components become `Absent`;
- unique valid numbers become `Explicit` with exact group occurrence/span;
- invalid ASCII numbers remain typed and source-anchored; and
- duplicates become typed `MultipleValues` with occurrence count and first raw
  occurrence as diagnostic provenance.

TEXT exposes first/second alignment, height, thickness, rotation, width,
oblique, generation flags, horizontal/vertical justification, and extrusion.
SHAPE exposes insertion, size, thickness, rotation, width, oblique, and
extrusion. ASCII and Binary documents share the same cardinality-driven
selection path.

## Test evidence

`text_shape_scalar_tests.rs` covers all nine AC1009--AC1032 dialects in ASCII
and Binary; bit-exact explicit/default parity; every documented scalar default;
optional second-alignment absence and partial tuples; missing required values;
invalid ASCII doubles and signed-16 values; duplicate values; first-occurrence
diagnostic provenance; independent extrusion components; family scope; source
identity; lookup bounds; cancellation; and public `Copy`/`Send`/`Sync` bounds.

## Non-claims

M14.2c does not decode content, style, or shape names; validate positive
height/size/width or generation/justification ranges; choose first versus
second alignment; transform OCS/WCS coordinates; validate or normalize
extrusion; resolve styles or shape definitions; produce glyph geometry; edit;
or write. MTEXT and TOLERANCE scalar selection, MTEXT group-50 precedence and
column structure, background ownership, and ordered text chunks remain later
checkpoints.

## Required checkpoint gates

All required gates passed on 2026-07-31: dependency policy, generated-schema
drift, release-evidence drift, formatting, workspace Clippy with warnings
denied, 627 workspace tests with zero failures/ignored tests, and
`git diff --check`. No dependency manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 78 | `17001fc144e8a0bcee3cb30bf6bbbb31edba51026d80db6b145c6e0abcfd20f6` |
| `crates/seacad-dxf-core/src/lib.rs` | 666 | `89fc2abc47649cda79c344860ecb8666a3dd15d88e41dfca54f039c5ea9584d6` |
| `crates/seacad-dxf-core/src/text_shape_scalar.rs` | 296 | `d39b0945bf9fcd902421845e438da08aff26daba27e752388b51ffd762ae2ae5` |
| `crates/seacad-dxf-core/src/text_shape_scalar_build.rs` | 302 | `278218b2b18545b3f082c3e7bd7b955694c502ca2749351ca15678252b66c330` |
| `crates/seacad-dxf-core/src/text_shape_scalar_value.rs` | 153 | `e6e6203e60b1af654baac2ff52a40d28df7356f9fe17fd9b34db3e0b00979979` |
| `crates/seacad-dxf-core/tests/text_shape_scalar_tests.rs` | 441 | `7ec0ac9c4531b5d81be30e7b2a5c5f2518eb75f084848e534fa8ac089d71797f` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 115 | `c17fc8c7b4275e295a97224bc53d29889ec853903f7f1830a245f8813c9eaefa` |
| `docs/IMPLEMENTATION_PLAN.md` | 1416 | `d65059d8ba7c86b61c0e4e83a5810814308befa62af0d15dc970f46a06f84941` |
| `docs/SUPPORT_MATRIX.md` | 1133 | `7208b5790c18fee2cc5ab0a9a22aced704fd1cf351f7cc3975a127a0d472d006` |
