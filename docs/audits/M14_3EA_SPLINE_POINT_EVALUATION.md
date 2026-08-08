# M14.3ea SPLINE Point Evaluation

Retrieved: 2026-08-09

## Scope

M14.3ea adds the first bounded geometry operation over M14.3q2 analytically
ready SPLINE data: evaluate one point at one caller-supplied parameter. It does
not sample or tessellate a curve and does not add edit or write support.

## Evidence boundary

Autodesk's SPLINE table recorded by M14.3q1/q2 defines WCS control points,
positive effective weights, degree, knot vector, and the declared counts used
by the analytic readiness gate:

`https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-E1F884F8-AA90-4864-A215-3182D47A9C74.htm`

The homogeneous De Boor recurrence is an independently authored mathematical
operation over that already validated representation. No external or legacy
implementation, source, fixture, dependency, or private corpus data was copied,
translated, vendored, linked, or used at runtime.

## Contract

- `DxfSplineAnalyticDirectory::evaluate_point_for_raw_record` keeps lookup
  source-bound and returns `None` for an unknown record ordinal.
- Only `DxfSplineAnalyticState::Available` enters arithmetic. Existing readiness
  issues are retained as `AnalyticUnavailable` rather than flattened.
- The parameter must be finite and inside the closed active knot interval. The
  exact terminal parameter selects the final control span.
- Unit and explicit positive weights are promoted to homogeneous `[xw,yw,zw,w]`
  points, interpolated with De Boor, and divided by the resulting positive
  weight. Derived signed zero is canonicalized to positive zero.
- Degree 64 is a fixed public CPU/scratch ceiling. Evaluation reserves at most
  65 homogeneous points and performs at most 4,096 four-component recurrence
  steps. Higher degree is a typed unavailable result.
- Non-finite parameters and stored inputs, out-of-domain parameters, degenerate
  knot denominators, arithmetic overflow, and nonpositive resulting weight are
  separate typed issues. Cancellation is checked before lookup, while loading
  controls, at every recurrence level, and before publication.
- Allocation is fallible, indexing and integer conversion are checked, no
  dependency is added, and source bytes are never changed.

## Verification boundary

One test spans every AC1009-AC1032 dialect in ASCII and Binary and proves
byte-identical output bits at the quadratic endpoints and midpoint. A separate
rational linear fixture proves homogeneous weight division. Negative coverage
proves unavailable analytic input, non-finite and out-of-domain parameters,
the degree-65 ceiling, a Binary non-finite control coordinate, cancellation,
missing lookup, public traits, and compact result metadata.

Adaptive sampling, derivatives, arc-length measurement, tessellation,
rendering, extrapolation/wrapping, HELIX evaluation, SPLINE edit/write, corpus
qualification, current-checkpoint six-native CI, and SPLINE `Complete` remain
outside this checkpoint.

## Verification

The focused evaluator suite passes 4/4. The workspace passes exactly 1,083
tests. Cargo-deny, Rust 1.97.1 formatting, generated-schema and release-evidence
checks, workspace Clippy with warnings denied, production safety scan, local
Markdown links, protected-surface diff, and `git diff --check` pass. No
manifest, dependency, lockfile, schema, corpus, legal, release, or license
surface changes. This audit intentionally omits its own hash.
