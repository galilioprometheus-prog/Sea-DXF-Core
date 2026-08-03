# M14.3ch POINT Object-Reference Clone

Retrieved: 2026-08-03

## Scope

M14.3ch extends canonical semantic POINT clone with the reviewed common
material and plot-style references in groups 347 and 390. Each explicit source
handle remains exact only when it resolves uniquely in the same document to the
required object kind, then survives typed draft encoding, strict reparse,
semantic verification, and inverse generation.

## Contract

- The POINT draft accepts optional typed material and plot-style handles; its
  compact expectation preserves presence separately from the handle values.
- Canonical encoding emits group 347 and group 390 in common-field order and
  validates their targets as `MATERIAL` and `ACDBPLACEHOLDER`, respectively.
- Clone preparation accepts each handle only in its reviewed common-field scope
  and rejects null, malformed, duplicate, or wrong-scope evidence without
  queueing an insertion.
- Missing, ambiguous, or incompatible same-document targets fail through the
  existing typed common-reference classifier.
- AC1009 rejects both fields as not applicable instead of attempting to encode
  binary group codes above its supported range.
- Post-image verification requires the exact explicit handles before a journal
  or inverse can escape.

## Verification boundary

The fixture contains a `MATERIAL` object at handle 13 and an
`ACDBPLACEHOLDER` object at handle 14. Every supported AC1012+ version in ASCII
and Binary clones a POINT to a fresh handle, retains both references, satisfies
insertion verification, and produces an exact inverse. A dedicated negative
case proves that a material handle aimed at the plot-style placeholder returns
the typed incompatible-target issue and leaves the edit queue empty. Existing
minimal, scalar-common, linetype, POINT-payload, scope-decoy, invalid-domain,
owner/placement, and mixed-session tests remain active.

## Nonclaims

This checkpoint does not clone color-book name, proxy graphics, XDATA,
extension dictionaries, reactors, ownership/reference graph closure,
cross-container targets, or cross-document data. POINT is not yet `Complete`.

## Verification

The focused entity insert session suite passed 14/14 tests and the POINT edit
suite passed 43/43 tests. The full workspace passed 963/963 tests, including all
17 locale tests. Generated-schema and release-evidence checks, `cargo deny
check`, formatting, workspace Clippy with warnings denied, the production
forbidden-construct scan, and `git diff --check` all passed. Production changes
are 183 insertions and no deletions; focused test changes are 135 insertions and
11 deletions. No manifest, lockfile, dependency, committed fixture, generated
schema, locale source, or external corpus changed. This audit intentionally
omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 381 | `61d3203fdb2f70231b09429f2ba26af3e39cdb570555a48c454f5278fe473052` |
| `crates/seacad-dxf-core/src/entity_draft_record.rs` | 1,180 | `fed6bb4041b892845ce2542cc6b3ba841f698c48f70f92cda595e8b502774bea` |
| `crates/seacad-dxf-core/src/entity_edit_session.rs` | 2,752 | `4305d4ba3ce3af3894be0a3931e394ed552655cff349fcd8adadb28050b01571` |
| `crates/seacad-dxf-core/src/entity_edit_verification.rs` | 1,300 | `caec9943c8af45503f342858e09292c0d84d997f0106485847fe1437ad44a615` |
| `crates/seacad-dxf-core/tests/entity_insert_session_tests.rs` | 1,494 | `2d9ad8adc03b60198ed61778008f875e616b4e0db375132478f6d40c414fafb1` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,297 | `c7b73eeafe964c4d4bb5ae6505f14319e779cf29d5547b6852c8850ed6917845` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,724 | `761c9f8fef18cd44b1258024f34594da152889e7e509d2ef0d6d76a68fa32fbd` |
| `docs/SUPPORT_MATRIX.md` | 2,374 | `10fa5cc39414d2eceefbc670ed8015a93418e288a87218b96d3508282740ddf7` |
