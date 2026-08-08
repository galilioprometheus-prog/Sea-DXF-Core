# M14.4b HATCH Scalar Evidence

## Scope

M14.4b adds typed source-anchored evidence for the 25 HATCH roles whose group
codes do not change meaning inside Autodesk's nested HATCH grammar. It does not
select duplicates, apply defaults or domains, assemble tuples, or parse
boundary paths and pattern lines.

## Normative evidence

Autodesk's HATCH table defines the top-level and gradient-envelope roles:

`https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-C6C71CED-CE0F-4184-82A5-07AD6241F15B.htm`

The separate boundary-path and pattern-line tables identify the colliding codes
that remain excluded from this typed layer:

`https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-DC5215D6-E73F-4DFF-8BE9-01CA9610FAEE.htm`

`https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-7C05C0EC-B0FB-4A86-A164-B9E5C6C03990.htm`

Autodesk's group-value type table defines codes 60-79 as 16-bit integers,
90-99 as 32-bit integers, 450-459 as long values, 460-469 as doubles, and
470-479 as strings:

`https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-2553CF98-44F6-4828-82DD-FE3BC7448113.htm`

No external or legacy implementation, fixture, data, or dependency was copied,
translated, vendored, linked, or consulted.

## Contract

- Every exact M14.4a `AcDbHatch` subclass receives one independent scalar entry
  linked to its raw subclass marker, source record, and global subclass ordinal.
- The fixed public role inventory contains 25 roles: elevation Z; extrusion;
  pattern name/flags/counts/style/type/angle/scale; pixel size; seed count; and
  the optional gradient envelope.
- Text values retain exact source payload spans without decoding. Numeric
  values retain exact `Double`, `Int16`, or `Int32` results and typed invalid
  ASCII syntax/range issues.
- Duplicates remain source ordered. No occurrence is selected and no default,
  domain, count relation, or semantic condition is applied.
- Boundary-path, edge, polyline, spline-edge, pattern-line, seed-coordinate,
  MPolygon-only, application-group, and XDATA values do not collide with the
  typed inventory and remain available through M14.4a/raw layers.
- Construction is source-identity checked, cancellation-aware, fallibly
  allocated, and bounded by the raw-document resource profile.

## Dialect boundary

Paired ASCII/Binary fixtures cover all nine Core dialects. AC1009 omits group
450-470 in both physical formats because its one-byte Binary entity group-code
grammar cannot encode those codes. This is physical wire evidence only; HATCH
applicability remains `NotYetReviewed`.

## Nonclaims

M14.4b does not add cardinality cards, defaults, domains, tuples, HATCH boundary
or pattern state, gradient relations, topology, geometry, rendering,
applicability, CRUD/write, corpus qualification, or `Complete` support.

## Verification

The focused HATCH scalar suite passes 4/4 tests and the workspace passes
1,097/1,097 tests. Dependency policy, formatting, schema, release evidence,
workspace Clippy with warnings denied, safety/link/protected-surface scans, and
whitespace checks pass. No dependency, schema, license, release, corpus, or CI
configuration changes. The checkpoint adds 400 physical production lines: 395
in the scalar module and five module/export lines. This audit intentionally
omits its own self-referential hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 194 | `1f0b3158c7539b57b4add87b13940c15fd29f5d7adb40ca32b9cceb1b805cbcf` |
| `README.vi.md` | 192 | `47cc4ac292063f2068f9104e837bcd9eb450580c0f5061dd3dd7be093518e9f2` |
| `crates/seacad-dxf-core/src/fill_mesh_evidence.rs` | 399 | `814364cbf0b63b7b060c38a0b08d0975dc7befc18051cc00fbb63ad2545d1fa1` |
| `crates/seacad-dxf-core/src/hatch_scalar_evidence.rs` | 395 | `4e76086db9803d9e56b497df55bddcf942554f77e9a7978227ca72bb59410aec` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,279 | `12290c2cd4bcc984b85c8a0fada60be50365b037cf24628221b1a09b2220c5f4` |
| `crates/seacad-dxf-core/tests/hatch_scalar_evidence_tests.rs` | 457 | `95c76aabaaffe0e2c7cfd1be1ba0d659853705e621f67571cb5bc59baca62af3` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `360508086fc7f5be54e1dbfc6f7c21d5782278729f6987a1aeab976678f735be` |
| `docs/SUPPORT_MATRIX.md` | 3,016 | `72dffc897b66c384bcbb6e6ddc523ac1b0f953b498dcc9d5afcaac68b5663113` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 1,942 | `c2762d066c9ce78512b0ef9d94a8ad947ed677f0c32c83b22b7f1360f5ad6628` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,369 | `095e144a7b31f7ad60b5024e8eb90215fbabdd185216684b8a5842a4ef1a4a89` |
