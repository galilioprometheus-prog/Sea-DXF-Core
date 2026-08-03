# M14.3bz Mixed Entity Insert/Update Session

Retrieved: 2026-08-03

## Scope

M14.3bz allows a unified edit session to contain existing-entity common/POINT
updates and one or more typed POINT insertions in either API order. It does not
clone or delete entity graphs, add another draft family, render, or advance
POINT to `Complete`.

## Prior evidence

- M14.3ae through M14.3ag establish source-bound common-field session batching,
  semantic postconditions, verified write, and exact inverse restoration.
- M14.3aw establishes atomic composition of independently planned source-bound
  transactions and rejects unresolved overlap or insertion conflicts.
- M14.3ay through M14.3bb establish placement, owner binding, handle
  reservation, dialect admission, and prepared draft identity.
- M14.3bk through M14.3by establish the complete public POINT draft payload,
  multi-insert batching, and every current POINT-family set/reset operation.

The clean-room boundary remains recorded in
`docs/audits/M14_3BK_POINT_DRAFT_RECORD.md`. No legacy or external parser code,
fixture, data, dependency, or generated artifact is copied, translated,
vendored, linked, or used at runtime.

## Contract

- `DxfEntityEditSession::insert` no longer rejects queued updates, and
  `update` no longer rejects queued insertions. Existing typed admission rules
  still run before any operation enters the session.
- One combined logical-operation limit covers common updates, POINT updates,
  and POINT insertions. Receipts and `queued_edit_count` report the complete
  mixed queue.
- Finalization independently freezes existing-record updates and the insertion
  batch, then rebuilds them as one source-bound transaction. It retains handle
  reservation, successor `$HANDSEED`, new records, common-field patches, and
  POINT-family patches atomically.
- Empty-span fragments at one source offset are coalesced deterministically.
  Existing-record update fragments precede new entity-record fragments.
- Existing-record postconditions are ordinal-independent with respect to the
  queued insertions: each expected raw-record ordinal adds the exact count of
  inserted records whose placement precedes that entity's source marker.
- Strict semantic verification covers both existing and inserted entities
  before an executable inverse is released. The inverse restores every source
  byte exactly.

## Verification boundary

Both insert-first and update-first sessions are tested for ASCII and Binary
AC1009 through AC1032. Each session combines a POINT angle update, a generated
common visibility update, handle reservation, `$HANDSEED`, and a complete
POINT insertion. A separate all-dialect fixture inserts into an earlier
ENTITIES section and updates a POINT in a later section, proving that both
POINT-family and common-field postconditions use adjusted raw-record ordinals.
Focused regression coverage also retains multi-insert behavior, admission
failures, transaction composition, verified create-new write, tamper failure,
cancellation, and byte-identical inverse restoration.

## Nonclaims

This checkpoint does not clone or delete an entity graph, infer graph closure,
add non-POINT draft encoders, render, or advance POINT to `Complete`.

## Verification

The mixed insert suite passed 6/6 tests and the POINT edit suite passed 35/35.
The full workspace passed 947/947 tests, including all 17 locale tests.
Generated-schema and release-evidence checks, `cargo deny --locked check`,
formatting, workspace Clippy with warnings denied, the production forbidden-
construct scan, and `git diff --check` all passed. Production changes are 149
insertions and 64 deletions; focused test changes are 285 insertions and 31
deletions. No manifest, lockfile, dependency, committed fixture, generated
schema, locale source, or external corpus changed. This audit intentionally
omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 342 | `1c0eada21ee770a3418fec334e52b0a08096c8d70ab36f85528f41801f38c1fb` |
| `crates/seacad-dxf-core/src/entity_edit_session.rs` | 1,705 | `c3666ede8ef73297ba53a448a51cd8da3f07cc89e6cf0dfd3d1e65f0a3fbadeb` |
| `crates/seacad-dxf-core/tests/entity_insert_session_tests.rs` | 822 | `181b363ce28728a82e4d80966b411c0d3c7d2728dc94c304ccd06ff9a3bb40fd` |
| `crates/seacad-dxf-core/tests/point_edit_session_tests.rs` | 2,737 | `1450edb421f1080d5a449c5edf15ee3e0670618a3bdb1bf0ee0052c90812c6d3` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,207 | `ec8f93a4dc399f4917f699cd5e023c9552fb249c6679b146e3fbbfb3f88dc9cf` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,637 | `9e30887a697759ecced66cf39e670e4f2d3e31b7a34b1a42e1a5c9d830cdd60c` |
| `docs/SUPPORT_MATRIX.md` | 2,288 | `9fd3f2fe0fdf33867302ee63f021cec60dd597a232960ae964f436c778039bfc` |
