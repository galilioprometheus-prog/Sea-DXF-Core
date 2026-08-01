# M14.3x Common Entity Field Semantics

## Scope

M14.3x projects the M14.3j generated common-field schema and M14.3k exact
evidence/cards into one typed semantic entry for every common field of every
semantic entity. It closes value decoding and reviewed defaults at the shared
platform layer; it does not edit or write entities.

## Normative evidence

Autodesk's common entity-code table states that field order must not be relied
on, identifies omitted defaults, defines the numeric/handle/text meanings, and
defines group 310 as a repeated proxy-graphics binary-chunk record:

`https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm`

The existing generated registry pins those 19 facts and their normalized
source receipt. The legacy tree was searched read-only for a matching common-
field semantic layer; no usable oracle was needed. No external or legacy code,
test, or fixture was copied, translated, vendored, linked, or used at runtime.

## Contract

- Each M14.3k card produces one stable semantic entry in entity/descriptor
  order. Cross-source lookup fails with `SourceIdentityMismatch`.
- Non-sequence fields use
  `DxfSemanticValue<DxfEntityFieldValue, DxfEntityFieldSemanticIssue>` and
  retain exact field/raw provenance.
- Explicit binary64, Int16, Int32, handle, and exact-text wire domains remain
  distinct. Binary64 must be finite. Handles use the reviewed exact hexadecimal
  grammar without normalizing their source spelling.
- Exact text stores source identity, group occurrence, value span, and document
  encoding. Caller-buffer decoding uses the existing no-replacement text path
  and verifies the returned receipt.
- Omitted optional fields materialize only generated Autodesk defaults:
  Int16, binary64 bits, static exact text, or the distinct material `ByLayer`
  state. An optional field with no default remains `Absent`.
- A missing required field is `Invalid(MissingRequired)`. Duplicate singleton
  evidence is `Invalid(MultipleValues)` with no selected raw occurrence.
  Invalid ASCII numbers/handles and Binary non-finite doubles retain exact raw
  provenance.
- Proxy group-310 data is never decoded or concatenated. Its semantic state is
  an opaque occurrence count and its exact members remain accessible through
  the evidence card.
- Construction uses checked compact ordinals, exact reservations, fallible
  allocation, source checks, and cancellation before/during/after projection.
  A public test guards entries at 320 bytes; the measured target layout is 312
  bytes.

## Dialect boundary

ASCII and Binary fixtures cover all nine supported dialects AC1009 through
AC1032. AC1009 paired fixtures omit common codes above its one-byte Binary
group-code range. Matching semantic state is required within each dialect;
field applicability remains explicitly `NotYetReviewed`.

## Nonclaims

M14.3x does not validate color, visibility, lineweight, transparency, shadow,
or proxy-size domains; resolve owner/material/plot-style/extension-dictionary
targets; require application-group closure; reconcile group 92 with group 310
bytes; expose Unicode edit input; edit, insert, clone, delete, or write; settle
field applicability; or advance any entity to `Complete`.

## Verification

The focused common-field semantic suite passes 4/4 tests and the workspace
passes all 797 tests. Schema and release-evidence freshness checks,
`cargo deny --locked check`, format, workspace Clippy with warnings denied,
workspace tests, the production forbidden-macro scan, and `git diff --check`
all pass. The checkpoint adds 430 physical production lines: 424 in the common
field semantic module and six module/export lines. This audit intentionally
omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 130 | `efa47ad4854f649602495f647fb124156eecdb1c438d1e0bc2dd6b6a8c6aa3c1` |
| `crates/seacad-dxf-core/src/lib.rs` | 939 | `ad6ea12ea3fb0d88b7e0b3310988fcaef7f1132cb086654c63b293bacf81d72a` |
| `crates/seacad-dxf-core/src/entity_field_semantic.rs` | 424 | `db69ebec7ec6adb4c64835750b126ae748ad4b86babdea1e72fafd14c5de9f7c` |
| `crates/seacad-dxf-core/tests/entity_field_semantic_tests.rs` | 566 | `cd5d06093dea2948152acbaead4527c0508df2b30fe4769fbbc6fe1e231d526f` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 547 | `216b6160f320498dd0b73b32dcdaeb14886a582c4bdda1fb5ddf37e3b88f548c` |
| `docs/IMPLEMENTATION_PLAN.md` | 1968 | `cb490ecc5c5a2859bb897d03053cc2b5da4121bff14ea9262f56e93b7344e473` |
| `docs/SUPPORT_MATRIX.md` | 1611 | `a6ecf9099d29e90853c25fa4b5cc8ff41933e377eda2bf1306bc4dffb396d2fd` |
