# M10.1i INSERT Record Semantics

Retrieved: 2026-07-30

## Normative boundary

Autodesk [INSERT
(DXF)](https://help.autodesk.com/cloudhelp/2018/ENU/AutoCAD-DXF/files/GUID-28FA4CFB-9D5E-4880-9F11-36C97578252F.htm)
marks scale, rotation, array counts/spacing, attributes-follow, and extrusion
as optional and documents their defaults. It does not mark block name or
insertion-point components optional.

## Implementation contract

`crates/seacad-dxf-core/src/insert_record_semantic.rs` lazily projects:

- required block name and insertion point `(x,y,z)`;
- scale default `(1,1,1)` and rotation default `0` degrees;
- column/row count default `(1,1)` and spacing default `(0,0)`;
- attributes-follow default `0`;
- extrusion default `(0,0,1)`.

Each field distinguishes explicit, defaulted, and invalid state with stable
field provenance and exact raw provenance when source evidence exists.
Invalid ASCII numbers and multiple values remain invalid rather than receiving
a default. Composite accessors return no value if any component is unusable.

## Test evidence

`crates/seacad-dxf-core/tests/insert_record_semantic_tests.rs` covers
ASCII/Binary parity across all nine supported dialects, every explicit value,
every documented default, missing required fields, invalid ASCII values,
duplicates, first-occurrence provenance, composite fail-closed behavior,
lookups, cancellation, source identity, and public traits.

## Non-claims

M10.1i does not validate empty names, numeric finiteness, count ranges, or
attributes-follow values; resolve a block target; follow ATTRIB/SEQEND; form an
OCS transform; edit, write, or render.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, formatting,
generated-schema drift, workspace Clippy with warnings denied, 463 workspace
tests with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/insert_record_semantic.rs` | 480 | `cb5b7134d3b06ce5b4f8c6d923b0aeb6ca7ebae3354d854f5a882a0a8cc3cb2f` |
| `crates/seacad-dxf-core/src/lib.rs` | 470 | `9ab115fabf8da18fa0a3324a8665e40a1312bfacb25f5fd039862564a6a8cf4a` |
| `crates/seacad-dxf-core/tests/insert_record_semantic_tests.rs` | 379 | `f3d1cfcf0355c67ce9a660eec425f5d0d43c273039cc26562b58a10864871fd8` |
| `docs/IMPLEMENTATION_PLAN.md` | 812 | `919d2073dbe3be70ee64b1d7dd85f966987642c2253ad44dc3bee8512179c4cb` |
| `docs/SUPPORT_MATRIX.md` | 692 | `7cbf45c4082dcf0d6181d47422df5ed326b28cfeb2b7d86119525a98e19309d5` |
