# M14.3cg POINT Linetype Clone

Retrieved: 2026-08-03

## Scope

M14.3cg extends canonical semantic POINT clone with the exact common linetype
symbol in group 6. The source value must resolve uniquely in the same
document's LTYPE table, then remains exact through typed draft encoding,
strict reparse, semantic verification, and inverse generation.

## Contract

- The POINT draft accepts an optional borrowed exact linetype name and the
  record expectation owns the bytes needed for deferred verification.
- Canonical encoding emits group 6 in common-field order and validates the
  exact name against one same-document LTYPE table entry.
- Clone preparation accepts group 6 only in the reviewed common-field scope;
  a colliding code in POINT-family or application scope remains unsupported.
- Missing, ambiguous, wrong-scope, or otherwise unsupported names fail typed
  without queueing an insertion.
- Post-image verification requires the explicit linetype semantic value before
  a journal or inverse can escape.

## Verification boundary

The fixture contains a unique DASHED LTYPE table record and one canonical POINT
with explicit group 6. All nine supported versions in ASCII and Binary clone to
a fresh handle, retain exact DASHED bytes, satisfy insertion verification, and
produce an exact inverse. Existing minimal, scalar-common, POINT-payload,
scope-decoy, invalid-domain, owner/placement, and mixed-session tests remain
active.

## Nonclaims

This checkpoint does not clone material, plot style, color-book name, proxy
graphics, XDATA, extension dictionaries, reactors, ownership/reference graph
closure, cross-container targets, or cross-document data. POINT is not yet
`Complete`.

## Verification

The focused entity insert session suite passed 12/12 tests and the POINT edit
suite passed 43/43 tests. The full workspace passed 961/961 tests, including all
17 locale tests. Generated-schema and release-evidence checks, `cargo deny
check`, formatting, workspace Clippy with warnings denied, the production
forbidden-construct scan, and `git diff --check` all passed. Production changes
are 68 insertions and no deletions; focused test changes are 96 insertions and
no deletions. No manifest, lockfile, dependency, committed fixture, generated
schema, locale source, or external corpus changed. This audit intentionally
omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 379 | `b91038698bc96b6bc55c3c32f31b8b068376328c044b100e0d169ba61d8cc2e6` |
| `crates/seacad-dxf-core/src/entity_draft_record.rs` | 1,041 | `f5c1962e224c7f4f6a463ca81751517e9f8f4f04c7a6472e72ec8aa84fea08d4` |
| `crates/seacad-dxf-core/src/entity_edit_session.rs` | 2,720 | `6db5bb65f1c966c9ff6841be7641ead7353efda71bfbf85fbb7789a112cee289` |
| `crates/seacad-dxf-core/src/entity_edit_verification.rs` | 1,288 | `c7f507db9ebaaaa04ad58672c6f8d50fa5b3aa1610a7f2efe65eeff63be46f72` |
| `crates/seacad-dxf-core/tests/entity_insert_session_tests.rs` | 1,370 | `fd05a4e70beb761e706c78c42ff85e6f13d1cea3590001fb8e5d7728dbc75302` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,289 | `bba744ae0715437547a3c6987b92dd2673107a61c1256182c599d690f472f1c6` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,716 | `0d950df2889132ebdbe1d5d27c61d019ab7e54ca9431cf36f2d18fd60e9c81b0` |
| `docs/SUPPORT_MATRIX.md` | 2,366 | `7bfb9eef296fecbcf7fadcb5e7244a1e3d8a1db1f4111105728e8fe78379585a` |
