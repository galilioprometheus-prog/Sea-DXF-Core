# M11.2b Atomic Handle Assignment Plan Audit

## Outcome

M11.2b converts a ready M11.2a allocation policy and caller-ordered raw-record
targets into one immutable M11.1a transaction plan. The plan inserts every new
object identity and replaces `$HANDSEED` with the successor seed; materialized
bytes re-open with exact assigned identities and a ready allocation policy.

## Normative evidence

- Autodesk's
  [entity example](https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-995ABB55-571A-4D0F-882E-8A74A738643E.htm)
  places group 5 handle evidence after the group 0 entity type.
- Autodesk lists group 5 as the handle for
  [common nongraphical objects](https://help.autodesk.com/cloudhelp/2025/ENU/AutoCAD-DXF/files/GUID-6939D69E-04CB-4F4C-87B2-67BC540FCF58.htm).
- Autodesk documents group 5 for symbol-table entries except group 105 for
  [DIMSTYLE only](https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-5926A569-3E40-4ED2-AE06-6ACCE0EFC813.htm).
- Autodesk says symbol-table object/entry handles are positioned after group 2
  and identifies DIMSTYLE as the sole group-105 exception in that context in
  [Symbol Table Group Codes](https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-5AB9300F-F0AC-4ADE-89EA-A9D1D152D8B8.htm).
- Autodesk specifies Binary DXF group codes as one byte before R13 and
  two-byte little-endian from R13, with
  [NUL-terminated ASCII strings](https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-FC1C3C69-DBC2-49E4-893A-000D6538C0FE.htm).

## Contract

- The M11.2a policy and raw document must have the same `DxfSourceId`.
- Policy unavailability and numeric exhaustion remain their existing typed
  outcomes.
- Caller order determines allocation order; duplicate target ordinals fail
  before planning.
- Every target must exist and have `DxfHandleIdentityState::Absent`.
- CLASSES records, ENDTAB boundaries, and TABLES records without group 2 are
  typed unsupported targets.
- BLOCKS, ENTITIES, and OBJECTS insert uppercase group-5 hexadecimal after the
  group-0 marker.
- TABLES inserts after the first group 2. Exact `DIMSTYLE` entries use group
  105; all other admitted table objects and entries use group 5.
- ASCII reuses the anchor value line's CR, LF, or CRLF ending for both inserted
  lines. Binary uses the declared dialect's group-code width and a NUL
  terminator.
- Nonempty assignments replace the exact `$HANDSEED` payload with the
  successor seed in the same transaction. Empty assignments produce an empty
  source-bound plan and do not rewrite equivalent spelling.
- Any cancellation, resource, source-read, encoding, or transaction-builder
  failure prevents a completed assignment plan from escaping.

## Evidence

- ASCII/Binary plan materialization and strict re-open cover all nine supported
  AC1009-AC1032 dialects.
- Reparse verifies caller-order handle assignment, successor `$HANDSEED`, and
  a ready M11.2a policy.
- CRLF TABLES coverage verifies group-5 table-object and group-105 DIMSTYLE
  placement after group 2 without line-ending normalization.
- Tests distinguish duplicate, missing, already-identified, CLASSES, ENDTAB,
  and missing-table-name targets.
- Tests retain policy-unavailable, cancellation, source-identity mismatch,
  empty-plan, public bounded-type, and debug-redaction behavior.

## Non-claims

M11.2b does not discover which records require identities, assign identities
to CLASSES or incomplete table records, modify existing identities, update
pointer/owner/reactor values, validate reference topology, apply the returned
transaction, calculate its post-image identity, write or replace a filesystem
destination, or publish a snapshot.

## Gates

- `cargo deny --locked check`
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace --quiet` (579 passed, 0 failed, 0 ignored)
- `git diff --check`

All gates passed.

## Artifact receipt

| Artifact | Lines | SHA-256 |
| --- | ---: | --- |
| `crates/seacad-dxf-core/src/handle_assignment_plan.rs` | 438 | `ae14c78f3728ff41d58bb2072faa1f4da1ce2e34c24043b841eb36600f978d8b` |
| `crates/seacad-dxf-core/src/lib.rs` | 620 | `29247eb830f37e900b5a477722ca2387b992fa7cc87bfd59767085af2632113a` |
| `crates/seacad-dxf-core/tests/handle_assignment_plan_tests.rs` | 454 | `896206aa6a496ecade1a5bdab4a9307530b8c82f6c2d30e90c67084bfaf1141a` |
| `docs/IMPLEMENTATION_PLAN.md` | 1185 | `3fa32e6442f422bd62355558b5ce4ab6c0f19e12a365f888b4eebbeeae1e80f6` |
| `docs/SUPPORT_MATRIX.md` | 971 | `61c6c564aa0c504d5bf492bcc56c09f78712255b85b4f582513b2ccc4efd05f5` |
