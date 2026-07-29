# M9.2i Classic POLYLINE Family Semantics

Retrieved: 2026-07-30

## Normative boundary

- Autodesk [POLYLINE (DXF)](https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-ABF6B778-BE20-4B49-9B58-A94E64CEFFF3.htm)
  identifies parent flag bits `8`, `16`, and `64` as 3D polyline, polygon mesh,
  and polyface mesh; no family bit denotes the ordinary 2D form.
- Autodesk [VERTEX (DXF)](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-0741E831-599E-4CBF-91E1-8ADBCFD6556D.htm)
  identifies VERTEX bits `32`, `64`, and `128`. For polyface meshes, `128|64`
  is a coordinate vertex while `128` without `64` is a face record.
- Neither table defines precedence for contradictory simultaneous family bits.
  M9.2i therefore retains those combinations as conflicts.

## Implementation contract

`crates/seacad-dxf-core/src/polyline_family_semantic.rs` provides lazy:

- parent classification as 2D, 3D, polygon mesh, or polyface mesh;
- VERTEX classification as 2D, 3D, polygon mesh, polyface coordinate, or
  polyface face;
- `Unavailable` for absent/invalid/duplicate flags and `Conflicting` with the
  exact family-bit mask for contradictory combinations;
- `Matched`, `Mismatched`, or `NotComparable` parent/VERTEX comparison without
  enforcing consistency or choosing precedence;
- retained M9.2g/h semantic graphs, source identity, ASCII/Binary adapters,
  bounded cancellation, and raw-ordinal lookup.

## Test evidence

`crates/seacad-dxf-core/tests/polyline_family_semantic_tests.rs` covers:

- ASCII/Binary parity across every supported AC1009-AC1032 dialect;
- all four parent families, all five VERTEX families, and both polyface forms;
- matched, mismatched, unavailable, and conflicting evidence;
- parent default-zero flags without inventing a VERTEX flag default;
- source identity, lookup bounds, cancellation, and public traits.

## Non-claims

M9.2i does not validate subclass/version applicability, require parent/VERTEX
agreement, interpret polyface indices, resolve faces, transform OCS/WCS,
assemble geometry, edit, write, render, or diagnose conformance.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, formatting,
generated-schema drift, workspace Clippy with warnings denied, 392 workspace
tests with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/polyline_family_semantic.rs` | 277 | `f83967fbfa217b5e9470021d73f0152b7fb925838df88ee529b4015e828f93f4` |
| `crates/seacad-dxf-core/src/lib.rs` | 358 | `a32f0f2d327f6d49e8c31680ef938d336b346c1796bf0e889548b83c31a0ef1d` |
| `crates/seacad-dxf-core/tests/polyline_family_semantic_tests.rs` | 254 | `6b0a2314ce11bc6d34cf8224a9b93ad632569cca059786e87bab76f2819f9746` |
| `docs/IMPLEMENTATION_PLAN.md` | 593 | `522deb53895310ccdd34f7cbc4d824b58e2b0ebcfff76f0b85837cf562f0ef30` |
| `docs/SUPPORT_MATRIX.md` | 498 | `5d71b665c332291374fd055f70f8d3d3105451498f71ac304b2e1b9cb0ab6756` |
