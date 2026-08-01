# M14.3q1 SPLINE Analytic Values

## Scope

M14.3q1 materializes exact knots, effective WCS control/fit points, and
positive effective control weights. It is the value layer beneath the final
analytic readiness projection and does not itself claim a usable curve.

## Evidence boundary

Autodesk's 2024 SPLINE table defines repeated group 40 knots, group 41 weights,
and group 10/20/30 and 11/21/31 WCS control and fit points:

`https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-E1F884F8-AA90-4864-A215-3182D47A9C74.htm`

The read-only legacy geometry projection requires X/Y, defaults omitted Z to
zero, applies unit weights when group 41 is absent, and rejects nonpositive
weights. The Z policy is explicitly behavioral because the cited Autodesk row
does not label the component optional.

| Legacy artifact | SHA-256 |
|---|---|
| `D:\SeaCad\tham khảo\New folder\cad_2026-07-23_source\crates\cad-io\src\dxf_native_spline_geometry.rs` | `ee17d46783e8a4b4d79c80abae3709450aab8becea7014f0199e660401abb615` |

No legacy or external code or fixture is copied, translated, vendored, or
linked.

## Contract

- Each retained SPLINE has one stable value entry over the owned auxiliary
  semantic/evidence chain.
- Every published knot retains its exact `DxfSplineValue` evidence and finite
  `DxfDouble` value.
- Control and fit points require valid X and Y. Omitted Z receives effective
  zero while `explicit_z` preserves whether the value existed. Tuple ordinals
  link every point to exact component members.
- Absent weights produce unit weights. Explicit weights publish only when the
  sequence matches control anchors and every member is valid and positive.
- Knot, control-point, fit-point, and weight issues accumulate independently.
  An unavailable entry contributes no partial values to global ranges.
- Construction is linear, uses checked compact ranges, fallible reservations,
  source-identity verification, cancellation, and no new dependency.

## Nonclaims

M14.3q1 does not validate degree, declared counts, knot order or multiplicity,
parameter domain, flags, tangents, normals, or curve readiness; evaluate or
tessellate NURBS; process HELIX; edit; write; or advance SPLINE to `Complete`.
Those final analytic readiness checks remain M14.3q2.

## Verification

The focused analytic-value suite passes 3/3 tests and the workspace passes all
771 tests. Schema and release-evidence checks, `cargo deny --locked check`,
format, workspace Clippy with warnings denied, workspace tests, the production
forbidden-macro scan, and `git diff --check` all pass. The checkpoint adds 508
physical production lines: 502 in the value module and six module/export
lines. This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 97 | `f9211e0e58b0c7e05ac8d486432acaf3262513863b3d6f751ef198d601a5ce2f` |
| `crates/seacad-dxf-core/src/lib.rs` | 896 | `079a41624bab32605065529bd876e1746ad0427db8f50d488109dea99b0fb3d6` |
| `crates/seacad-dxf-core/src/spline_analytic_value.rs` | 502 | `016399584a691a4c6801269d4fce7874b3277148f65ce556e5aa49bcddf80cfb` |
| `crates/seacad-dxf-core/tests/spline_analytic_value_tests.rs` | 383 | `44481c9dea9f70b3b0d8d0a014a18017e92715053931f6a212c5b4d5db78f562` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 457 | `e635c774c8ab4286fc589328533cb9b510260d1ea6adbe578ed94a855902058e` |
| `docs/IMPLEMENTATION_PLAN.md` | 1859 | `dc38609c0b45cbc5e95de5216f65ce960d7e1bf9650befca5758ad517404fd74` |
| `docs/SUPPORT_MATRIX.md` | 1524 | `148e87b2c8c80f33120a0620244fd6f4fecc5a9870b8fcf80b16411c00375173` |
