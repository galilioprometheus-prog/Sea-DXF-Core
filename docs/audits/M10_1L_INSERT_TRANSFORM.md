# M10.1l INSERT Transform

Retrieved: 2026-07-30

## Normative boundary

Autodesk documents the INSERT insertion point, scale factors, rotation, and
extrusion in [INSERT
(DXF)](https://help.autodesk.com/cloudhelp/2018/ENU/AutoCAD-DXF/files/GUID-28FA4CFB-9D5E-4880-9F11-36C97578252F.htm).
The [Object Coordinate
System](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-D99F1509-E4E4-47A3-8691-92EA07DC88F5.htm)
defines OCS-to-WCS placement, and Autodesk's [arbitrary-axis
algorithm](https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-E19E5B42-0CC7-4EBA-B29F-5E1D595149EE.htm)
defines the normalized OCS basis and its `1/64` branch.

M10.1l combines those rules with the target BLOCK base point already retained
by SeaCad. It derives evidence; it does not claim that every transformable
record is a conforming AutoCAD INSERT.

## Implementation contract

`crates/seacad-dxf-core/src/insert_transform.rs` publishes one entry for every
M10.1k eligibility entry. Eligible single INSERTs receive a finite row-major
3x4 affine matrix from BLOCK coordinates to WCS:

`WCS = OCS_basis * (Insertion + Rz * Scale * (BlockPoint - BasePoint))`

The public transform also retains the normalized extrusion normal and applies
the matrix to caller-supplied finite points. Derived zero components are
canonicalized to positive zero for deterministic bit-level evidence.

Typed failures retain ineligible targets, unavailable INSERT semantics,
unavailable BLOCK base points, each non-finite input class, zero-length
extrusion, non-finite derived matrices, non-finite application input, and
non-finite application results. Directory construction preserves source
identity, checks cancellation, uses fallible bounded allocation, and keeps the
M10.1k eligibility evidence reachable.

## Test evidence

`crates/seacad-dxf-core/tests/insert_transform_tests.rs` covers ASCII/Binary
matrix parity across all nine supported dialects; identity and nonzero BLOCK
base points; nonuniform scale; rotation; both arbitrary-axis branches;
extrusion normalization; exact positive-zero canonicalization; every
non-finite input class; zero extrusion; derived and application overflow;
unavailable semantics/base points; unresolved targets; cancellation; lookup;
source identity; and public traits.

## Non-claims

M10.1l derives one INSERT instance only. It does not expand row/column arrays,
convert BLOCK units, follow ATTRIB/SEQEND, recursively transform member
geometry, edit, write, or render.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, formatting,
generated-schema drift, workspace Clippy with warnings denied, 478 workspace
tests with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/insert_transform.rs` | 396 | `7aa92fbc97babae862594e682ad7d5788326f40a21771e70ffcdbc5d7d9f2c86` |
| `crates/seacad-dxf-core/src/lib.rs` | 485 | `adf62c95b1559c9371fd757da0052d727228c82d18c4a10596355a3864944a51` |
| `crates/seacad-dxf-core/tests/insert_transform_tests.rs` | 416 | `23b71d7264468b1ac6af3f0f40d926515764112a5556292e903aa323d8b7e2c2` |
| `docs/IMPLEMENTATION_PLAN.md` | 846 | `d077e91fb254dabf548be14e18c140758789fdbb029e237595d3524fe14dbec2` |
| `docs/SUPPORT_MATRIX.md` | 720 | `735620d04f8789e8fb1c317ae564d8c81b7993417b8fb94cfc32680755c37c2f` |
