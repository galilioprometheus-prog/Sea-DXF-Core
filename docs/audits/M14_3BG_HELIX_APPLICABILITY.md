# M14.3bg HELIX Applicability Receipt

## Scope

M14.3bg adds the fourth reviewed canonical-topic applicability range to the
generated entity registry. It admits the exact canonical `HELIX` draft only
for AC1021 and later supported dialects.

## Normative evidence

Autodesk's AutoCAD 2007 API History marks `IAcadHelix`, `AcHelixConstrainType`,
and `AcHelixTwistType` as new. The existing Autodesk-backed `$ACADVER` registry
maps the AutoCAD 2007 drawing format family to AC1021. The generated source
receipt records:

- source ID `autodesk.helix.compatibility.2024`;
- Autodesk topic `GUID-CC6BE90C-5ABE-4DE5-9390-B36FDCFF798B`;
- evidence kind `applicability_list`;
- normalized one-row SHA-256
  `65aa11c362793c3c6b6a66f2efb59c06831c643dec37a7ba1b7ebbfa3960d65f`.

Autodesk's HELIX DXF page independently defines helix entity fields and the
`AcDbHelix` subclass under topic
`GUID-76DB3ABF-3C8C-47D1-8AFB-72942D9AE1FF`. The source registry and
applicability manifest remain generator inputs, and their normalized receipts
fail closed when source identity or facts drift.

The user-authorized legacy trees under `D:\SeaCad\tham khảo\New folder` were
searched read-only for behavioral risks. Their AutoCAD-derived inventory and
public-repository intake observe the exact `HELIX` marker and `AcDbHelix`
subclass. They also record sparse HELIX examples but no complete nine-dialect
version matrix, so they are not used to establish the minimum. The same search
showed that Autodesk topic names can differ from emitted markers, notably
`SECTION` versus observed `SECTIONOBJECT`; that risk is why SECTION remains
fail-closed. No external or legacy code, fixture, data, dependency, or
unsupported fact was copied, translated, vendored, or linked.

## Contract

- Canonical `HELIX` is `NotApplicable` for AC1009, AC1012, AC1014, AC1015,
  and AC1018.
- Canonical `HELIX` is `Applicable` for AC1021, AC1024, AC1027, and AC1032,
  with no reviewed maximum version.
- The descriptor exposes `AutodeskCompatibility` evidence and the exact API
  history GUID.
- The independent draft-admission matrix tests all 59 canonical/alias names
  over all nine ASCII and Binary dialect pairs.
- Eight names now have reviewed ranges. The other 51 names remain
  `NotYetReviewed`.

## Nonclaims

This checkpoint does not change existing HELIX scalar/vector evidence, compose
the embedded spline and helix subclasses, validate analytic relationships,
encode records, insert, update, clone, delete, or claim `Complete` support. It
changes no production dependency and does not infer applicability for SECTION,
SECTIONOBJECT, SURFACE, or any other AutoCAD 2007-era entity.

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
| `README.md` | 252 | `457fceffbc8d548df554f9ab8fb8c3749d866f916587ccaacbb979817cc02c19` |
| `schema/dxf/v1/sources.json` | 83 | `cad032b30225db8b047ed0c74377a7ad2c5adc33db2927ad2aaa740971ad9231` |
| `schema/dxf/v1/entity_applicability.json` | 65 | `010f81855f247e0871344e65779fafe7ee902cb566312b923dfc8cd46e0edc2d` |
| `crates/seacad-dxf-core/src/generated/entity_schema.rs` | 1,576 | `7a2a8e0e58439d6821739924feb2e2efca66c0032bf8ea7dcdfd98be35828627` |
| `crates/seacad-dxf-core/src/generated/header_schema.rs` | 2,392 | `23ff5054189f0a819413c7db803f0e7f843cf57955155c51091e3f40c91dad83` |
| `crates/seacad-dxf-core/tests/entity_schema_tests.rs` | 549 | `ccf467376a609ab4ab7bd57c8a0285caea78b69de2d9d2a4ceb5c79d98f8f4f8` |
| `crates/seacad-dxf-core/tests/entity_draft_applicability_tests.rs` | 454 | `6673f96278a953c0e426e514365b97fce18a5d7feba51142e678bdd4c358edef` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 967 | `0ea103ad5d5876a6c80e3c547e53139a8246c7b2e4d9ea14e3ea1f6349c3403f` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,401 | `15abd6c173ef6dc5aa0b931625b43f21e1d1f640346b695132269fdabf90563f` |
| `docs/SUPPORT_MATRIX.md` | 2,052 | `e36c125d10238ea37075a14480aa19389a06742007976d796959932a7ae5cbb7` |
