# M14.3ck POINT Delete Graph Scope

Retrieved: 2026-08-03

## Scope

M14.3ck hardens standalone canonical POINT deletion against record-local graph
attachments. The delete path now refuses to remove a raw POINT record that
contains any group-102 application control or group-360 hard-owner occurrence,
preventing the current single-record transaction from orphaning graph payload.

## Contract

- The complete selected POINT record is scanned before handle identity,
  incoming-reference, transaction, or queue admission.
- Any group 102 rejects the operation, including persistent-reactor,
  extension-dictionary, custom, malformed, interrupted, or unmatched
  application controls.
- Any group 360 rejects the operation even outside a recognized application
  group, because the standalone delete cannot prove ownership-graph closure.
- `DxfEntityDeleteIssue::AttachedGraphScope` reports the selected entity key,
  exact raw group occurrence, and group code.
- Cancellation remains checked during the record scan, and a rejection leaves
  every accepted operation and the selected record unchanged.

## Verification boundary

The negative matrix covers AC1012 through AC1032 in ASCII and Binary. Each
dialect rejects a POINT carrying an `ACAD_REACTORS` group with a 330 reference,
an `ACAD_XDICTIONARY` group with a 360 reference, and an unscoped group 360.
All 48 cases assert the typed source-bound rejection and an empty session.
Existing deletion, update, clone, and inverse tests remain active.

## Nonclaims

This checkpoint prevents creation of new orphans; it does not delete, clone,
remap, or verify an attached object graph. It does not preserve application
groups or XDATA during clone, implement cross-document graph remapping, or
advance POINT to `Complete`.

## Verification

The focused POINT edit suite passed 44/44 tests. The final workspace run passed
968/968 listed tests, including all 17 locale tests. Its first non-quiet attempt
hit the command's 120-second wrapper timeout after only passing output; the
complete quiet rerun exited successfully in 109.2 seconds. Generated-schema and
release-evidence checks, `cargo deny --locked check`, formatting, workspace
Clippy with warnings denied, the production forbidden-construct scan, and `git
diff --check` all passed. Production changes are 23 insertions and one deletion;
focused test changes are 117 insertions. No manifest, lockfile, dependency,
committed fixture, generated schema, locale source, or external corpus changed.
This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 396 | `68c3fa5fad6bb9d0be2c2bb7528c4ec59d6239cdc60739998c9e127c7612ce0b` |
| `crates/seacad-dxf-core/src/entity_edit_session.rs` | 2,827 | `67588c3bf08bfcee4c88e0d612544a28c57a1df3aafa94a6250251d22da0764d` |
| `crates/seacad-dxf-core/tests/point_edit_session_tests.rs` | 3,450 | `31899187b1308e22766042d8fcd44505372c2778b7991e0fb707d21d569ab097` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,324 | `cc8dd69038c6676423b0e211c008850ef875d9372abf387271874062e587db55` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,749 | `c3976bd6861114737390e254f7fbb10bca7513878c0e8878ddab6b0a29aeab13` |
| `docs/SUPPORT_MATRIX.md` | 2,398 | `b27500730c50e6d48269ef825cbfb29bc7adce42fa126ee5e9b37a35ccdc22d5` |
