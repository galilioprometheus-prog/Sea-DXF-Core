# M14.3ce Mixed POINT Delete Session

Retrieved: 2026-08-03

## Scope

M14.3ce composes canonical POINT deletion with unrelated existing-record
updates and new POINT insertions in either API order. It preserves per-record
conflict safety, exact post-image ordinals, handle and entity-count deletion
postconditions, and one executable inverse. It does not update or clone a
record selected for deletion, broaden clone payload, or advance POINT to
`Complete`.

## Contract

- Insert requests may coexist with pending deletes because canonical POINT
  drafts cannot introduce an owner or pointer to a POINT delete target.
- An update or clone whose source key is pending deletion fails typed. Deleting
  a key with a pending common or POINT update also fails typed. Unrelated keys
  remain composable.
- Existing-record semantic ordinals add every inserted record placed before the
  source marker and subtract every queued deletion with an earlier raw-record
  ordinal.
- Finalization composes common-field patches, POINT-family patches, handle
  reservation, `$HANDSEED`, inserted record bytes, and exact deletion spans.
- Handle-backed deletes require identity absence. Handleless deletes require
  the exact final source entity count plus inserted entities minus deletions.
- Strict reparse, exact transaction bytes, every semantic postcondition, and
  inverse materialization must succeed before a journal escapes.

## Verification boundary

Delete/update fixtures contain two POINTs and exercise deletion of the earlier
record plus location update of the later record in both API orders across all
18 ASCII/Binary dialect pairs. This proves the delete-induced ordinal
reduction. Delete/insert fixtures exercise a canonical existing POINT and one
fresh POINT in both API orders across the same 18 pairs. Legacy handleless and
modern handle-backed deletion both retain the correct final entity count,
fresh handle identity, `$HANDSEED`, semantic verification, and byte-identical
inverse. Existing same-key conflict and cancellation tests remain active.

## Nonclaims

This checkpoint does not update then delete the same record, clone a deleted
source, admit reference-closure cascades, copy broader common properties, or
claim complete entity editing.

## Verification

The focused POINT edit suite passed 43/43 tests and the entity insert session
suite passed 10/10 tests. The full workspace passed 959/959 tests, including
all 17 locale tests. Generated-schema and release-evidence checks, `cargo deny
check`, formatting, workspace Clippy with warnings denied, the production
forbidden-construct scan, and `git diff --check` all passed. Production changes
are 107 insertions and 75 deletions; focused test changes are 152 insertions and
8 deletions. No manifest, lockfile, dependency, committed fixture, generated
schema, locale source, or external corpus changed. This audit intentionally
omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 369 | `38e1b53d8f116ce3027be6d8370d565263a4275fab45b1f6523e2d3282912cb7` |
| `crates/seacad-dxf-core/src/entity_edit_session.rs` | 2,509 | `5dc98ec4b9ea9ff1e787f98354e4f1ae550e141472825f12cdea1d222e231fa9` |
| `crates/seacad-dxf-core/src/entity_edit_verification.rs` | 1,240 | `78dc656b957db7379761bfe4b4a2512628c7e8e393c234e67f4f1667751c172c` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,068 | `64aca4f5d173a42a8ad0116e2bfe93340012ca49e5531418d8a984a55eea5536` |
| `crates/seacad-dxf-core/src/point_edit.rs` | 1,084 | `bcef8c022f2c17e5bf672a06692a9c28787ef7e56fbc24194c0ab44a33c46e51` |
| `crates/seacad-dxf-core/tests/entity_insert_session_tests.rs` | 1,098 | `0cd76e75355d9eb5a8c1094fbfa0691221ba3458fd6d48ae25521578dca1ccfa` |
| `crates/seacad-dxf-core/tests/point_edit_session_tests.rs` | 3,333 | `07fd1a4af8727054a0defe0c07d19014a69fdad059a51159a3e2a9de6a166d1f` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,270 | `296a4c537cef83eb697677b0eba27742882d56ca21c6d5fb8c27e9a79dde9ff1` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,697 | `6acddea9ad6d5613865c71fcb70118e0c6c9a506c9331ded8fd0bfb73610dfad` |
| `docs/SUPPORT_MATRIX.md` | 2,347 | `4d62ac78bd752c75793cf1ced7dbe412164348f1f479cb2cc5f039719973a842` |
