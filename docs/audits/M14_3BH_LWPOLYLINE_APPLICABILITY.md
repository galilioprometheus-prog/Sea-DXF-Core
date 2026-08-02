# M14.3bh LWPOLYLINE Applicability Receipt

## Scope

M14.3bh adds the fifth reviewed canonical-topic applicability range to the
generated entity registry. It admits the exact canonical `LWPOLYLINE` draft
only for AC1014 and later supported dialects.

## Normative evidence

Autodesk's old-style versus lightweight polyline guidance states that, as of
AutoCAD Release 14, 2D polylines are created as lightweight polyline entities
and that eligible earlier-release 2D polylines convert when opened. The
existing Autodesk-backed `$ACADVER` registry maps Release 14 to AC1014. The
generated source receipt records:

- source ID `autodesk.lwpolyline.compatibility.2026`;
- Autodesk topic `GUID-0A3004D1-1BF6-468A-9F69-4D0BA88857F2`;
- evidence kind `applicability_list`;
- normalized one-row SHA-256
  `1d9f7f455305cb2e7861e77e50f823c21d7c0198f53cf342e56ec3d39109699e`.

Autodesk's LWPOLYLINE DXF page independently defines the exact entity and
`AcDbPolyline` subclass under topic
`GUID-748FC305-F3F2-4F74-825A-61F04D757A50`. The source registry and
applicability manifest remain generator inputs, and their normalized receipts
fail closed when source identity or facts drift.

The user-authorized legacy trees under `D:\SeaCad\tham khảo\New folder` were
searched read-only for behavioral risks. Their AutoCAD-derived inventory and
public-repository intake observe exact `LWPOLYLINE` records across many
fixtures, including AC1014 material, but do not provide a normative complete
nine-dialect introduction matrix. They are not used to establish the minimum.
No external or legacy code, fixture, data, dependency, or unsupported fact was
copied, translated, vendored, or linked.

## Contract

- Canonical `LWPOLYLINE` is `NotApplicable` for AC1009 and AC1012.
- Canonical `LWPOLYLINE` is `Applicable` for AC1014, AC1015, AC1018, AC1021,
  AC1024, AC1027, and AC1032, with no reviewed maximum version.
- The descriptor exposes `AutodeskCompatibility` evidence and the exact
  guidance GUID.
- The independent draft-admission matrix tests all 59 canonical/alias names
  over all nine ASCII and Binary dialect pairs.
- Nine names now have reviewed ranges. The other 50 names remain
  `NotYetReviewed`.

## Nonclaims

This checkpoint does not change existing LWPOLYLINE count, vertex, width,
bulge, elevation, extrusion, segment, OCS/WCS, or geometry semantics. It does
not encode a new record, insert, update, clone, delete, or claim `Complete`
support. It changes no production dependency and does not infer applicability
for legacy `POLYLINE`, `VERTEX`, or `SEQEND` sequences.

## Verification

Focused schema tests passed 9/9 and focused draft-applicability tests passed
4/4. The full workspace passed 898/898 tests. Generated schema and
release-evidence checks, `cargo deny --locked check`, formatting, workspace
Clippy with warnings denied, workspace tests, the forbidden production-macro
scan, and `git diff --check` passed. This evidence/generated-only checkpoint
adds no handwritten production code; the generated production changes are the
applicability and source-registry receipts, so the usual 200-500 handwritten
production-line target does not apply. No production dependency changed. This
audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 253 | `178fe7024cd31f4ef4e08a48ed48e6b74b41cc80e936bbe3243dc7a51561aae6` |
| `schema/dxf/v1/sources.json` | 89 | `1ec213be3f5093e227e9b44ffd9b0e22d71968ba7de21fe5c6c65cdbb4e4e7f4` |
| `schema/dxf/v1/entity_applicability.json` | 65 | `061887513323e51025e4cbe30e472c01fd152deef0adf1d2e163b32def9cfc9b` |
| `crates/seacad-dxf-core/src/generated/entity_schema.rs` | 1,576 | `7908806d2b2ece05f60bac90547198e64b443422a09ccfa56bb20c151734e1f1` |
| `crates/seacad-dxf-core/src/generated/header_schema.rs` | 2,392 | `7445a53848de31fe9c3595886c87d2f72e01ba4842bffa02badc63615cb204d9` |
| `crates/seacad-dxf-core/tests/entity_schema_tests.rs` | 571 | `9d953cd7c29ba4a6068b6ca090f7da2afe3d17ab9d44b281b031f8219a95f99a` |
| `crates/seacad-dxf-core/tests/entity_draft_applicability_tests.rs` | 457 | `5b46deabc779d0f0018642ff3fecd2bd30b21f6d8e681da67523222dc017e425` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 976 | `f5b99c11a991319d349dccddfe4bff59c1ee70c62e7ed49912e749c1918f1c8a` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,410 | `7a6eede5769ba2a4b03215ce36dada993049a61cf3e61e8cb932868d7e0521b1` |
| `docs/SUPPORT_MATRIX.md` | 2,061 | `773fcfac29e301ead50b08c0a1a3a8892208259eadfbc6da61fa7e5c605d9328` |
