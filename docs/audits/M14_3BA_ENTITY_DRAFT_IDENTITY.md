# M14.3ba Typed Entity Draft Identity

## Scope

M14.3ba prepares the source-bound identity of one future entity draft by
combining an exact generated entity name, one placement-owner binding, and
exactly one reserved handle. It does not encode or insert an entity record.

## Evidence boundary

- The generated M14.3f/M14.3g registry remains the sole inventory for the 45
  reviewed canonical topics and 14 reviewed aliases.
- M14.3ay remains the reviewed source of single-range handle reservation and
  the reversible `$HANDSEED` transaction.
- M14.3az remains the reviewed source of placement and caller-selected,
  uniquely resolved BLOCK_RECORD owner binding.
- The existing transaction kernel remains the reviewed source of strict source
  preconditions and byte-identical inverse materialization.
- Autodesk's [Entity Group Codes inventory](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-7D07C886-FD1D-4A0C-A7AB-B4D21F18E484.htm)
  and [object/entity rules](https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-A35B8C2A-1885-4A8E-8533-E61D8A423D62.htm)
  remain the normative inventory and placement boundary. This checkpoint adds
  no new wire-field claim.

The user-authorized legacy trees under `D:\SeaCad\tham khảo\New folder` were
searched read-only. They expose read-only/topology observations but no typed,
editable draft-identity primitive. No external or legacy code, fixture, data,
or dependency was copied, translated, vendored, or linked.

## Contract

- `DxfEntityDraftName` admits only generated canonical topics or generated
  aliases. Unknown/custom names cannot enter this typed preparation path.
- The exact group-zero spelling is retained separately from the alias's
  canonical topic mapping.
- `prepare_entity_draft_identity` requires the owner binding, reservation, and
  raw document view to share one exact source identity.
- The reservation transaction must still satisfy its source precondition.
- Exactly one handle must be reserved. Zero or multiple handles return the
  typed `ReservationCardinality` issue; no occurrence or handle is selected by
  guesswork.
- A successful plan retains source identity, exact generated name, canonical
  mapping, placement, owner handle, admitted BLOCK_RECORD, allocation, and the
  reversible `$HANDSEED` transaction.
- Preparation performs no draft-byte allocation, mutation, or write. ASCII,
  Binary, and format-neutral raw document views expose the same behavior.
- Cancellation is checked before validation and again before a successful plan
  escapes.

## Dialect boundary

All 59 registered names are prepared against paired strict ASCII/Binary
fixtures for all nine supported dialect declarations. This proves registry,
source binding, reservation, and transaction parity only. It does not claim
that every entity name or future owner group is applicable to every dialect.
Applicability remains a later schema/draft-encoding gate.

## Nonclaims

This checkpoint does not encode group 0, handle 5, owner 330, subclass markers,
or family fields; insert a record; validate family payloads; update, clone, or
delete an entity; infer an owner; add unknown/custom draft names; or advance any
entity topic to `Complete`.

## Verification

Focused tests cover all 45 canonical topics and 14 aliases, all 18
dialect/format pairs, exact name and canonical mapping, zero and multi-handle
reservations, source mismatches, cancellation, strict `$HANDSEED` reparse, and
byte-identical inverse materialization. The focused suite passed 4/4 tests and
the full workspace passed 894/894 tests. Generated schema and release-evidence
checks, `cargo deny --locked check`, formatting, workspace Clippy with warnings
denied, workspace tests, the forbidden-production-macro scan, and
`git diff --check` passed. Production adds 230 lines: 226 in the new module and
four module/export lines, within the usual 200-500 line checkpoint target. No
production dependency changed. This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 252 | `2ce3915bd439f018aa43d76f4d1a9befd282808588bfa0f1b4e40b7d06a223cd` |
| `crates/seacad-dxf-core/src/entity_draft_identity.rs` | 226 | `3a9dfb58d3f1c35570e0e1cfc469c5376bab705f70601bf3486d85b7c587d3d5` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,054 | `25c700c6cffaec42afa1ee87c98346a53e1180f1ad45795a9e439a640974919e` |
| `crates/seacad-dxf-core/tests/entity_draft_identity_tests.rs` | 387 | `1a2e5df97e3ee2490a5c63b1b9a89e443df80856e30b5a7778444320424a0b7c` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 913 | `f9fdba47a2d9a357ac2a6ad267c1e2c7266b01515e27b56ce66612c869ece1ba` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,341 | `2e1a974e2ad19edaa56858834f9f3375037fb81463e837d2f435d160ff2e739a` |
| `docs/SUPPORT_MATRIX.md` | 1,995 | `a535eadb035c9398b1429960e0cdc73ca1e31cf82e1181dc2dc6cfe10b0589a3` |
