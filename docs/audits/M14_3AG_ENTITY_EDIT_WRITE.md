# M14.3ag Entity Edit Write

## Scope

M14.3ag connects `DxfEntityEditPlan` to the existing create-new transaction
writer. A successful call streams one immutable transaction, strictly reparses
the independently opened output, verifies requested common-field semantics and
all raw post-image bytes, and returns both receipts with an executable inverse.

## Evidence boundary

This checkpoint adds no DXF field meaning. It composes the reviewed M12.1a/b
writer, M14.3af semantic verifier, generated common-field registry, and M11.1b
inverse contract. The user-authorized read-only legacy trees under
`D:\SeaCad\tham khảo\New folder` were searched for a comparable transaction
writer; they contained planning notes but no implementation used here. No
external or legacy code, fixture, data, or dependency was copied, translated,
vendored, linked, or added at runtime.

## Contract

- `write_reparse_verify_and_journal_to_new_file` requires the session's exact
  source view and a destination that does not exist.
- M12 streams, hashes, flushes, syncs, reopens, and verifies the complete raw
  output before the entity layer opens that same path with strict read options
  for the transaction's ASCII or Binary format.
- Semantic postconditions run before exact raw comparison. A changed edited
  field therefore returns `Unavailable` with its typed issue; an unrelated raw
  mutation remains a transaction post-image mismatch.
- Successful verification cross-checks write and semantic source/output
  identities. The journal exposes both receipts and the executable inverse;
  duplicated identities are reconstructed only after that validation.
- Strict parse, semantic, raw, cancellation, or identity failure after creation
  removes the destination. Semantic unavailability also removes it before the
  typed outcome escapes. Cleanup failure replaces the primary result, matching
  the existing writer policy. A pre-existing destination remains untouched.

## Nonclaims

M14.3ag does not replace paths or publish snapshots; add crash-atomic rename;
validate common-property domains or references; edit sequences, nested grammar,
or topic-family fields; insert entities; allocate handles/owners; clone/delete
closed sets; settle applicability; or advance any entity topic to `Complete`.

## Verification

Focused tests cover successful create-new write, strict ASCII/Binary reparse,
semantic receipts, source immutability, and byte-identical inverse restoration
for AC1009 through AC1032. Post-hash tampering separately covers strict parse
failure, typed edited-field mismatch, and unrelated raw mismatch, each with
destination cleanup. Source mismatch, pre-existing destination, mid-write
cancellation, public traits, debug redaction, and existing-file preservation are
covered. The focused suite passed 5/5 tests and the full workspace passed
830/830 tests. Schema generation and release-evidence checks, `cargo deny
--locked check`, formatting, workspace Clippy with warnings denied, workspace
tests, and `git diff --check` all passed. The production diff is 204 added and
9 removed lines: 193 added and 5 removed in entity edit verification, 10 added
and 3 removed in transaction writing, and one export-line replacement. This
audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 168 | `484921a84ce1cfea035a65780dc028cd0ecea75a2cfff34132cbec3f8c364327` |
| `crates/seacad-dxf-core/src/entity_edit_verification.rs` | 613 | `58f267ab957bf1904c542daa2113d013dcb09305940073c72f96c9cc9650998b` |
| `crates/seacad-dxf-core/src/lib.rs` | 972 | `502c37cce92edf58366ab782466908709049c08bdd521749081929a218bf9343` |
| `crates/seacad-dxf-core/src/transaction_write.rs` | 500 | `6fced108d511eecd4588e8fbed49dadbd19e69568bc8a43cef8d562e34f5189b` |
| `crates/seacad-dxf-core/tests/entity_edit_write_tests.rs` | 518 | `0ce72ea990208a47c689b7f1c722190fe034cfe18b717a4b9688d09ab14aefe1` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 655 | `351c3d3f5902094d5534f118ef0a071f354dd1eeff8728d17f99fea7c3830d18` |
| `docs/IMPLEMENTATION_PLAN.md` | 2084 | `5f2e72f2f452ca589426cf34966a8792367ae44b6eede15f76ae77164a613685` |
| `docs/SUPPORT_MATRIX.md` | 1737 | `6d2cc190f910f179a013b47149c80255586dd91c9dcc71372c947d9031300ba8` |
