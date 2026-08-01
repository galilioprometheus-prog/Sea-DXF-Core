# M14.3ab Common-Field Writer Order

## Scope

M14.3ab adds a separate canonical writer-order ordinal to the generated
descriptor for each of the 19 reviewed common entity fields. It is metadata
for future field emission only; no source group is inserted, moved, replaced,
or deleted by this checkpoint.

## Evidence boundary

The order is derived from the usual presentation in Autodesk's common entity
group-code table:

`https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm`

Autodesk also states that DXF readers must not rely on table order because the
order can differ and may change. The generated ordinal is therefore a
SeaCad writer policy, not a parser assumption. Existing entity readers remain
order-independent and preserve unknown groups.

The user-authorized legacy tree named in the task was only a read-only search
boundary. No ordering implementation, code, fixture, or data was copied,
translated, vendored, linked, or added at runtime.

## Contract

- Every common-field schema row carries one `write_order` value.
- The generator validates the exact reviewed mapping and rejects missing,
  duplicate, or reordered values with
  `SCHEMA_ENTITY_COMMON_WRITE_ORDER`.
- Public `DxfEntityFieldWriteOrder` is copyable, ordered metadata with an
  explicit `ordinal()` accessor.
- `DxfEntityFieldDescriptor::write_order()` exposes the value without changing
  the stable field/registry ordinal.
- The separation is observable: extension dictionary has writer ordinal 1 and
  owner has writer ordinal 2, while their registry ordinals remain reversed.
- Generated receipts include the reviewed source-fact update and are checked
  fail closed by the schema generator.

## Nonclaims

M14.3ab does not calculate a record-specific insertion offset; choose between
subclass or application-group envelopes; insert, move, replace, or delete a
group; combine edit operations; allocate handles or owners; validate domains
or references; write a destination; settle applicability; or advance any
entity to `Complete`. Canonical insertion-anchor planning begins in M14.3ac.

## Verification

The generator test freezes the exact 19-value mapping and proves a duplicate
ordinal is rejected. The public schema suite passes 9/9 and verifies every
descriptor value and the public trait bounds. All 809 workspace tests pass.
Schema freshness, release evidence, `cargo deny --locked check`, formatting,
workspace Clippy with warnings denied, the added-production forbidden-macro
scan, and `git diff --check` all pass. The production-facing diff adds 125
lines across the generator, generated entity registry, and public export. This
audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 148 | `b0cc84890adfbd1c7edfae4b1b432b0617a489b568eca592c4d4482380520250` |
| `crates/seacad-dxf-core/src/generated/entity_schema.rs` | 1576 | `fc2d2f16d868d2be61ae4a8462a79492fd6840fab337be21e4e3a544e7b5ae19` |
| `crates/seacad-dxf-core/src/generated/header_schema.rs` | 2392 | `4719222b92821b3ca541dff6892c9d8a2d3f92740c2178b978a2dd6741c3c140` |
| `crates/seacad-dxf-core/src/lib.rs` | 952 | `f398c47cfd32525fa201d06b6d9d1df2b29fa7524f12e060395405bf7cc386d7` |
| `crates/seacad-dxf-core/tests/entity_schema_tests.rs` | 421 | `d754ea23293c27ac6a6d77846ac1f5e58176202182e33d95620ac145aa5b90a3` |
| `crates/seacad-schema-gen/src/main.rs` | 3001 | `5bd80c68b7bd4bdf26c503d5b4ab32459cbbae57a0622fc2235114bf5b56cb2e` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 591 | `4fdec4c56ba6b1c8377553c7469e4038bce1f0e0ba26882dd8009018b2fd9a43` |
| `docs/IMPLEMENTATION_PLAN.md` | 2019 | `9c917ca7b1d9f4d42b704edb90f19fb3eedbb82c55fcd8e5dcfde1f1227850ad` |
| `docs/SUPPORT_MATRIX.md` | 1660 | `e5b764668371af0188b2d934907de5b4cb0ee1631a8c801de225422e384daf68` |
| `schema/dxf/v1/entity_common_fields.json` | 253 | `d4213387739709cab7f99eab6a852065fdcd2beb8382078d278571fc1f210233` |
| `schema/dxf/v1/sources.json` | 53 | `aaf8278833a6c2d6f02067e1b8769522bb8e6cbab07f28366e855eebe0cadb20` |
