# M14.3z Entity Field Replacement Plan

## Scope

M14.3z introduces stable source-bound entity keys and the first update planner
on the unified entity platform. It replaces one explicitly selected common
field only when M14.3k proves that field is an existing unique non-sequence
singleton. The output is an immutable M11 raw transaction plan; no source or
destination is written.

## Normative evidence

Autodesk's entity rules state that an entity begins with group 0, its groups
must be interpreted by code rather than physical order, and unknown groups must
be tolerated:

`https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-A35B8C2A-1885-4A8E-8533-E61D8A423D62.htm`

Autodesk's common entity-code table defines the common property roles, their
wire domains, omitted defaults, application-group envelope, and proxy-graphics
sequence used by the existing generated schema and evidence cards:

`https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm`

Autodesk's Binary DXF reference supplies the dialect-dependent wire framing
implemented by M14.3y and reused without a second encoder here:

`https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-FC1C3C69-DBC2-49E4-893A-000D6538C0FE.htm`

The user-authorized legacy tree under
`D:\SeaCad\tham khảo\New folder` was searched read-only for entity keys,
common-field update/replacement planners, and transaction-plan equivalents.
No applicable entity-field oracle was found or used. No legacy or external
source, test, fixture, or implementation was copied, translated, vendored,
linked, or used at runtime.

## Contract

- `DxfEntityKey` is obtained from an indexed `DxfEntityRef` and retains exact
  `DxfSourceId` plus raw-record ordinal. Cross-source directory lookup and
  planning fail with `SourceIdentityMismatch`.
- Planning requires a supported declared dialect, an entity admitted by the
  unified `BLOCKS`/`ENTITIES` classification, and its M14.3k common-field card.
- An existing `Unique` required or optional singleton resolves to exactly one
  source occurrence. M14.3y validates and encodes the caller's explicit value.
- The resulting M11 transaction has one patch over that occurrence's exact
  full group span. Replacement bytes are canonical for the document's physical
  format and dialect; every other source byte remains outside the patch.
- `AbsentRequired` and `AbsentOptional` remain distinct typed failures because
  no canonical insertion anchor exists yet. `Duplicate` fails with its exact
  occurrence count and never selects an occurrence.
- Any optional-sequence descriptor, including absent or populated group 310,
  returns `SequenceOperationRequired`; a singleton API cannot replace sequence
  structure.
- Reviewed entity names found in the wrong section fail typed. Missing entity,
  unavailable dialect, wire/value encoding, cancellation, resource, source,
  transaction, and allocation failures prevent a plan from escaping.
- Plan `Debug` delegates to the payload-redacted transaction receipt and does
  not expose the explicit replacement value.

## Verification boundary

For all nine supported dialects AC1009 through AC1032, paired ASCII and Binary
fixtures replace an existing layer value, materialize the post-image, strictly
reparse it, and verify the typed exact-text semantic. M11 inverse-plan
materialization then restores byte-identical source. Additional cases cover
handle, exact text, Int16, binary64, and Int32 replacement bytes; duplicate,
required/optional absence, populated sequence, wrong section, absent dialect,
wire mismatch, cross-source evidence/key, pre-cancellation, entity-key lookup,
public bounds, and debug redaction.

## Nonclaims

M14.3z does not insert an absent field; calculate canonical subclass insertion
anchors; implement `ResetToDefault`; replace sequence/nested grammar; validate
color, visibility, lineweight, transparency, shadow, or reference domains;
resolve ownership; allocate handles; combine multiple field edits; provide
typed family drafts/patches; clone or delete closed sets; run a verified
create-new destination write; expose a complete `DxfEntityEditSession`; mutate
source bytes; settle applicability; or advance any entity to `Complete`.

## Verification

The focused replacement suite passes 4/4 tests and the workspace passes all
805 tests. Schema and release-evidence freshness checks, `cargo deny --locked
check`, format, workspace Clippy with warnings denied, workspace tests, the
production forbidden-macro scan, and `git diff --check` all pass. The
checkpoint production diff is 301 additions and two export-line deletions:
255 lines in the replacement module, 40 key/directory lines, and six
module/export additions. This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 140 | `cad9c9cb6c98507602a5c40f585c3911942c735bd10838f49514ac46eb16f233` |
| `crates/seacad-dxf-core/src/lib.rs` | 948 | `7e110c1b122ec1b50dd764f586d52b133aea0d9be45a80158851959894cacaba` |
| `crates/seacad-dxf-core/src/entity_directory.rs` | 451 | `65fbce4e6919437e9a3c242f2dadb3a27ddceceb2494bbe25a3eb3eaadc64d5c` |
| `crates/seacad-dxf-core/src/entity_field_replacement.rs` | 255 | `5d8f0aab2f64127e78a168b531d966b6f0e231eb1a32a4d707da67020fe97fcd` |
| `crates/seacad-dxf-core/tests/entity_field_replacement_tests.rs` | 624 | `c071358816eea363f3a5457a5489e6649f6b0144c1db79e51a45ade1a6b9520c` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 570 | `c6b559fb46d33ea909664c255d619d406fb1e298b66ee6e9fa4f737b5ad7ba82` |
| `docs/IMPLEMENTATION_PLAN.md` | 1997 | `86f32ff1aa00bbbb333186fd0a4f7986f1bfbf7983954470c397b29f1aa6f377` |
| `docs/SUPPORT_MATRIX.md` | 1638 | `429cb84c335685ff163f657e1d3261b45c419c8972ec8355f1bced60c25fea51` |
