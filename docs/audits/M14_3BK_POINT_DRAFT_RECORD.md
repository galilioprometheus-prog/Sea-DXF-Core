# M14.3bk POINT Draft Record

## Scope

M14.3bk adds the first typed whole-entity draft to the unified CRUD platform.
It validates and encodes one canonical `POINT` record but deliberately leaves
the final `DxfEntityEditSession::insert` API, clone, delete, and `Complete`
support to later checkpoints.

## Normative evidence

- Autodesk [POINT (DXF)](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-9C6AD32D-769D-4213-85A4-CA9CCB5C5317.htm)
  defines `AcDbPoint`, WCS groups 10/20/30, and the optional thickness,
  extrusion, and angle fields.
- Autodesk [Common Group Codes for Entities](https://help.autodesk.com/cloudhelp/2017/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm)
  defines the non-omitted type, handle, BLOCK_RECORD owner, `AcDbEntity`,
  layout, layer, and lineweight envelope; lineweight has no omission default.
  Its explicit order-independence rule continues to govern readers even though
  this writer emits the documented canonical subclass order.
- Autodesk [About Adding an Entity without Using the Command Function](https://help.autodesk.com/cloudhelp/2025/ENU/AutoCAD-MAC-AutoLisp/files/GUID-DFDAE6CD-E753-4D01-9D9B-4D1F66B1DE6E.htm)
  identifies POINT as pre-R13 and states that post-R12 entity definitions use
  `AcDbEntity` plus the family subclass markers.
- Autodesk [AutoCAD 2000 API History](https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-ActiveX/files/GUID-75B59871-39AA-4577-8075-5522D589DABA.htm)
  marks entity `Lineweight`, block `IsLayout`/`Layout`, and the `Layouts`
  collection as new in AutoCAD 2000. Combined with the reviewed AC1015 mapping,
  this is an explicit inference that the new-record layout/lineweight envelope
  begins at AC1015; it is not presented as a direct per-group DXF range table.

The user-authorized legacy fixture
`D:\SeaCad\tham khảo\New folder\cad_2026-07-23_source\tests\fixtures\real_thua115_91.dxf`
was inspected read-only as a behavioral oracle. Its SHA-256 is
`63bd1ad7b58ddae429a6dddce8f66507d1b8f2780c30c0dc6e5bb1fc724b60f0`.
Four AC1032 POINT records inside BLOCK definitions carry owner, `AcDbEntity`,
layer, lineweight `-1`, `AcDbPoint`, and WCS location while omitting layout
410. No legacy or external code, fixture, data, or dependency was copied,
translated, vendored, linked, or used at runtime.

## Contract

- `DxfEntityDraft` is a non-exhaustive typed family enum; its first variant is
  `DxfPointDraft` with exact raw layer/layout input, typed lineweight, and exact
  binary64 WCS location values.
- Encoding requires the exact source-bound applicability plan and validates its
  transaction precondition before producing bytes. A mismatched admitted name,
  foreign source, cancellation, or stale transaction cannot be hidden.
- The layer must be nonempty, wire-safe, and resolve byte-identically to one
  entry in a completely closed same-document LAYER table. The writer never
  creates a missing symbol implicitly.
- AC1009 emits the legacy type/handle/layer/location envelope. AC1012 and later
  add owner 330, `AcDbEntity`, and `AcDbPoint`. AC1015 and later require an
  explicit valid public lineweight; no default is invented.
- AC1015+ ENTITIES placement requires an exact layout name resolving to one
  same-document `OBJECTS`/`LAYOUT` record. BLOCK placement rejects layout and
  omits group 410. Supplying modern values to earlier dialects fails typed.
- The record reuses the reviewed ASCII/Binary raw value encoder through a
  crate-private raw-group path. Text framing, finite-double, group-code,
  cancellation, allocation, and resource limits therefore stay identical to
  the common-field encoder.
- Whole-record bytes are bounded by the resource profile before publication.
  The plan owns those bytes, retains its source/handle/version/transaction, and
  redacts payload content from `Debug`.

## Verification boundary

Paired ASCII/Binary coverage spans all nine Core dialects. ENTITIES tests
compose the POINT record with the reserved `$HANDSEED` transaction and exact
placement, strict-reparse the post-image, resolve the allocated handle, publish
the expected POINT semantics, and materialize a byte-identical inverse. BLOCK
tests repeat the same transaction/reparse/semantic/inverse proof for every
R13+ dialect where the owner pointer is representable, and verify the distinct
layout envelope.

Typed negative coverage includes draft-name mismatch, empty and malformed layer,
missing and ambiguous reference classes, missing or inapplicable layout and
lineweight, non-finite coordinates, cancellation, and foreign source identity.
Unknown groups are not involved because this milestone creates a new record;
all existing source bytes remain untouched by the record plan.

## Nonclaims

This checkpoint does not add thickness, extrusion, UCS angle, Unicode semantic
input, layer/layout creation, placement-aware generic common-field
cardinality, a public insert operation on `DxfEntityEditSession`, update/reset,
clone/delete closure, full semantic postconditions for insertion, or POINT
`Complete` status. The AC1015 boundary remains the documented inference above
until a direct historical DXF field-applicability receipt replaces it.

## Verification

The focused suite passed 4/4 tests and the workspace passed 902/902 tests. The
schema and release-evidence checks, `cargo deny --locked check`, formatting,
Clippy with warnings denied, the production forbidden-pattern scan, and
`git diff --check` all passed. Clippy initially identified two collapsible
conditionals; both were simplified before the final clean run. The handwritten
production diff is 536 additions and 16 removals, with 916 lines of focused
integration coverage. No production dependency changed. This audit
intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 263 | `7efacef112ea0800b30bf0ee43f6f4025dc86c28892fd66d29975c8b917f75a1` |
| `crates/seacad-dxf-core/src/entity_draft_record.rs` | 501 | `8d2ea68e1c326e6ebcfe224429124b15b856cbae7611ce41a744e166defad83a` |
| `crates/seacad-dxf-core/src/entity_value_encoder.rs` | 512 | `287df820c2693b2f855e35f5471d80a1aac63b8ecb649e673852f58310c8e35f` |
| `crates/seacad-dxf-core/src/lib.rs` | 1062 | `da80e7fce39c30551f983a91fb08337630ec00e2c4f2bd2bdd6ea49b2cffeca2` |
| `crates/seacad-dxf-core/tests/entity_draft_record_tests.rs` | 916 | `3f314a9f31863a0215616bfa74b1e5f6843a30b0c60fe45e8b528e9b7eb2d4b1` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1013 | `0b56b44674688818a9b99318cdbf6b2c1eb02b35f81b01de7d51b819eb3200c4` |
| `docs/IMPLEMENTATION_PLAN.md` | 2445 | `cf9d7fd853a7322b0ac6f92e661b5f84d5aa8c9447cfd4cb391080b362ec1cfd` |
| `docs/SUPPORT_MATRIX.md` | 2095 | `2574e09c7f866c81b65a5198a27e81bf776341563a010519982ba648384440a1` |
