# SeaCad

SeaCad is a proprietary, clean-room CAD kernel project. Its first release target
is a lossless and resource-bounded DXF core for ASCII and Binary DXF AC1009
through AC1032.

## Current status

Release-evidence implementation is complete through M13.2g and entity-semantic
expansion is complete through M14.3ac. SeaCad opens bounded lossless ASCII
and Binary DXF AC1009 through AC1032, preserves exact source identity and raw
evidence, exposes the reviewed HEADER/record/entity semantics and geometry,
plans reversible handle and unique common-field edits, writes verified
preserve-patch outputs, and
emits strictly reparsed canonical ASCII or Binary framing. Dependency policy,
deterministic CycloneDX inventory, distributable legal files, six-native CI
staging, a redacted 1,000-file/10-GiB corpus release gate, and a manual
six-target native artifact assembly workflow, exact six-artifact receipt
aggregation, a host-independent six-platform SBOM dependency union, actionable
stale-evidence diagnostics, canonical-LF Cargo.lock identity, and one
successful six-native artifact/aggregate workflow receipt are present. Private
corpus achievement, twenty consecutive six-native nightly receipts,
signatures, and final Core 1.0 authorization remain open M13 evidence.
The generated schema freezes the complete reviewed HEADER inventory, all 45
canonical Autodesk entity topics, and 14 reviewed alias/specialization wire
names. A generated 59-name by nine-dialect applicability matrix keeps
unreviewed ranges explicit; only the three Autodesk-sourced underlay ranges
are currently classified as applicable or not applicable. None of this
inventory metadata implies semantic support.
The unified entity directory indexes exact canonical, alias, unknown, and
reviewed wrong-section markers in source order across `BLOCKS` and `ENTITIES`,
while retaining exact subclass paths and all untouched raw groups.
A generated common-field registry now exposes 19 Autodesk-backed property
roles with wire type, cardinality, default, subclass/application scope,
coordinate-space classification, version-review state, provenance, and a
separate canonical writer-order ordinal. The writer metadata follows the
usual Autodesk common-code presentation while readers remain completely
order-independent.
A generic common-field evidence directory retains exact source occurrences and
fixed per-entity cardinality cards without decoding or selecting duplicates.
Typed common-field semantics now project all 19 roles through the shared
four-state value model: exact numbers and handles are decoded fail-closed,
exact text remains source-backed, reviewed omitted defaults remain explicit,
and proxy graphics stay an opaque occurrence sequence. Duplicate singletons
are never selected.
The CRUD foundation now includes a typed common-field value encoder. It emits
canonical ASCII groups or dialect-correct Binary groups for exact raw text,
handles, finite binary64, Int16, Int32, and bounded opaque chunks; invalid
wire/value combinations and AC1009-inexpressible group codes fail typed before
any document edit is planned.
Source-bound entity keys now drive explicit replacement planning for an
existing unique common-field singleton. The planner replaces exactly one raw
group through the immutable transaction/inverse kernel; absent fields require
a later insertion-anchor operation, duplicates are never selected, and opaque
sequences require specialized edits.
Optional singleton reset planning can also delete one exact unique raw group
to restore its generated implicit default. Required fields, duplicates,
sequences, and nested extension-dictionary members fail typed. Canonical
common-field insertion-anchor planning now locates one exact between-group
byte offset for an absent singleton. It respects preamble/application-group
envelopes, exact `AcDbEntity` scope, writer order, unknown groups, and legacy
AC1009 records without editing the source. Encoding the insertion and complete
CRUD remain later checkpoints.
SPLINE now exposes an analytic-readiness projection that composes exact knots,
weighted WCS control/fit points, degree, declared counts, knot order and
multiplicity, active parameter domain, flags, optional tangents, and planar
normal requirements. Failures accumulate as typed issues and never publish a
partial curve; evaluation and tessellation remain out of scope.
HELIX now has a source-anchored subclass evidence directory for all 16 public
`AcDbHelix` roles. It preserves duplicates and invalid numbers without
selection and prevents colliding SPLINE, application-group, unknown-subclass,
or wrong-section values from entering HELIX evidence.
HELIX cardinality publishes exactly 16 stable cards per record with absent,
unique, or multiple states and exact member-to-value links. Invalid numeric
syntax remains a unique occurrence rather than being mistaken for absence.
Seven HELIX scalar roles now use the shared provenance-bearing semantic value
model. Public handedness and constraint domains are typed, non-finite doubles
fail closed, and undocumented positivity/default policies are not inferred.
HELIX axis base, start point, and axis vector now expose three exact WCS
component semantics each. A tuple is available only when all three components
are usable; missing Y/Z is not silently defaulted.
HELIX relation semantics now normalize a usable nonzero axis, retain the exact
orthogonality residual, derive the base radius from axis-base/start geometry,
classify the stored radius and turns domains, and derive axial height from
turns and turn height. Zero-height flat helices remain observable.
HELIX analytic readiness now composes that metadata with the exact embedded
`AcDbSpline` projection on the same raw record. It requires one ordered
`AcDbSpline`/`AcDbHelix` subclass pair, a valid embedded curve, perpendicular
axis geometry, nonnegative radius, positive turns, and finite derived height;
failures accumulate without publishing partial analytic data. Evaluation and
tessellation remain out of scope.
Budget-aware development runs required quality/dependency gates locally and
does not allocate hosted runners for pushes or pull requests. GitHub Actions
retains a manual Windows x64 self-hosted diagnostic and the manual six-native
hosted release-artifact workflow only.
The remaining documented entity families are tracked in
`docs/DXF_ENTITY_COMPLETION_PLAN.md`; raw preservation does not imply complete
typed semantics for every entity.

## Workspace

- `seacad-dxf-core`: bounded lossless ASCII/Binary framing, source identity,
  reviewed lazy semantics/topology/geometry, reversible transaction planning,
  and verified preserve-patch/canonical create-new writers.
- `seacad-cli`: operational `inspect` and `verify` commands with English or
  Vietnamese human output sourced from per-locale compile-time catalogs and
  stable JSON v1, plus the separate aggregate-only `seacad-corpus-receipt`
  offline evidence harness.
- `seacad-schema-gen`: deterministic provenance-backed schema generation,
  CycloneDX inventory, legal-bundle generation, cross-platform freshness
  verification, fail-closed native artifact assembly, and six-target receipt
  verification.

The previous CAD workspaces under `D:\Backups` are immutable research inputs.
Production code is not copied from them. Tests, fixtures, and knowledge may be
transferred only after the M1 provenance audit.

## Build

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

The repository pins Rust 1.97.1 with rustfmt, Clippy, and LLVM tools.

## Current CLI

```text
seacad inspect drawing.dxf
seacad verify drawing.dxf
seacad inspect drawing.dxf --lang vi
seacad verify drawing.dxf --json
```

English is the default. `--lang vi` localizes human help and reports. JSON
keys, statuses, and codes remain stable English identifiers. Paths are hidden
unless `--show-path` is explicit. See `docs/CLI_JSON_V1.md` for the complete
contract and `docs/SUPPORT_MATRIX.md` for the exact support boundary.

Private corpus bytes stay outside the repository. The Q2.2 harness reads the
public aggregate-only policy in `corpus/offline-manifest.json` and emits no
paths, filenames, source IDs, or per-file hashes. See
`docs/Q2_2_OFFLINE_CORPUS_RECEIPT_CONTRACT.md`.
M13 release qualification uses the separate threshold policy in
`corpus/release-manifest.json`; policy presence is not evidence that a private
corpus has passed.

## License

Copyright (c) 2026 SeaCad. All rights reserved. See `LICENSE`.
