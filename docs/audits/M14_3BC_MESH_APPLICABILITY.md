# M14.3bc MESH Applicability Receipt

## Scope

M14.3bc adds the first reviewed canonical-topic applicability range to the
generated entity registry. It admits the exact canonical `MESH` draft only for
AC1024 and later supported dialects.

## Normative evidence

Autodesk's MESH compatibility page identifies the newer MESH object type as
implemented in the AutoCAD 2010 context. The existing Autodesk-backed
`$ACADVER` registry maps AutoCAD 2010 to AC1024. The generated source receipt
therefore records:

- source ID `autodesk.mesh.compatibility.2018`;
- Autodesk topic `GUID-73981F72-60DD-46E7-BED1-BAF9692490A5`;
- evidence kind `applicability_list`;
- normalized one-row SHA-256
  `1c0f0b305dd3c6c70691569bac124e424b25cfe53bf8f14524b963f9a6563c28`.

The source registry and applicability manifest are generator inputs. Their
normalized receipts fail closed when source identity or facts drift.

The user-authorized legacy tree under `D:\SeaCad\tham khảo\New folder` was
searched read-only for behavioral risks. It contains entity readers and MESH
topology handling but no normative applicability primitive used here. No
external or legacy code, fixture, data, dependency, or unsupported fact was
copied, translated, vendored, or linked.

## Contract

- Canonical `MESH` is `NotApplicable` for AC1009, AC1012, AC1014, AC1015,
  AC1018, and AC1021.
- Canonical `MESH` is `Applicable` for AC1024, AC1027, and AC1032, with no
  reviewed maximum version.
- The descriptor exposes `AutodeskCompatibility` evidence and the exact source
  GUID.
- The independent draft-admission matrix tests all 59 canonical/alias names
  over all nine ASCII and Binary dialect pairs.
- Four names now have reviewed ranges: canonical MESH plus DGN, DWF, and PDF
  underlay aliases. The other 55 names remain `NotYetReviewed`.

## Nonclaims

This checkpoint does not add MESH field schemas, tuple/sequence evidence,
topology, geometry, subdivision evaluation, record encoding, insertion,
family validation, update, clone, delete, or `Complete` support. It changes no
production dependency and does not infer applicability for any other entity.

## Verification

Focused schema tests passed 9/9 and focused draft-applicability tests passed
4/4. The full workspace passed 898/898 tests. Generated schema and
release-evidence checks, `cargo deny --locked check`, formatting, workspace
Clippy with warnings denied, workspace tests, and `git diff --check` passed.
This evidence/generated-only checkpoint adds no handwritten production code;
the generated production changes are the applicability and source-registry
receipts, so the usual 200-500 handwritten production-line target does not
apply. No production dependency changed. This audit intentionally omits its
own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 252 | `3028f68aff831548c32a38332576d081a45f615f31e52ff0ae3906008ca73c53` |
| `schema/dxf/v1/sources.json` | 59 | `4179764c50ed5937a8f5dc10d8e367dcd9e46222c15c21c5ff116316a2705d9f` |
| `schema/dxf/v1/entity_applicability.json` | 65 | `01d882caea6c9029b861b4b2fa3e5016bcbdafb160f39f8efa659bf66f2d1f4e` |
| `crates/seacad-dxf-core/src/generated/entity_schema.rs` | 1,576 | `5bc8f3ec18beb0948312d6ac0bc9b1f930e6ee4e2ea6d70fe8dab9c8fd340496` |
| `crates/seacad-dxf-core/src/generated/header_schema.rs` | 2,392 | `b7f6181c2e98f7aaba263f41ec9f9c37f625489f3d26ea1981425f25f65d56fa` |
| `crates/seacad-dxf-core/tests/entity_schema_tests.rs` | 443 | `80733ab9e99d8eb090ecd65b7d8f0714933f6e0f601b7a18bb47270ba47e8cce` |
| `crates/seacad-dxf-core/tests/entity_draft_applicability_tests.rs` | 442 | `5d243bbd8a39730e6a6d1c086c21ff73cf62cb64b40f4588de091a1d45ce1530` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 934 | `da5e178312aa47a03994bf9477a0f2d7b2cd6a029189427d5e64f82a6f8ee83a` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,366 | `a4733b68035a1ea983f6757b0d2bcc3428910d48b5e5277a09811edd866d846e` |
| `docs/SUPPORT_MATRIX.md` | 2,016 | `e7c388f4eea188b831e8c464e9336b3bb65bb72e7516c9e0e7dabba7d8952f71` |
