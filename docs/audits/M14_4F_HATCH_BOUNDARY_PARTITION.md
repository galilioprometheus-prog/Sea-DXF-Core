# M14.4f HATCH Boundary Envelope Partition

## Scope

M14.4f partitions every exact `AcDbHatch` subclass around the canonical group
91 and group 75 fences while retaining boundary-path data as opaque raw fields.
It does not decode path or edge topology.

## Normative evidence

The Autodesk HATCH table places the declared boundary-path count in group 91,
followed by repeated boundary-path data, followed by hatch style in group 75:

`https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-C6C71CED-CE0F-4184-82A5-07AD6241F15B.htm`

The boundary-path table documents the nested group-code collisions retained in
the opaque middle span:

`https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-DC5215D6-E73F-4DFF-8BE9-01CA9610FAEE.htm`

No external or legacy implementation, fixture, data, or dependency was copied,
translated, vendored, linked, or consulted.

## Contract

- Every exact HATCH subclass receives one independent partition entry.
- Exactly one group 91 before exactly one group 75 yields three contiguous,
  non-overlapping ranges covering every retained subclass field: header through
  group 91, opaque boundary payload, and trailing data beginning with group 75.
- Both fence fields retain their exact raw group and source identity. Boundary
  groups 10/20 and every unknown retained field remain untouched and reachable.
- Missing or duplicate path-count/style fences and reversed order are typed
  issues. Invalid entries expose no range slices and no anchor is selected.
- Construction is cancellation-aware, fallibly allocated, source-identity
  checked, and bounded by the M14.4a evidence inventory.

## Nonclaims

M14.4f does not decode boundary path flags, polyline vertices, edges, source
handles, or SPLINE data; reconcile group 91 with observed paths; select an
elevation tuple; partition pattern lines, seed points, or gradient fields;
derive geometry; establish applicability; add CRUD/write behavior; qualify a
corpus; or claim `Complete` support.

## Verification

The focused boundary-partition suite passes 4/4 tests and the workspace passes
1,113/1,113 tests. Dependency policy, formatting, schema, release evidence,
workspace Clippy with warnings denied, safety/link/protected-surface scans, and
whitespace checks pass. No dependency, schema, license, release, corpus, or CI
configuration changes. The checkpoint adds 355 physical production lines: 350
in the partition module and five module/export lines. This audit intentionally
omits its own self-referential hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 204 | `e1a9e3169b62149c6e8a00403bd2feda93f79ec3511b8166c3ace1c4c3477dc0` |
| `README.vi.md` | 202 | `b41822f56dcadc5a7acd99cefa044166f82f1fff19d8161af5c5e33b52b825a3` |
| `crates/seacad-dxf-core/src/hatch_boundary_partition.rs` | 350 | `4cc56cbefc4d1eb73dfa5baaf9a3f0867774df56c1e8a347e4ec71ce07dc98d8` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,300 | `d536e23b5bf06c087cb059fbd33edbce5542cd93ba1e58deb18ee897f68183d3` |
| `crates/seacad-dxf-core/tests/hatch_boundary_partition_tests.rs` | 397 | `e6cbc4677ebbb30347e88434c54526629f7552d5030023397d6beef03162ae3b` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `73cf972f3b9647d2373ebd634b3c44290657c3ad6d89c4c7bffe2fe98ec9031c` |
| `docs/SUPPORT_MATRIX.md` | 3,067 | `e232261eaf88a7718bb07daeab47eac5b4c11caaff59b21a3a9259b3cc011ffc` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 1,990 | `66398f193bcc4069dde96819ae2eb5fdbb81f23e05f758798355f4ff8ceb6058` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,412 | `42e40acfec385fe8d832522184fd0ca2eefcfb42a732544179901717f679cbba` |
