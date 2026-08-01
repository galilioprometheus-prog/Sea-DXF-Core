# M14.3q2 SPLINE Analytic Readiness

## Scope

M14.3q2 composes the existing SPLINE value, topology, count, flag-relation,
and vector-semantic layers into a single fail-closed analytic NURBS readiness
projection. It does not evaluate or tessellate the curve.

## Evidence boundary

Autodesk's 2024 SPLINE table defines flags, degree, declared counts, knots,
weights, control/fit points, optional tangents, and the planar normal:

`https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-E1F884F8-AA90-4864-A215-3182D47A9C74.htm`

The active parameter interval and the degree-plus-one multiplicity ceiling are
mathematical readiness checks over a knot vector; they are not represented as
additional Autodesk table claims. A read-only legacy validator was consulted
only as a behavioral risk oracle:

| Legacy artifact | SHA-256 |
|---|---|
| `D:\SeaCad\tham khảo\New folder\cad_2026-07-23_source\crates\cad-io\src\dxf_native_spline_geometry.rs` | `f1d1e07660ed61dae2ed0170817a4947bd55b589397e59964ccb932183258104` |

No legacy or external code or fixture is copied, translated, vendored, or
linked.

## Contract

- Every retained SPLINE has one stable analytic entry over source-identity
  checked owned prerequisite directories.
- Availability requires q1 analytic values, unique valid flags and degree,
  matched declared knot and control-point counts, and an absent fit count only
  when no fit anchors exist.
- Knot order, degree/control readiness, and the defining NURBS count equation
  must be satisfied. Knot multiplicity is at most degree plus one, and the
  active interval from knot indices `degree` and `control_count` is positive.
- Optional tangents may be absent or usable, but malformed tangent evidence
  fails readiness. Nonplanar splines omit the normal; planar splines require a
  usable nonzero normal.
- All failures accumulate in a compact typed issue mask. Unavailable entries
  publish no partial `DxfSplineAnalyticData`.
- Construction is bounded, cancellation-aware, uses fallible allocation and
  checked indexing, and adds no dependency.

## Nonclaims

M14.3q2 does not evaluate, sample, or tessellate NURBS; impose an undocumented
closed/periodic equivalence; process HELIX; edit; write; or advance SPLINE to
`Complete`.

## Verification

The focused analytic-readiness suite passes 3/3 tests and the workspace passes
all 774 tests. Schema and release-evidence checks, `cargo deny --locked check`,
format, workspace Clippy with warnings denied, workspace tests, the production
forbidden-macro scan, and `git diff --check` all pass. The checkpoint adds 478
physical production lines: 473 in the analytic module and five module/export
lines. This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 102 | `30f12863175db0d3d6c57a3504cd0ef7d57cf68a044316cef80488dfc0f92ad3` |
| `crates/seacad-dxf-core/src/lib.rs` | 901 | `6c1d2940d927e1565083ff7e7b606284e4896986d8750270b50377f58c633bda` |
| `crates/seacad-dxf-core/src/spline_analytic.rs` | 473 | `0239d7b285b30c5d71cbe0b3520bcbe604c14fb88ea239b71d7ab4b1acdbd62e` |
| `crates/seacad-dxf-core/tests/spline_analytic_tests.rs` | 332 | `94ed616aaef536ed1d72fff9e2c06ebd366ebdf6c6574c252881abd9c16bbac8` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 466 | `df4d1073b6d9ad710ec6f1cdb3b3181e39c8055ec9f6c559b643b4f3bc5f8e38` |
| `docs/IMPLEMENTATION_PLAN.md` | 1870 | `8132f353064885356cb1938935a95c7f27ceff77981f1f3dd8a9145a54b3b5e5` |
| `docs/SUPPORT_MATRIX.md` | 1535 | `c69ba7b1a2e21744b086f5a63733306ead1b99352f55e1b1e8993573f2221123` |
