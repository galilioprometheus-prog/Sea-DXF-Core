# M14.3bn POINT Multi-Insert

## Scope

M14.3bn removes the one-insert limit from `DxfEntityEditSession` for canonical
POINT drafts. One session now owns multiple admitted records, allocates one
consecutive handle range, emits one `$HANDSEED` update, and retains one semantic
expectation per inserted record.

## Evidence

No new DXF wire claim is introduced. The normative POINT, common entity,
ownership, placement, applicability, and verified-writer evidence remains:

- `docs/audits/M14_3BK_POINT_DRAFT_RECORD.md`;
- `docs/audits/M14_3BL_POINT_INSERT_VERIFICATION.md`;
- `docs/audits/M14_3BM_POINT_SESSION_INSERT.md`.

This checkpoint changes batching and ownership of already reviewed data. No
external or legacy code, fixture, data, dependency, or generated artifact was
copied, translated, vendored, linked, or used at runtime.

## Contract

- Insert admission validates the source-bound placement and explicit owner,
  generated name applicability, exact symbol/layout references, common-field
  applicability, numeric domains, and resource limits before queue mutation.
- The current source policy proposes the complete count after each admission.
  The newly admitted record receives the proposal handle at its caller-order
  index. A failed admission leaves the queue and the next handle unchanged.
- The session stores owned canonical record bytes and owned semantic evidence;
  no caller text slice survives the call and no reservation transaction is
  cloned per record.
- `finish` creates one reservation for the final count and verifies every
  proposed handle against it. Any disagreement is an internal fail-closed
  error rather than silent remapping.
- Pending records sort by source anchor and then monotonic handle. Equal-anchor
  bytes are passed as ordered fragments to one transaction patch, preserving
  caller order. Distinct anchors remain source-ordered by the transaction
  builder.
- The insertion transaction is composed with exactly one successor
  `$HANDSEED` transaction. `finish_verifiable` emits one POINT insertion
  expectation per handle, so identity, placement, owner, common fields, and
  geometry remain independently verified.
- Insert/update mixing remains rejected because existing update expectations
  are raw-ordinal based. This checkpoint does not weaken that guard.

## Verification boundary

For every ASCII and Binary Core dialect, one session admits three POINT drafts,
returns handles `0x40`, `0x41`, and `0x42`, reports three logical edits, emits
one combined same-anchor insertion patch plus one seed patch, and reparses with
successor `$HANDSEED` `0x43`. Handle identity raw ordinals prove caller order.
All three family expectations verify and the journal inverse restores the
byte-identical source.

Negative coverage proves that missing/invalid owners, invalid record data,
foreign placement, cancellation, and numeric handle exhaustion queue nothing.
An invalid draft between two successful calls does not consume `0x41`.
Existing single-record raw `finish`, applicability, draft-record, and update-
session regression suites remain green.

## Nonclaims

This checkpoint does not mix insert/update expectations, insert non-POINT
families, share handles across concurrent sessions, provide a cross-session
lock, add POINT thickness/extrusion/angle, or implement POINT update, reset,
clone, delete, and graph closure. POINT remains below `Complete`.

## Verification

The four focused applicability, draft-record, update-session, and insert-
session suites passed 18/18 tests. The full workspace passed 908/908 tests.
Generated-schema and release-evidence checks, `cargo deny --locked check`,
formatting, workspace Clippy with warnings denied, the production forbidden-
macro scan, and `git diff --check` all passed. Production changes are
315 insertions and 143 deletions; the expanded insert-session test changes are
78 insertions and 23 deletions. No manifest, lockfile, production dependency,
fixture, or generated schema changed. This audit intentionally omits its own
hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 286 | `abc8ceb2ab77b437f9ad8eeba5a514c8ccd340f2c11481705b197f404def2b47` |
| `crates/seacad-dxf-core/src/entity_draft_applicability.rs` | 205 | `70759dd4e89144b8a73a46afa29a67df233351768d4f1c3cd99ab2729b8431ba` |
| `crates/seacad-dxf-core/src/entity_draft_record.rs` | 657 | `5e2c6996519433c1641c346cc691c0b95bd0a6e436125f6911c51a529fdb85ee` |
| `crates/seacad-dxf-core/src/entity_edit_session.rs` | 1,219 | `c965a76a75edb31f32a105c0e7f15671b5c456cbb4613877945eec1cb13c0ace` |
| `crates/seacad-dxf-core/tests/entity_insert_session_tests.rs` | 543 | `93f65d7a2427256ba347eddf828ef2558e79001eae87050bf7c8f654c36c146c` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,058 | `dc24da3ddb590f124b208534fd36285ece5236481b468fd3287ecf3a85ead84e` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,485 | `869aebb13687d3bfc0c9f7a6eb1d17905f835a2c1a9f46201044cd1d7379557d` |
| `docs/SUPPORT_MATRIX.md` | 2,137 | `cd64fe397ec121a8e71c4e613890a6820f97fe8b62b97167f685604a2c584222` |
