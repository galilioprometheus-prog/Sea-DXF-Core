# M14.3bl POINT Insert Verification

## Scope

M14.3bl converts one prepared M14.3bk POINT record into an atomic insertion
transaction with family semantic postconditions. It deliberately leaves the
public `DxfEntityEditSession::insert` method, multi-record allocation, and the
remaining POINT CRUD ladder to later checkpoints.

## Normative evidence

- Autodesk [Entity Group Codes in DXF Files](https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-995ABB55-571A-4D0F-882E-8A74A738643E.htm)
  shows the graphical-entity envelope with type, handle, owner, `AcDbEntity`,
  layer, family subclass, and family data inside `ENTITIES`.
- Autodesk [Common Group Codes for Entities](https://help.autodesk.com/cloudhelp/2017/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm)
  identifies group 330 as the non-omitted owner BLOCK_RECORD pointer and lists
  the common layout, layer, and lineweight fields. Its order-independence rule
  still governs reading; this milestone only preserves the reviewed canonical
  new-record order from M14.3bk.
- Autodesk [BLOCKS Section Group Codes](https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-9DDFC343-6C87-4AF8-B3B9-77E0AB5A3031.htm)
  states that member entities live between the matching BLOCK and ENDBLK and
  use the same entity format as records in `ENTITIES`.
- Autodesk [Entity Ownership](https://help.autodesk.com/cloudhelp/2021/ENU/OARX-DevGuide/files/GUID-A0FDAE4E-9DAE-4256-8CE0-E3BD8D81A058.htm)
  states that database entities normally belong to an `AcDbBlockTableRecord`.
- The POINT field evidence and the read-only legacy fixture receipt remain the
  exact M14.3bk evidence recorded in
  `docs/audits/M14_3BK_POINT_DRAFT_RECORD.md`. No external or legacy code,
  fixture, data, dependency, or generated artifact was copied or translated.

## Contract

- `plan_entity_draft_insert` consumes one source-bound
  `DxfEntityDraftRecordPlan`. The source identity, source envelope, retained
  `$HANDSEED` transaction, placement, handle, owner, applicability, and
  cancellation preconditions must all remain valid.
- The planner creates one zero-width insertion patch at the previously admitted
  anchor and composes it with the reservation transaction. Composition reuses
  the existing conflict, resource-limit, source-order, and inverse machinery.
- The result is a standard `DxfEntityEditPlan` with one logical insertion
  expectation. No second writer or verification pipeline is introduced.
- POINT verification first requires the allocated handle to identify exactly
  one record. That record must be a canonical POINT inside the exact requested
  ENTITIES structure section or exact BLOCK member list.
- R13+ owner, exact layer, applicable layout, applicable lineweight, and the
  three-component WCS point are compared as typed semantic values. AC1009 does
  not invent an owner group that its Binary dialect cannot represent.
- Create-new writing retains its established guarantees: strict reparse,
  semantic verification before success, cleanup on failure, exact raw
  transaction verification, and an executable byte-identical inverse journal.
- Payload-bearing debug output remains redacted. The immutable source document
  is never changed.

## Verification boundary

Direct insertion and semantic/inverse verification cover ASCII and Binary for
all nine Core dialects. The create-new write/reparse/verify/journal path repeats
that full 18-pair matrix. BLOCK membership and inverse verification cover every
R13+ dialect in both physical formats, where the reviewed owner envelope is
representable.

Same-length post-image tampering proves typed rejection of a missing allocated
handle, a duplicated handle, a non-POINT marker, an altered layer, and an
altered WCS coordinate. Source mismatch and cancellation are rejected before a
plan escapes. Existing verifier/write regression suites prove no change to
singleton edits, cleanup behavior, transaction composition, or inverse
materialization.

## Nonclaims

This checkpoint does not expose `DxfEntityEditSession::insert`, combine
multiple new records under one reservation, mix insert and update expectations,
derive an owner from layout, add thickness/extrusion/angle, or implement POINT
update, reset, clone, delete, and closed-set reference policy. POINT therefore
remains below `Complete`.

## Verification

The focused draft/insert suite passed 6/6 tests; the edit-verification, edit-
write, and transaction-composition regression set joined it for 19/19 tests.
The complete workspace passed 904/904 tests. Schema and release-evidence
checks, `cargo deny --locked check`, formatting, Clippy with warnings denied,
the production forbidden-pattern scan, and `git diff --check` all passed. The
handwritten production diff is 381 additions and 14 removals; focused
integration coverage changed by 208 additions and 41 removals. No production
dependency changed. This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 272 | `10f4968edb108f1dfe15354ba74609bd92e82b63ed838c564893a4eca40a739b` |
| `crates/seacad-dxf-core/src/entity_draft_insert.rs` | 85 | `6cdef79bf7c01affafad8fe6e6c6f53ddd3060a53a27f2a09c353600d0c47928` |
| `crates/seacad-dxf-core/src/entity_draft_record.rs` | 559 | `c3f07a0a94efb3411be04ac7717a80d2083531f3a33b19537d44ef76ecbf1c6b` |
| `crates/seacad-dxf-core/src/entity_edit_session.rs` | 863 | `a8359d29e0a903ad8f30a265470caab226be4a790724ab010083c896db7ce764` |
| `crates/seacad-dxf-core/src/entity_edit_verification.rs` | 864 | `ae1b0338c752b3c5acb421085843dcd9d5cce1fba99abd105398ceb24cd200ae` |
| `crates/seacad-dxf-core/src/lib.rs` | 1063 | `4988509b334b779ab2f93e463333775a75c6620d59195c4b29fef103618b6408` |
| `crates/seacad-dxf-core/tests/entity_draft_record_tests.rs` | 1083 | `d8cb34676a5fb542dc28f21e3bfd3412f8c72b90561da67849a9759089700832` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1028 | `908b68c524776613dfb90cd96f41f1fae490a40edad3980c808a420974d32e2b` |
| `docs/IMPLEMENTATION_PLAN.md` | 2459 | `a4a6f58ebb35f016baeb6e2c12ef1d4ee4a46f5059c420ee3c2c87d046e41315` |
| `docs/SUPPORT_MATRIX.md` | 2109 | `2aa8553bb359a81479f72804a3ccd58649936e4a28b8860f9d0e2775584fa2ce` |
