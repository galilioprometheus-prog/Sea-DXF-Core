# M14.3ca POINT Reference-Safe Delete

Retrieved: 2026-08-03

## Scope

M14.3ca adds the first whole-record delete operation to the unified entity edit
session. It admits one standalone canonical POINT with a unique non-null handle
and no uniquely resolved incoming reference from another record. It does not
admit handleless deletion, operation mixing, multi-delete, clone, or POINT
`Complete`.

## Prior evidence

- M14.3i through M14.3r establish record-local identity, pointer/owner
  classification, document-local handle resolution, and incoming ownership
  evidence.
- M14.3aw establishes source-bound transaction composition and exact inverse
  materialization.
- M14.3bk through M14.3bz establish canonical POINT insertion, full public
  POINT payload update/reset, mixed insert/update sessions, strict semantic
  verification, and byte-identical restoration.

No legacy or external parser code, fixture, data, dependency, generated
artifact, or private corpus content was copied, translated, vendored, linked,
or used at runtime.

## Contract

- `DxfEntityEditSession::delete` accepts only a canonical POINT selected by a
  source-bound `DxfEntityKey`.
- The selected raw record must expose exactly one parsed, non-null handle, and
  document lookup must resolve that handle uniquely back to the same record.
- Every resolved pointer and owner occurrence is checked. A unique incoming
  target from any other raw record rejects deletion with the exact source
  record, group occurrence, and handle class retained in the issue.
- Self-contained references inside the record do not block removal because the
  complete record is deleted atomically.
- Admission removes the exact span from the group-zero marker through the last
  group in the selected raw record. No neighboring record bytes are selected.
- The first delete checkpoint is deliberately standalone. Existing queued
  updates/inserts block delete, and an admitted delete blocks later update or
  insert calls.
- Post-image verification requires the deleted handle to be absent. Exact
  transaction-byte verification runs before the executable inverse is
  released, and the inverse restores the source byte-for-byte.

## Verification boundary

Paired in-memory fixtures exercise ASCII and Binary AC1009 through AC1032.
They prove exact POINT removal while retaining the following LINE, strict
reparse, semantic handle absence, and byte-identical inverse restoration.
Negative coverage includes hard-pointer incoming references, ambiguous and
missing identity, wrong family, update/delete ordering in both directions, and
cancellation. Missing, invalid, null, multiple, or ambiguous identity remains
fail-closed by contract.

## Nonclaims

This checkpoint does not delete handleless records, calculate arbitrary entity
graph closure, cascade ownership graphs, mix or batch deletes, clone entities,
render POINT display behavior, or advance POINT to `Complete`.

## Verification

The focused POINT edit suite passed 38/38 tests. The full workspace passed
950/950 tests, including all 17 locale tests. Generated-schema and release-
evidence checks, `cargo deny --locked check`, formatting, workspace Clippy with
warnings denied, the production forbidden-construct scan, and
`git diff --check` all passed. Production changes are 266 insertions and 14
deletions; focused test changes are 273 insertions and 5 deletions. No manifest,
lockfile, dependency, committed fixture, generated schema, locale source, or
external corpus changed. This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 350 | `0ccd414422ddc9900a579a92e83d38aef2c91ff16274015a4a3dc86567f207ec` |
| `crates/seacad-dxf-core/src/entity_edit_session.rs` | 1,919 | `0dd9c783d2dc6a6f7482c2c81db3680352d326c6d480ecbe328cdab02b6ab769` |
| `crates/seacad-dxf-core/src/entity_edit_verification.rs` | 1,200 | `e924044499ac0f7514ed93b4535f803ff7c2d29efde17882e82f2f7454361bfb` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,067 | `c9aad48b64260382bc5bb606e6b5d75114f7431ece321bd4b29d4f8480c7505a` |
| `crates/seacad-dxf-core/tests/point_edit_session_tests.rs` | 3,005 | `286656da41a43692d73dfe3fe3280f7f7f4ef331b40971941e7746bff6fc3d61` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,220 | `f94dfcdbc0d658aa8f9981998189504c13e16659d2bcd97fe3e8243ec1916082` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,649 | `05ae4be67bae3d79e3ecc73b1054e8b2c1fb0b33edeab58d8a39cae3f99059f5` |
| `docs/SUPPORT_MATRIX.md` | 2,300 | `c54411de609bf1b1ce36c8efce7b304906f3b5eaf3744789e3b59835c4ffe3a8` |
