# M14.4a HATCH/MESH Exact Evidence

## Scope

M14.4a adds one exact source-anchored raw evidence directory for canonical
HATCH and modern MESH records. It does not assign field roles, decode values,
assemble HATCH boundary paths, interpret MESH topology, or change entity
applicability.

## Normative evidence

Autodesk documents the HATCH family subclass as `AcDbHatch` and its top-level
and nested group-code envelope:

`https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-C6C71CED-CE0F-4184-82A5-07AD6241F15B.htm`

Autodesk documents the nested HATCH boundary-path data separately. Repeated and
colliding group codes therefore remain raw until a later stateful milestone:

`https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-DC5215D6-E73F-4DFF-8BE9-01CA9610FAEE.htm`

Autodesk documents the modern MESH subclass marker as `AcDbSubDMesh`, with
group codes whose meaning depends on surrounding count/list state:

`https://help.autodesk.com/cloudhelp/2017/ENU/AutoCAD-DXF/files/GUID-4B9ADA67-87C8-4673-A579-6E4C76FF7025.htm`

Autodesk also requires order-independent entity processing and defines the same
entity representation for BLOCKS and ENTITIES:

`https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-A35B8C2A-1885-4A8E-8533-E61D8A423D62.htm`

No external or legacy source, fixture, data, or implementation was copied,
translated, vendored, linked, or consulted for this checkpoint.

## Contract

- Only exact canonical uppercase HATCH and MESH record markers in completely
  indexed BLOCKS or ENTITIES sections enter the directory.
- Only exact `AcDbHatch` and `AcDbSubDMesh` subclass markers open family
  scopes. Every duplicate marker creates a distinct source-ordered scope.
- Each retained ordinary field exposes its exact `DxfRawGroup`, family, and
  subclass ordinal. Record entries and compact half-open ranges retain the
  source record boundary and deterministic lookup.
- Groups before the matching subclass, after the next subclass, inside group-
  102 application envelopes, or in XDATA do not enter family evidence. They
  remain accessible through the existing lossless raw record/group layers.
- Field codes are deliberately untyped. Repetition and collisions are retained
  in exact source order without defaulting, selection, or role assignment.
- Construction is source-identity checked, cancellation-aware, fallibly
  allocated, and bounded by the already enforced raw-document limits.

## Dialect and applicability boundary

All nine Core dialects have paired ASCII/Binary fixtures with identical record,
subclass, family, and field-code shape. This is parser evidence only. HATCH
applicability remains `NotYetReviewed`; canonical MESH retains its separately
reviewed AC1024-and-later applicability boundary.

## Nonclaims

M14.4a does not add HATCH/MESH cardinality, typed scalar or tuple semantics,
boundary-path state, loop validation, hatch evaluation, MESH face/edge state,
topology, geometry, subdivision, rendering, CRUD/write, corpus qualification,
or `Complete` support.

## Verification

The focused evidence suite passes 4/4 tests. The full workspace passes
1,093/1,093 tests. Schema and release-evidence checks, dependency policy,
formatting, workspace Clippy with warnings denied, forbidden-production scan,
protected-surface scan, Markdown-link validation, and whitespace checks pass.
No dependency, schema, license, release, corpus, or CI configuration changes.
The checkpoint adds 404 physical production lines: 399 in the evidence module
and five module/export lines. This audit intentionally omits its own
self-referential hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 192 | `1890c444e5d6cdb6472d88e4625cf32260cc41cdf0b403c1a9755cb51fba9ae6` |
| `README.vi.md` | 190 | `c82218c815efe06fb8feb69a92548d53d9424d47481fd9e0319e5e443090c6f7` |
| `crates/seacad-dxf-core/src/fill_mesh_evidence.rs` | 399 | `1c14128a4de3fa123e05f844f369f5442364660f6b75a60dca2371ec6d43ec82` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,274 | `57ded695a026dea77066e03d121c666c19ee5186389673e84f683aee663aa72b` |
| `crates/seacad-dxf-core/tests/fill_mesh_evidence_tests.rs` | 358 | `4ba42f81dac85ca940de29bd6e97f89b0cda49d0846e831eaa3217586b400189` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `157c8f3c16a24692a159cd84837d18e111995a15473ab64e45755b120b892145` |
| `docs/SUPPORT_MATRIX.md` | 3,002 | `339a021f313186b0ced4df66584456aabe3bb29349a2e78c3005fdb6054e53a0` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 1,926 | `d4eef6301ab751ace17131c7823f4632f4a31b07884420ec0a3d9df68bc82dd2` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,357 | `d393c39f1d446418d6050814a5605bb41df71c2b3439ba52df8dbb0051dce5dd` |
