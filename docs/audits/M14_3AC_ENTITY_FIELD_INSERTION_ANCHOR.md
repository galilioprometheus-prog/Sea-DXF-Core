# M14.3ac Entity Field Insertion Anchor

## Scope

M14.3ac locates one exact source-bound byte position for a future insertion of
an absent common-entity singleton. The position is always between complete raw
groups and retains its immediate group neighbors. This checkpoint does not
encode an inserted group or create a transaction.

## Evidence boundary

The canonical order and scope remain derived from Autodesk's common entity
group-code table:

`https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm`

Autodesk states that readers cannot rely on table order. M14.3ac therefore
uses M14.3ab order only as a writer policy; parsing remains order-independent
and unknown groups remain exact. Exact subclass and application-group evidence
comes from the existing unified platform rather than a new scan.

The user-authorized legacy trees named in the task were searched read-only.
Their fixtures corroborate the observed preamble / `AcDbEntity` / family-
subclass boundary and show extension dictionaries as complete group-102
envelopes. No implementation, code, fixture, or data was copied, translated,
vendored, linked, or added at runtime.

## Contract

- Planning is bound to the raw document, common-field evidence directory,
  `DxfEntityKey`, supported declared dialect, field, and cancellation token.
- Only an absent required or optional singleton can obtain a normal anchor.
  Existing singletons retain their exact occurrence count; proxy group-310
  sequences and extension dictionaries require specialized operations.
- Handle anchors immediately after the exact group-zero entity marker.
- AC1012 and later owner/property anchors require exactly one source-exact
  `AcDbEntity` subclass outside application groups. Owner precedes its marker;
  property anchors stay inside its half-open subclass range.
- AC1009 skips handle/owner and complete leading group-102 envelopes to find a
  bounded legacy property start. An interrupted or unclosed envelope fails.
- For AcDbEntity properties, every lower writer ordinal constrains the target
  after its group and every higher ordinal constrains it before its group. If
  these bounds cross, `ConflictingCommonFieldOrder` retains both occurrences.
- Unknown groups are never reordered. A chosen boundary may be adjacent to an
  unknown group, but it never divides a raw group or application envelope.
- The anchor exposes source identity, key, field, byte offset, and immediate
  preceding/following group occurrences without reading or copying payloads.

## Nonclaims

M14.3ac does not encode an ASCII/Binary value; insert, replace, move, or delete
bytes; create a transaction/inverse; synthesize an extension-dictionary
envelope; choose a sequence operation; allocate handles/owners; combine field
patches; validate property domains/references; write a destination; settle
applicability; or advance any entity to `Complete`.

## Verification

Focused tests cover all nine dialects in strict ASCII and Binary form, exact
byte/group-neighbor anchors, preamble reactors/extension dictionaries, existing
and duplicate singletons, populated sequences, absent nested structures,
missing/duplicate `AcDbEntity`, contradictory existing order, wrong sections,
unavailable dialects, legacy unclosed application groups, BLOCKS unknown
entities, untouched unknown groups, source mismatch, pre-cancellation, and
public trait bounds. The focused suite passes 4/4 and all 813 workspace tests
pass. Schema and release-evidence freshness, `cargo deny --locked check`,
formatting, workspace Clippy with warnings denied, the production forbidden-
macro scan, and `git diff --check` all pass. The production diff is 487 added
lines: 482 in the planner and five module/export lines. This audit intentionally
omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 152 | `18d62cc081c4c5e2e47f3dcbee2fced5bcb2ca42dd1f9568f301228e619c59c0` |
| `crates/seacad-dxf-core/src/lib.rs` | 957 | `b35a13b8b3a7c7da8780e6de79d12f797a0bda45a27f0313d993bc1d745d9552` |
| `crates/seacad-dxf-core/src/entity_field_insertion_anchor.rs` | 482 | `8fbfffd00bb373db76770746a0bb5ed96e5ad3cf03aa50bcd87d8722ab032867` |
| `crates/seacad-dxf-core/tests/entity_field_insertion_anchor_tests.rs` | 584 | `7e047f087e80f27f5f79cf376131b42d8668f6c37050bd0484fe1ef8c193ad63` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 603 | `38b231f2d910f9adfad8e866a3f63040d08d7a93d5e5f7fd9619f7d70a52e681` |
| `docs/IMPLEMENTATION_PLAN.md` | 2032 | `b6a2bdb132229af52a82f36738b657a9222ce880ac4d5ca6887bb39a440b1553` |
| `docs/SUPPORT_MATRIX.md` | 1675 | `b97d109086ace8c6820e82ff1f7b5c398f8843999b4e94e6d475c09ce7e6f364` |
