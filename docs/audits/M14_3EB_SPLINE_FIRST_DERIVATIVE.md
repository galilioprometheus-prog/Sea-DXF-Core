# M14.3eb SPLINE First Derivative

Retrieved: 2026-08-09

## Scope

M14.3eb adds a bounded source-bound first derivative for the rational and
non-rational SPLINE point evaluator introduced by M14.3ea. One result retains
the evaluated WCS point and its unnormalized Cartesian derivative vector.

HELIX is deliberately not routed through this API. Existing HELIX evidence
records Autodesk's warning that inherited NURBS operations have unknown
behavior and are not recommended; this checkpoint does not reinterpret that
warning as evaluator authorization.

## Evidence boundary

Autodesk's SPLINE table recorded by M14.3q1/q2 defines the WCS control points,
positive effective weights, degree, knot vector, and counts consumed here:

`https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-E1F884F8-AA90-4864-A215-3182D47A9C74.htm`

The derivative control polygon, homogeneous De Boor recurrence, and rational
quotient rule are independently authored mathematical operations over the
validated representation. No external or legacy implementation, fixture,
dependency, or private corpus data was copied, translated, vendored, linked,
or used at runtime.

## Contract

- Point and derivative evaluation share one bounded preflight for analytic
  readiness, finite closed-domain parameter admission, degree ceiling, knot and
  control input validation, span selection, and local homogeneous controls.
- For original degree `p`, local derivative controls are
  `p * (H[i+1] - H[i]) / (U[i+p+1] - U[i+1])`. Zero knot denominators remain the
  existing typed `DegenerateKnotInterval` result.
- The derivative controls use the knot vector with its first and last members
  trimmed and a degree-`p-1` De Boor recurrence. The original point uses the
  same M14.3ea recurrence implementation.
- Cartesian projection applies `(H'xyz - C * H'w) / Hw`. The point and first
  derivative are published only when all intermediate and final values remain
  finite and the homogeneous point weight remains positive.
- The public degree-64 ceiling bounds simultaneous scratch storage to 65 point
  controls plus 64 derivative controls. The two triangular recurrences perform
  at most `64^2 = 4,096` combined four-component interpolation steps.
- Cancellation is checked before lookup and publication, during homogeneous
  control loading, during derivative-control construction, and at every
  recurrence level. Allocation is fallible and indexing is checked.
- The first derivative is not normalized. An exact zero derivative remains a
  valid result so later tangent-frame or cusp policy cannot be guessed here.

## Verification boundary

One test spans every AC1009-AC1032 dialect in ASCII and Binary. A quadratic
fixture proves exact point and derivative values at both endpoints and the
midpoint and requires bit-for-bit physical-format parity. A rational linear
fixture with nonconstant weights independently proves the quotient rule.
Negative coverage proves non-finite and out-of-domain parameters, the degree-65
limit, unavailable analytic data, cancellation, missing lookup, public traits,
and compact metadata. The existing M14.3ea 4/4 suite remains unchanged and
passes through the refactored shared math path.

Normalization, Frenet or other tangent frames, higher derivatives, curvature,
adaptive sampling, tessellation, rendering, HELIX evaluation, SPLINE CRUD/
write, corpus qualification, current-checkpoint six-native CI, and SPLINE
`Complete` remain outside this checkpoint.

## Verification

The focused first-derivative suite passes 4/4 and the adjacent point-evaluation
suite passes 4/4. The workspace passes exactly 1,087 tests. Cargo-deny, Rust
1.97.1 formatting, generated-schema and release-evidence checks, workspace
Clippy with warnings denied, production safety scan, local Markdown links,
protected-surface diff, and `git diff --check` pass. No manifest, dependency,
lockfile, schema, corpus, legal, release, or license surface changes. This audit
intentionally omits its own hash.
