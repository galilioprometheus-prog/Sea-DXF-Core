# M10.1n INSERT Attribute Sequence

Retrieved: 2026-07-30

## Normative boundary

Autodesk [INSERT
(DXF)](https://help.autodesk.com/cloudhelp/2018/ENU/AutoCAD-DXF/files/GUID-28FA4CFB-9D5E-4880-9F11-36C97578252F.htm)
states that attributes-follow group `66` defaults to zero and that value `1`
expects following ATTRIB entities terminated by SEQEND. Autodesk [SEQEND
(DXF)](https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-FD4FAA74-1F6D-45F6-B132-BF0C4BE6CC3B.htm)
describes the INSERT attribute terminator using a present nonzero group `66`.

M10.1n reconciles those wordings conservatively: zero consumes no sequence;
any exact nonzero signed-16-bit value scans sequence topology while remaining
unchanged and visible to callers. SeaCad does not normalize a non-`1` value
into a conforming flag.

## Implementation contract

`crates/seacad-dxf-core/src/insert_attribute_sequence.rs` publishes one entry
for every M10.1i INSERT semantic record:

- `NoAttributesFollow` for usable zero;
- `FlagUnavailable` for invalid or duplicate group `66` evidence;
- `Closed` when zero or more consecutive exact uppercase ATTRIB records are
  followed by exact uppercase SEQEND;
- `Interrupted` with the first same-section unexpected record; or
- `Unclosed` when the containing complete section ends first.

Entries retain the exact attributes-follow value, compact ranges into one
source-order ATTRIB-record array, and the exact boundary record when present.
Scanning is restricted to the same completely indexed BLOCKS or ENTITIES
section. Marker comparisons are byte-exact, cancellation is checked throughout,
and all growth uses fallible bounded allocation.

## Test evidence

`crates/seacad-dxf-core/tests/insert_attribute_sequence_tests.rs` covers
ASCII/Binary parity across all nine supported dialects; multiple and empty
closed sequences; non-`1` nonzero retention; zero consuming no ATTRIB/SEQEND;
interrupted and section-end-unclosed sequences; invalid and duplicate flags;
exact marker case; compact ranges; boundary identity; cancellation; source
identity; directory lookup; and public traits.

## Non-claims

M10.1n does not decode ATTRIB fields, associate ATTDEF definitions, validate
ownership, apply attribute transforms, expand member geometry, edit, write, or
render.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, formatting,
generated-schema drift, workspace Clippy with warnings denied, 487 workspace
tests with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/insert_attribute_sequence.rs` | 339 | `aeaf5e7e40e1a95c643612b42a467b54702102ae9449f193ea09f1efed6b39b4` |
| `crates/seacad-dxf-core/src/lib.rs` | 495 | `ace673d795f67c89d7e370851a159478dbf49415249818f57f749ce162a0b5d6` |
| `crates/seacad-dxf-core/tests/insert_attribute_sequence_tests.rs` | 299 | `52b7fc373d0486aff7da1a11681ce90a33dc68468ec18baea725aea9d7108f88` |
| `docs/IMPLEMENTATION_PLAN.md` | 871 | `ceddcd6c4f945a98c672b1c4e20da55eb22568ab3fee53dfb1915cc92cfc46c5` |
| `docs/SUPPORT_MATRIX.md` | 740 | `09c1189f7b5dcbf744b016f1285f0b979f5a8fd6b47892bd862fc5a7d3468143` |
