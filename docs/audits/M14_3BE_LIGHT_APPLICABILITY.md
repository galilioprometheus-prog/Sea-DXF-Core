# M14.3be LIGHT Applicability Receipt

## Scope

M14.3be adds the third reviewed canonical-topic applicability range to the
generated entity registry. It admits the exact canonical `LIGHT` draft only
for AC1021 and later supported dialects.

## Normative evidence

Autodesk's 2024 earlier-version compatibility page requires lighting from
versions before AutoCAD 2007 to be converted to the AutoCAD 2007/2008 lighting
format. The existing Autodesk-backed `$ACADVER` registry maps that drawing
format family to AC1021. The generated source receipt records:

- source ID `autodesk.light.compatibility.2024`;
- Autodesk topic `GUID-CE870800-C598-483B-81A0-5AA0208F1851`;
- evidence kind `applicability_list`;
- normalized one-row SHA-256
  `e561222f0c714d1347c10752a575f5f676e3442914d04be9f9dad1ab0c8c70cb`.

Autodesk's DXF reference separately defines the exact LIGHT entity and its
`AcDbLight` subclass. The source registry and applicability manifest remain
generator inputs, and their normalized receipts fail closed when source
identity or facts drift.

The user-authorized legacy tree under `D:\SeaCad\tham khảo\New folder` was
searched read-only for behavioral risks. Its AutoCAD-derived inventory observes
the exact `LIGHT` marker but contains no complete version matrix, so it is not
used to establish the minimum. No external or legacy code, fixture, data,
dependency, or unsupported fact was copied, translated, vendored, or linked.

## Contract

- Canonical `LIGHT` is `NotApplicable` for AC1009, AC1012, AC1014, AC1015,
  and AC1018.
- Canonical `LIGHT` is `Applicable` for AC1021, AC1024, AC1027, and AC1032,
  with no reviewed maximum version.
- The descriptor exposes `AutodeskCompatibility` evidence and the exact source
  GUID.
- The independent draft-admission matrix tests all 59 canonical/alias names
  over all nine ASCII and Binary dialect pairs.
- Six names now have reviewed ranges. The other 53 names remain
  `NotYetReviewed`.

## Nonclaims

This checkpoint does not add LIGHT fields, light-type domains, positions,
targets, attenuation, photometric settings, rendering, shadows, record
encoding, insertion, update, clone, delete, or `Complete` support. It changes
no production dependency and does not infer applicability for SUN or any other
entity.

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
| `README.md` | 252 | `bd6cae5f13865ff85b5a47bda622c246aec80013a1dc461a81257b0574016aaf` |
| `schema/dxf/v1/sources.json` | 71 | `2a50c4b325950257b55f255f44eea25d03a0741d7515a0fa05fbb4167359411b` |
| `schema/dxf/v1/entity_applicability.json` | 65 | `80e8c6b02bf3795a49564a758cb9695c6f0dda1ab4a3967a5b8b858958438638` |
| `crates/seacad-dxf-core/src/generated/entity_schema.rs` | 1,576 | `18831f18dde0505f6c123b32d3d86ca4849031151f384bdb974f5fedbe07bebd` |
| `crates/seacad-dxf-core/src/generated/header_schema.rs` | 2,392 | `376af6fde5f4c6364687c7e6dcbc836b5e7708a841be1ed27d7c2a168a662894` |
| `crates/seacad-dxf-core/tests/entity_schema_tests.rs` | 496 | `71cbc03f5f34c8fdb72b01d34d0a770a18cde6aa8dde12ce26c8774c71bdb9fc` |
| `crates/seacad-dxf-core/tests/entity_draft_applicability_tests.rs` | 448 | `6ccc50d6fc768e01a712c1326df283f42d8276767e3dc0b2eafa4d46de4c368a` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 951 | `111319639a90d924e3bc675ab2dbfbaa656476f425589e0b742068d83f4f0f12` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,384 | `245e63b16f445a70242d966fdbbfb896fc60baec8dbdbaa3f70f1650681aa42a` |
| `docs/SUPPORT_MATRIX.md` | 2,035 | `e99036e673adabb2e5aba05a42325a8d9997bc787156a14d9b302d41e1b1bb2e` |
