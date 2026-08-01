# M14.3j Common Entity Field Schema

## Scope

M14.3j freezes the common entity-property vocabulary needed by the unified
entity platform. The registry contains 19 roles and is generated from reviewed
JSON with stable append-only ordinals.

## Normative evidence

Autodesk's 2024 “Common Group Codes for Entities” table is anchored by
`GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD`. It states the common group codes,
wire meanings, omission defaults, application-control scopes, and that readers
must not depend on table order. The normalized 19-row fact receipt is
`0e4534ff51e87471f08d60a9852e2c42149b0b08f9484790c413fed363623844`.

The read-only legacy repositories under
`D:\SeaCad\tham khảo\New folder` were searched for behavioral fixtures. Their
fixtures corroborate `AcDbEntity`, layer, and opaque proxy usage, but no legacy
implementation or inferred version floor was copied into the schema.

## Contract

- `DxfEntityField` provides stable ordinals and exact group-code lookup.
- `DxfEntityFieldDescriptor` exposes wire type, cardinality, documented
  default, structural scope, coordinate-space classification, applicability
  review state, source id/reference/facts receipt, and row anchor.
- Cardinality is `RequiredSingleton`, `OptionalSingleton`, or
  `OptionalSequence`; only proxy graphics group 310 is a common sequence.
- Handle and BLOCK_RECORD owner fields are in the entity preamble. Ordinary
  properties are in `AcDbEntity`. Extension dictionary group 360 is confined
  to the `ACAD_XDICTIONARY` application group.
- Documented defaults are retained exactly: paper-space and visibility zero,
  linetype `BYLAYER`, material `ByLayer`, color 256, and linetype scale binary64
  `1.0`. “No default” remains an explicit `None` and is not invented.
- All common fields currently expose `NotYetReviewed` version applicability.
  The registry does not claim a field exists in every dialect merely because
  the wire family can represent it.
- Generator validation fails closed on row count, duplicate ids or group
  codes, unsupported code/wire pairs, invalid sequence shape or default,
  malformed provenance, wrong source kind, stale source facts, and stale
  generated Rust.

## Nonclaims

This checkpoint does not scan entity records, assign occurrences to scopes,
form cardinality cards, decode handles/numbers/text/chunks, validate proxy byte
counts, project semantic defaults, edit fields, or gate a writer. Application
groups, subclasses, unknown fields, and every raw byte remain under the
existing lossless indexes. No entity advances to `Complete`.

## Verification

Generator and public API tests cover all 19 rows, stable order, group-code
lookup, wire types, defaults, scopes, provenance, sequence shape, duplicate
codes, invalid defaults, deterministic receipts, and public Copy/Send/Sync
traits. The generated registry receipt is
`77df4e5ed6ae77cfa2ad5e791a971f91bc26950e43cbafc6771ad8570519ece4`.
The generator target passed 19/19 tests, the public entity-schema target passed
9/9, and the complete workspace passed 749 tests. Schema and release-evidence
freshness, dependency policy, formatting, workspace Clippy with warnings
denied, and `git diff --check` all passed.

Handwritten generator production grew by 457 lines, within the checkpoint's
200--500-line review target. Generated Rust grew independently from that
handwritten production bound. This audit omits its own hash so its receipt is
not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 80 | `a58bb199f2bc13b292d771397c31d3a1fd3d51b54e62c037effe9f6054f50368` |
| `crates/seacad-dxf-core/src/generated/entity_schema.rs` | 1,452 | `960c439b0fcdb977fe1a6ea22f1687d81ef80b1795193c7f0fb2f9bf5403c52b` |
| `crates/seacad-dxf-core/src/generated/header_schema.rs` | 2,387 | `529c28e78054c9387862eb7f25c696aed181df843fe6491cf59ebb9f7315c2a8` |
| `crates/seacad-dxf-core/src/lib.rs` | 851 | `037a21d30b424d494bb27c7c3341ffa6f131d30fd34d7d0dbfecabc98dd151ff` |
| `crates/seacad-dxf-core/tests/entity_schema_tests.rs` | 390 | `799f8e156f57648629df0d348af028579ddfe2b5c6b92904d76aa381ce84327e` |
| `crates/seacad-schema-gen/src/main.rs` | 2,803 | `60ac45922400ad93219829d1eba56c33b14086e0ec79e1673d742e97d88c2000` |
| `schema/dxf/v1/entity_common_fields.json` | 234 | `62f379c7942024a844aae6c35e29ea38f480b52eabdbcd13a8b06507339c28dd` |
| `schema/dxf/v1/manifest.json` | 11 | `2b3ca697a91b29d68ba8b3a850abf10ce017023919540d87d382da2b902718c0` |
| `schema/dxf/v1/sources.json` | 53 | `42f226ffd2d08380e8a667898a308355ef884d82f1be5bde44d3a23c69181afa` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 365 | `06943e0e1fea586593689b8a5b76ab1cb2eae77f24ed5ea7250e2940ae86c1f7` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,772 | `d4cdc52aee590bb35be6c4e6db6a81ea7c4cdcc1f3a3471a8a20602c45dbb4a8` |
| `docs/SUPPORT_MATRIX.md` | 1,403 | `5ef26f2afceb02f93031dd518753a7d2c6d6e306b0d95a2998f74825834bdd87` |
