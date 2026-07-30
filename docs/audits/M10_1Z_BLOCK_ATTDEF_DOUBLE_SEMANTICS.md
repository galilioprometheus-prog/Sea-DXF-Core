# M10.1z Classic ATTDEF Double Semantics

Retrieved: 2026-07-30

## Normative boundary

Autodesk [ATTDEF
(DXF)](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-F0EA099B-6F88-4BCC-BEC7-247BA64838A4.htm)
defines the OCS text start and height, optional thickness/rotation/relative-X-
scale/oblique angle, justification-dependent alignment point, and optional
extrusion direction. It publishes defaults of zero thickness/rotation/oblique,
unit relative-X scale, and extrusion `(0,0,1)`.

M10.1z applies only those documented defaults and retains four-state typed
evidence for every classic double role.

## Implementation contract

`crates/seacad-dxf-core/src/block_attribute_definition_double_semantic.rs`
lazily projects the 14 M10.1y double cards:

- text-start X/Y/Z and text height are required;
- thickness, rotation, relative-X scale, oblique angle, and extrusion use only
  their documented defaults; and
- alignment-point X/Y/Z remain independently optional until justification is
  interpreted by a later milestone.

Unique values preserve exact `DxfDouble` bits and raw provenance. Invalid ASCII
numeric text, missing required values, and duplicate cards remain distinct
typed invalid states; neither invalid nor duplicate evidence receives a
default. Composite text-start, alignment-point, and extrusion accessors return
a tuple only when all three components are usable.

The directory owns its M10.1y card/evidence chain, preserves source identity,
and supports exact raw-record and BLOCK-local ATTDEF ordinal lookup without
allocating payload-sized buffers.

## Test evidence

`crates/seacad-dxf-core/tests/block_attribute_definition_double_semantic_tests.rs`
covers ASCII/Binary parity across all nine supported dialects; all 14 explicit
roles and exact bits; every documented default; missing required values;
optional alignment components; invalid ASCII syntax; duplicate values and
counts; raw provenance; partial tuples; raw-record and BLOCK-local definition
lookups; cancellation; source identity; bounded misses; and public traits.

## Non-claims

M10.1z does not validate finiteness, positivity, angles, alignment
applicability, justification combinations, or extrusion length. It does not
project ATTDEF text/integer fields, decode MText extensions, compare ATTRIB
tags, associate inserted attributes, transform geometry, edit, write, or
render.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, generated-schema
drift, formatting, workspace Clippy with warnings denied, 533 workspace tests
with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/block_attribute_definition_double_semantic.rs` | 429 | `de23f987adbfe13067db2e1018949b05dc8df98bfc94e176a87861221f290b17` |
| `crates/seacad-dxf-core/src/lib.rs` | 563 | `9a58e584de01093ca6bd7fc8e71a6d65e63b5608ef152fde2a5b942bd1339be1` |
| `crates/seacad-dxf-core/tests/block_attribute_definition_double_semantic_tests.rs` | 401 | `3c1d54231ec65af245b19a8324ee2c3287e00e348c3b8592456d2a9639c8d050` |
| `docs/IMPLEMENTATION_PLAN.md` | 1038 | `dcf0bc2354d84c9549459298a75b59e3b4e2bdfd6a16c9420a638f198c7d3f74` |
| `docs/SUPPORT_MATRIX.md` | 859 | `4bb5eec31f8336e3059232ff8911e76e5aebb4746a9e4b31eaf68dd708ba8cc4` |
