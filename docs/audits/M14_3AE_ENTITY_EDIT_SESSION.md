# M14.3ae Entity Edit Session

## Scope

M14.3ae introduces the first public source-bound `DxfEntityEditSession` and
typed `DxfEntityPatch` path. It batches common-field singleton replacements,
insertions, resets, and already-implicit reset receipts into one immutable M11
transaction. The source document and evidence remain unchanged.

## Evidence boundary

Common-field cardinality, scope, wire type, default, and writer order remain
backed by the generated registry and Autodesk's common entity group-code table:

`https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm`

M14.3ae adds no new entity-field interpretation. It composes the M14.3z
replacement, M14.3aa reset, M14.3ad insertion, and M11 transaction/inverse
contracts. The user-authorized legacy tree was searched read-only and exposed
only an earlier completion plan, not a reusable edit-session implementation.
No external or legacy implementation, code, fixture, or data was copied,
translated, vendored, linked, or added at runtime.

## Contract

- `DxfEntityPatch` starts with a topic-discriminated `CommonField` variant.
  Its operation is `SetExplicit` with a typed borrowed value or
  `ResetToDefault` with no invented value.
- A session is bound to one raw document, common-field evidence directory,
  resource profile, and shared cancellation token. Source mismatch and
  pre-cancellation prevent construction.
- `update(key, patch)` dispatches explicit set to replacement when the original
  card is unique and to insertion only when the original card is absent.
  Reset uses the existing reset contract and publishes an `AlreadyImplicit`
  receipt without queuing a patch when the optional field is absent.
- Structural and encoder failures retain their original typed issue under an
  insertion, replacement, or reset variant. A second queued edit for the same
  source-bound entity/field fails as `DuplicateFieldEdit`; no occurrence or
  last-writer policy is selected.
- Each admitted operation copies only its encoded replacement bytes into the
  payload-redacted session. Resource limits and cancellation are checked before
  the mutable queue commits the operation.
- `finish` orders edits by exact raw span. Empty insertion spans for the same
  entity and byte offset are ordered by generated writer ordinal and combined
  into one patch; caller order therefore cannot alter canonical field order.
- The final M11 builder independently revalidates source bounds, aggregate
  conflicts, projected size, resource limits, inverse capture, and
  cancellation before publishing one transaction.

## Nonclaims

M14.3ae does not add draft or family-specific patch variants; insert a complete
entity; replace group-310 sequences; edit nested group-102 structures; validate
property domains or references; allocate handles/owners; clone/delete closed
sets; write and verify a create-new destination; settle applicability; or
advance any entity topic to `Complete`.

## Verification

Focused tests cover five-operation batches across two entities for strict ASCII
and Binary AC1009 through AC1032, explicit replacement, exact reset, reverse-
order same-anchor insertion, generated writer ordering, already-implicit
receipts, duplicate queued targets, duplicate source singleton, sequence and
required-field dispatch, wire mismatch, source mismatch, safe-profile value
exhaustion, cancellation before construction and after queueing, source
immutability, semantic post-images, executable byte-identical inverse,
public traits, and debug redaction. The focused suite passes 4/4 and all 821
workspace tests pass. Schema and release-evidence freshness,
`cargo deny --locked check`, formatting, workspace Clippy with warnings denied,
the production forbidden-macro scan, and `git diff --check` all pass. The
production diff is 471 added lines: 466 in the edit session and five
module/export lines. This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 161 | `6bb368ec0ab6c8afc55594b251f78e408f87617f7ad64dcbe866551759d3c79f` |
| `crates/seacad-dxf-core/src/lib.rs` | 966 | `a4f15de3710b9047e2dfc0e82baf1c6c26b8d1b8d19326144e225ced72f35b3b` |
| `crates/seacad-dxf-core/src/entity_edit_session.rs` | 466 | `216a5b4fd86a1a4dbf1e289eacd30b763dec6ed7f1a5bbffb26fa86db5ccf6c4` |
| `crates/seacad-dxf-core/tests/entity_edit_session_tests.rs` | 612 | `aaf9dfdaebb7b9885b9dfe1eb30a3a3298f668f9206efe482a5f8d3eabb575de` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 627 | `1dd7baa53f21f23686d26400eee01b7e32ecae537b46422a3c573ce670b379f5` |
| `docs/IMPLEMENTATION_PLAN.md` | 2057 | `3308933d1a530eab9fec0e45a61d515f3776a44989748a9a048b9b3a94845a95` |
| `docs/SUPPORT_MATRIX.md` | 1705 | `31cad7d873bc739d7d26f14783066f2206432b134d06378e5e4cbdce7fbadae9` |
