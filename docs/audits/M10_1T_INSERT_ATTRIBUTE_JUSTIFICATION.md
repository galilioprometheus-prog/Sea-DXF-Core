# M10.1t Classic ATTRIB Justification

Retrieved: 2026-07-30

## Normative boundary

Autodesk [ATTRIB
(DXF)](https://help.autodesk.com/cloudhelp/2020/ENU/AutoCAD-DXF/files/GUID-7DD8B495-C3F8-48CD-A766-14F9D7D0DD9B.htm)
assigns classic horizontal justification to group `72`, vertical justification
to group `74`, and says the alignment point is present only when either value
is nonzero. Autodesk [TEXT
(DXF)](https://help.autodesk.com/cloudhelp/2020/ENU/AutoCAD-DXF/files/GUID-62E5383D-8A14-47B4-BFC4-35824CAE8363.htm)
publishes horizontal codes `0..5` and vertical codes `0..3`, which ATTRIB
references.

## Implementation contract

`crates/seacad-dxf-core/src/insert_attribute_justification.rs` lazily
classifies usable M10.1s values as:

- horizontal left, center, right, aligned, middle, or fit; and
- vertical baseline, bottom, middle, or top.

Typed semantic state preserves whether the original code was explicit or
defaulted, along with exact raw provenance when present. Codes outside the
published domains become typed invalid values. Underlying ASCII/cardinality
failures are retained as nested typed issues.

Alignment-point applicability is `true` when either classified code is
nonzero, `false` only when both classified codes are usable zero values, and
unavailable otherwise. This is applicability evidence only; it does not select
or validate a coordinate tuple.

Directory construction preserves source identity and reuses the cancellable,
bounded M10.1s integer directory. Raw-record, exact-entry, and
INSERT-sequence-local lookups do not allocate payload-sized buffers.

## Test evidence

`crates/seacad-dxf-core/tests/insert_attribute_justification_tests.rs` covers
ASCII/Binary parity across all nine supported dialects; every published
horizontal and vertical code; explicit and defaulted states; zero/nonzero
alignment applicability; unsupported signed codes; nested invalid ASCII
evidence; raw provenance; raw-record, exact-entry, and sequence-local lookups;
cancellation; source identity; bounds; and public traits.

## Non-claims

M10.1t does not validate horizontal/vertical combinations, require or select
text-start/alignment tuples, recalculate placement, measure styled text,
decode MText extensions, associate ATTDEF definitions, transform attributes,
edit, write, or render.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, formatting,
generated-schema drift, workspace Clippy with warnings denied, 510 workspace
tests with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/insert_attribute_justification.rs` | 277 | `d1d7a8cf9b8e3d6e2aef27e9a93e62c6fdc3fec09f1bfbe08a437535f2b85926` |
| `crates/seacad-dxf-core/src/lib.rs` | 530 | `4f61d242ba4dcbdc6d8941b8afd98a09272c66a8a03731f353b4687ccb7979ea` |
| `crates/seacad-dxf-core/tests/insert_attribute_justification_tests.rs` | 355 | `c4333fe780dec3f2c66fd529135e1cffb4b60d0d2453b58a21d176131f2b2ac9` |
| `docs/IMPLEMENTATION_PLAN.md` | 955 | `dfcc79863f519685fb18b44ff75ddeea3ad84989b341558f0a15f5627a903c78` |
| `docs/SUPPORT_MATRIX.md` | 800 | `51db8538661ffe0b27faf04b9dc4521d49c54a8c766f3931cbef577b8f144007` |
