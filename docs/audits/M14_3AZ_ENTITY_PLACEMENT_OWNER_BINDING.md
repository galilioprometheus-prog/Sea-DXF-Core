# M14.3az Entity Placement Owner Binding

## Scope

M14.3az binds one source-bound entity insertion placement to a caller-selected,
uniquely identified BLOCK_RECORD owner before typed draft encoding is allowed.

## Normative evidence

- Autodesk, [Common Group Codes for Entities](https://help.autodesk.com/cloudhelp/2017/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm):
  the common outside-application group 330 is a non-omitted soft pointer to the
  owner BLOCK_RECORD object and readers must not depend on field order.
- Autodesk, [Entity Group Codes in DXF Files](https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-995ABB55-571A-4D0F-882E-8A74A738643E.htm):
  entity records show handle 5 followed by owner pointer 330 as common entity
  data.
- Autodesk, [BLOCK](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-66D32572-005A-4E23-8B8B-8726E8C14302.htm):
  a BLOCK marker carries a soft-pointer group 330 to its owner object.
- Autodesk, [About Object and Entity Codes](https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-A35B8C2A-1885-4A8E-8533-E61D8A423D62.htm):
  entities use the same group-code rules in BLOCKS and ENTITIES, undefined
  groups must remain tolerated, and field order must not be assumed.

The user-authorized legacy trees under `D:\SeaCad\tham khảo\New folder` were
searched read-only. They expose candidate/read-only owner topology and
explicitly report zero editable topology; no authoritative placement-owner
binding primitive was found. No external or legacy code, fixture, data, or
dependency was copied, translated, vendored, or linked.

## Contract

- `DxfEntityPlacementOwnerDirectory` composes the existing source-bound
  placement, closed named-symbol table, and common-owner candidate directories.
- `bind` rejects a placement from another exact source identity and checks
  cancellation before any classification.
- The caller must select a non-null handle. Missing and ambiguous identity
  matches remain distinct typed outcomes.
- A unique identity is usable only when the same raw record is admitted as an
  exact uniquely named entry from a completely closed `BLOCK_RECORD` table.
  Resolution alone never substitutes for record-kind compatibility.
- An ENTITIES placement retains the caller's selected BLOCK_RECORD. It does
  not infer model or paper space from group 67, layout 410, record order, or a
  conventional handle.
- A BLOCK placement additionally requires exactly one outside-application
  common-owner candidate on its BLOCK marker. Invalid, null, missing,
  ambiguous, and multiple candidates fail independently.
- The uniquely resolved BLOCK marker owner and requested owner must identify
  the same raw BLOCK_RECORD. No automatic correction or fallback occurs.
- A successful compact binding retains exact source identity, placement,
  owner handle, and admitted BLOCK_RECORD entry for the next draft checkpoint.
- Construction inherits the explicit source and record limits of its reviewed
  component directories; `bind` performs no untrusted-size allocation.
- ASCII, Binary, and format-neutral raw document views expose the same API.

## Dialect boundary

Paired fixtures exercise all nine supported declarations. AC1009 deliberately
omits group 330 in both ASCII and Binary fixtures because the pre-R13 Binary
wire cannot encode that group code. Its BLOCK placement therefore exposes the
same typed `NoCandidate` result in both physical formats. This proves API and
raw-evidence parity; it is not a version-applicability claim for BLOCK_RECORD
or common owner semantics.

## Nonclaims

This checkpoint does not encode entity bytes, insert a draft, assign an owner
without caller input, review owner applicability, change an existing owner,
update ownership graphs, clone or delete a closed set, or advance any entity
topic to `Complete`.

## Verification

Focused tests cover all 18 dialect/format pairs, out-of-order relevant fields,
non-null/unique/closed-table admission, wrong table kind, null, missing and
duplicate requested handles, BLOCK owner absence, multiple candidates,
malformed spelling, null, dangling and ambiguous targets, owner mismatch,
cancellation, source mismatch, and public bounds. The focused suite passed
4/4 tests and the full workspace passed 890/890 tests. Generated schema and
release-evidence checks, `cargo deny --locked check`, formatting, workspace
Clippy with warnings denied, workspace tests, forbidden-production-macro scan,
and `git diff --check` passed. Production adds 322 lines: 317 in the new module
and five module/export lines, within the usual 200–500 line checkpoint target.
No production dependency changed. This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 252 | `563f5f625389b01f2bb7fa5ce3bd56651f57d72d56bc3cb251817272665646c6` |
| `crates/seacad-dxf-core/src/entity_placement_owner.rs` | 317 | `2818fe63a6c85092d6074715a087c9f811be20bd7b7c4756a1562b04f1865617` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,050 | `9e8c71389fb8fca3e626f5fa9fc836efeb6af4672eab54c7afedb2ab2d8754ac` |
| `crates/seacad-dxf-core/tests/entity_placement_owner_tests.rs` | 472 | `09a189a6ecccad80c1cb6b541272ef06ac7d31b0aec2454f304e7fc53a2a242c` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 900 | `5226d0eef36520e55dd948cf05600cb76fdf391c75fc1fd7dd0f9655d30563b4` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,327 | `efef6d37bf06413229795398df52f64e0f89d062bedb2f7106e129afd88096b2` |
| `docs/SUPPORT_MATRIX.md` | 1,982 | `bfd90e892857c72fa3bb7c348110a7087771ffb622578e0de400b7e8ac7dc2de` |
