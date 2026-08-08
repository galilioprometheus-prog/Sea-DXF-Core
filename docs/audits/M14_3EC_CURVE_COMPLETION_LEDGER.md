# M14.3ec Curve Completion Ledger

Retrieved: 2026-08-09

## Decision

M14.3ec audits SPLINE and HELIX against the six entity-completion levels before
the roadmap advances to M14.4 fills and meshes.

- SPLINE reaches `Geometry`, level 4 of 6.
- HELIX reaches `TypedSemantics`, level 3 of 6.
- POINT remains unchanged at `VerifiedMutation`, level 5 of 6.
- None of the three audited entities is `ReleaseQualified` or `Complete`.

The public assessment list is sorted by canonical topic ordinal and lookup uses
that invariant deterministically. A missing topic remains unaudited, not
unsupported; 42 of the 45 canonical topics therefore still have no assessment.

## SPLINE evidence

| Level | Evidence | Result |
|---|---|---|
| 1. Exact evidence | M14.3a retains exact record-scoped numeric occurrences and source provenance. | Satisfied |
| 2. Cardinality | M14.3b publishes 25 stable per-role cards without selecting duplicates. | Satisfied |
| 3. Typed semantics | M14.3c-q2 validate flags, scalars, counts, tuples, positive weights, vectors, relations, topology, knot order/multiplicity, active domain, and analytic readiness. | Satisfied |
| 4. Geometry | M14.3ea evaluates bounded rational/non-rational WCS points; M14.3eb evaluates the matching rational first derivative with physical-format parity. | Satisfied |
| 5. Verified mutation | No source-bound SPLINE create/update/clone/delete path with strict verification exists yet. | Blocked |
| 6. Release qualification | Private corpus and current-checkpoint six-native evidence are absent. | Blocked |

The exact SPLINE blockers are `VerifiedMutation`,
`PrivateCorpusQualification`, and `CurrentCheckpointSixNativeCi`.

## HELIX evidence

| Level | Evidence | Result |
|---|---|---|
| 1. Exact evidence | M14.3r retains the 16 documented `AcDbHelix` roles only inside exact subclass scope. | Satisfied |
| 2. Cardinality | M14.3s publishes 16 stable cards with absent, unique, and duplicate-preserving states. | Satisfied |
| 3. Typed semantics | M14.3t-w publish scalar/vector semantics, axis/radius/turn/height relations, ordered subclass paths, and fail-closed embedded-SPLINE readiness. | Satisfied |
| 4. Geometry | Autodesk documents HELIX as a spline approximation and warns that inherited NURBS operations have unknown behavior and are not recommended. Exact stored-curve evaluation is not qualified. | Blocked |
| 5. Verified mutation | No source-bound HELIX create/update/clone/delete path with strict verification exists. | Blocked |
| 6. Release qualification | Private corpus and current-checkpoint six-native evidence are absent. | Blocked |

The exact HELIX blockers are `PublicGeometryQualification`,
`VerifiedMutation`, `PrivateCorpusQualification`, and
`CurrentCheckpointSixNativeCi`. The existing ideal axis/radius/height relations
are not silently promoted into stored-curve geometry.

## Verification

The focused completion-ledger suite passes 8/8. It proves exact levels and
blocker order for POINT, SPLINE, and HELIX; levels satisfied and rejected by
each entry; false completion states; unique ordinal-sorted lookup; 42 unaudited
topics; compact storage; and stable public traits. The workspace passes exactly
1,089 tests. Cargo-deny, Rust 1.97.1 formatting, generated-schema and
release-evidence checks, workspace Clippy with warnings denied, production
safety scan, local Markdown links, protected-surface diff, and
`git diff --check` pass. No manifest, dependency, lockfile, schema, corpus,
legal, release, or license surface changes. This audit intentionally omits its
own hash.
