# M14.3bm POINT Session Insert

## Scope

M14.3bm exposes the first whole-entity insertion through the unified
`DxfEntityEditSession`. It implements the roadmap signature
`insert(DxfEntityPlacement, DxfEntityDraft)` for one canonical POINT while
leaving multi-record and mixed-operation sessions fail-closed.

## Evidence

The wire, ownership, placement, applicability, POINT field, and semantic
evidence is unchanged from the receipts below:

- `docs/audits/M14_3BK_POINT_DRAFT_RECORD.md` records the Autodesk POINT and
  common-envelope evidence plus the read-only legacy behavioral oracle.
- `docs/audits/M14_3BL_POINT_INSERT_VERIFICATION.md` records Autodesk entity
  ownership and ENTITIES/BLOCK membership evidence and proves the atomic
  insertion/write/reparse/inverse pipeline.

This milestone is an API and orchestration consolidation over those reviewed
contracts. No external or legacy code, fixture, data, dependency, or generated
artifact was copied, translated, vendored, linked, or used at runtime.

## Contract

- `DxfEntityDraft` can retain one exact caller-selected BLOCK_RECORD owner.
  Low-level record encoding accepts an omitted owner for the prior preparation
  API, but rejects a supplied owner that differs from the admitted binding.
- Session insertion requires an owner. It never derives model or paper space
  from group 67, layout 410, a conventional symbol name, or source order.
- Admission is atomic: validate the exact source-bound placement/owner, build a
  monotonic one-handle reservation, prepare typed identity, apply generated
  version applicability, encode the POINT record, and build the M14.3bl
  verified insertion plan. The session changes only after every step succeeds.
- The receipt publishes only allocated handle, typed draft name, and placement.
  Compact typed issues retain the relevant handles, raw ordinal, counts, and
  states without embedding the large owner-evidence record or payload bytes.
- `queued_edit_count` counts the insertion as one logical edit. `finish`
  returns the composed raw transaction and `finish_verifiable` returns the
  insertion's family semantic expectation.
- This checkpoint permits one insertion per session. A second insertion,
  update after insert, or insert after update is rejected without altering the
  already queued operation. This prevents existing raw-ordinal field
  expectations from being invalidated before the mixed verifier is available.

## Verification boundary

Session insertion, strict reparse, unique handle lookup, POINT semantic
verification, and byte-identical inverse restoration cover all nine dialects
in ASCII and Binary. A separate `finish` test proves the raw transaction path.
Negative coverage proves missing and unresolved owner, duplicate insertion,
both mixed-operation directions, foreign placement, cancellation, record
validation failure, and supplied/admitted owner mismatch. Existing edit-session
and draft-record regression suites remain green.

## Nonclaims

This checkpoint does not admit two inserted records, share one reservation
across multiple drafts, mix insert and update expectations, infer owners, add
POINT thickness/extrusion/angle, or implement POINT update, reset, clone,
delete, and graph closure. POINT remains below `Complete`.

## Verification

The final draft-record, edit-session, and insert-session focused set passed
14/14 tests. The complete workspace passed 908/908 tests. Schema and release-
evidence checks, `cargo deny --locked check`, formatting, Clippy with warnings
denied, the production forbidden-pattern scan, and `git diff --check` all
passed. Clippy initially identified that retaining the full owner-evidence
match made insert outcomes approximately 256 bytes; the public failure was
replaced with the compact allocation-free typed form before the clean run. The
handwritten production diff is 318 additions and 18 removals, with 507 lines of
focused integration additions. No production dependency changed. This audit
intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 280 | `f280b58389a4c3906fdb9db469adbcedd3318d2dbda3ccbabd836e5ae0311a69` |
| `crates/seacad-dxf-core/src/entity_draft_record.rs` | 592 | `b7047c637d513cf1d0153d52021ddab39223b5c8949bb0571b3008408697e016` |
| `crates/seacad-dxf-core/src/entity_edit_session.rs` | 1129 | `9797252cefd2220a6128abe881e673a0d4e959bcebcff9ea876397d0e272d841` |
| `crates/seacad-dxf-core/src/lib.rs` | 1064 | `0b7c48d4e95b64be1a589c84059309156893981cff879a3ded1ad4458886bcda` |
| `crates/seacad-dxf-core/tests/entity_draft_record_tests.rs` | 1102 | `e22466e1967f9ca2bdf9a5de6dc037a464fa3caaeddb288b0c11e5ca245c9d81` |
| `crates/seacad-dxf-core/tests/entity_insert_session_tests.rs` | 488 | `e1eb66b05101354d38fc4599ea8c53f4716c61ecc7104e980efc4658a99868cc` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1043 | `9be145bfcec3e9879006445032a73e585f969ddbb20962863088187e22d0a098` |
| `docs/IMPLEMENTATION_PLAN.md` | 2473 | `f3c3397da51e6dc690124d0ac1c9b262582887b5d345fe957ea2b23661b4bc9e` |
| `docs/SUPPORT_MATRIX.md` | 2124 | `4015783fea857fbc0552d6b0877a7459abe975ee421eeaa152d6974ae25e9302` |
