# M14.4e HATCH Extrusion Tuple

## Scope

M14.4e assembles the reviewed M14.4d extrusion components into one exact,
non-normalized tuple per `AcDbHatch` subclass. It preserves the selected
component provenance and fails closed when a usable vector cannot be proven.

## Normative evidence

Autodesk defines groups 210, 220, and 230 as the optional extrusion direction
and documents the omitted value as `(0,0,1)` in the HATCH entity table:

`https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-C6C71CED-CE0F-4184-82A5-07AD6241F15B.htm`

The boundary-path table shows why elevation groups 10/20 cannot yet be safely
assembled without stateful partitioning:

`https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-DC5215D6-E73F-4DFF-8BE9-01CA9610FAEE.htm`

No external or legacy implementation, fixture, data, or dependency was copied,
translated, vendored, linked, or consulted.

## Contract

- Every exact HATCH subclass receives one independent extrusion entry, including
  duplicate subclasses within one raw entity record.
- X/Y/Z preserve exact `DxfDouble` bits and explicit/defaulted input kind.
  Partially explicit vectors use only the reviewed independent `0/0/1` defaults.
- Duplicate, malformed, invalid, or non-finite component semantics do not fall
  back to defaults. They yield an unavailable-component bitmask.
- Exact all-zero vectors, including signed zero, yield `ZeroVector`. Usable
  vectors are never normalized.
- Construction is source-identity checked, cancellation-aware, fallibly
  allocated, and bounded by the retained scalar directories.

## Nonclaims

M14.4e does not assemble elevation, partition boundary paths, pattern lines, or
seed points, normalize or transform the extrusion, derive HATCH geometry,
establish version applicability, add CRUD/write behavior, qualify a corpus, or
claim `Complete` support.

## Verification

The focused HATCH extrusion suite passes 4/4 tests and the workspace passes
1,109/1,109 tests. Dependency policy, formatting, schema, release evidence,
workspace Clippy with warnings denied, safety/link/protected-surface scans, and
whitespace checks pass. No dependency, schema, license, release, corpus, or CI
configuration changes. The checkpoint adds 338 physical production lines: 332
in the extrusion module and six module/export lines. This audit intentionally
omits its own self-referential hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 201 | `f9484786d9bc6a9f8421a51dbe79127ab4bbb2da718cecc80431c246bd4ca6de` |
| `README.vi.md` | 199 | `1d96b45825ed3978b5606d0108dcb7a4995393d6ae50ee7e8ec19f591afa5916` |
| `crates/seacad-dxf-core/src/hatch_extrusion.rs` | 332 | `d19c3906f909be8fe9b57c6d62457e1a7ac0984e6525a07770812165be7e6916` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,295 | `d6412aaad09cceb7dd2c21bab2e614acaa70bf9bc2c7c459a88a185a94a12c40` |
| `crates/seacad-dxf-core/tests/hatch_extrusion_tests.rs` | 351 | `83ae95388810e2a1b50a0e4dcf28b073366a4d2f4fd4c5b247548240535d0cbf` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `2e48bca3ff35b69dd1fd56d6adb1109a0c253d985c013a957e9434ab95d82710` |
| `docs/SUPPORT_MATRIX.md` | 3,053 | `6d228de32843837cb5e43a4515dafbe011d90d01842468466893e122347a8c94` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 1,979 | `dd01ead3b6fc553d7fb59fe8fbd8ffed931c9e1c038e3151500995afcbd23d2e` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,401 | `70be6484622cbee6f3a541fad87b4730e08db02217449efbc3209f46d905d403` |
