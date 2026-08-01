# M14.3aa Entity Field Reset to Default

## Scope

M14.3aa adds the second common-field update operation on the unified entity
platform. It resets one optional singleton to its generated implicit semantic
state by deleting exactly one existing unique raw group. The planner returns an
immutable M11 transaction; it never mutates the source or writes a destination.

## Evidence boundary

The reviewed common-field registry remains derived from Autodesk's common
entity-code table:

`https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm`

The table supplies the omitted defaults and identifies group 360 inside the
`ACAD_XDICTIONARY` group-102 envelope plus repeated proxy-graphics group 310.
Entity records remain order-independent and unknown-group tolerant under the
existing Autodesk entity rules:

`https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-A35B8C2A-1885-4A8E-8533-E61D8A423D62.htm`

The user-authorized legacy tree named in the task was used only as a read-only
behavioral search boundary. No applicable reset planner or fixture was found
or used. No legacy or external code, test, fixture, or data was copied,
translated, vendored, linked, or added at runtime.

## Contract

- Planning is source-bound to `DxfEntityKey`, the matching evidence directory,
  one admitted `BLOCKS`/`ENTITIES` record, and a supported declared dialect.
- `AbsentOptional` returns `AlreadyImplicit` with the same key and field and
  creates no transaction.
- `Unique` is eligible only for `OptionalSingleton`. Its exact evidence member
  supplies the raw group's complete source span, which becomes one empty-
  replacement M11 patch with captured inverse bytes.
- `RequiredSingleton` always returns `RequiredField`; it is never deleted.
- `Duplicate` reports the exact count and never selects an occurrence.
- `OptionalSequence`, including an absent sequence, always reports
  `SequenceOperationRequired`; singleton reset cannot infer sequence intent.
- The extension-dictionary singleton reports
  `NestedStructureOperationRequired`. Deleting group 360 alone could leave an
  empty `ACAD_XDICTIONARY` group, so a later nested operation must own the
  complete envelope.
- Wrong-section, unavailable dialect, cross-source, cancellation, resource,
  transaction, and allocation failures prevent a plan from escaping.

## Verification boundary

All nine supported dialects exercise paired ASCII and Binary fixtures. The
linetype reset deletes one exact group, strictly reparses the post-image,
projects the generated `BYLAYER` default, and materializes an inverse that
restores byte-identical source. Exact deletion is also checked independently
for optional exact-text, handle, Int16, binary64, and Int32 wire domains.
Additional cases cover already-implicit absence, required fields, duplicates,
populated sequences, nested extension dictionaries, wrong sections, absent
dialect, cross-source evidence/key, pre-cancellation, and public trait bounds.

## Nonclaims

M14.3aa does not insert an absent field; delete a complete application group;
replace a sequence; calculate canonical subclass insertion anchors; combine
multiple patches in `DxfEntityEditSession`; validate property domains or
references; allocate handles or owners; clone/delete closed sets; run a
verified create-new destination write; settle field applicability; or advance
any entity to `Complete`.

## Verification

The focused reset suite passes 4/4 tests and the workspace passes all 809
tests. Schema and release-evidence freshness checks, `cargo deny --locked
check`, format, workspace Clippy with warnings denied, the production
forbidden-macro scan, and `git diff --check` all pass. The production diff is
252 additions: 248 lines in the reset planner plus four module/export lines.
This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 141 | `67e48ed3158f0c9e6a59265a366f0ee937f6c7cf90f734e15d7bf188a43d720f` |
| `crates/seacad-dxf-core/src/lib.rs` | 952 | `fb03f5c57277636b8fde2d3a64824bb3954b0dfeef4f6581704a4a8009545633` |
| `crates/seacad-dxf-core/src/entity_field_reset.rs` | 248 | `3c292b9ad01f9b55a13c713817bb8cdd54c43f064ae4b618b0a50e61b972127d` |
| `crates/seacad-dxf-core/tests/entity_field_reset_tests.rs` | 549 | `a8adb100ecb04f3fde88dbf1ae6eddc61ccace40666cdd241c0cb46e40d3fd99` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 582 | `ac2456ba1eec4fb001b6b2ea0c7659f2073006d421f56dd2c6f60aa910381f66` |
| `docs/IMPLEMENTATION_PLAN.md` | 2010 | `2e76bc5d1413980a8cf67a238ab9232d55144376d2c440b3f0a00cf0be4cf438` |
| `docs/SUPPORT_MATRIX.md` | 1651 | `0346dcff898e72a5a27399ab4d01e4123b877044999f2d9ddbc154f3079b9933` |
