# M14.3bf ACAD_TABLE Applicability Receipt

## Scope

M14.3bf adds the first reviewed non-underlay alias applicability range to the
generated entity registry. It admits the exact `ACAD_TABLE` draft only for
AC1018 and later supported dialects.

## Normative evidence

Autodesk's AutoCAD 2005 API History marks `IAcadTable` and the `acTable` entity
enum value as new. AutoCAD 2005 writes the AC1018 format family. The existing
normative TABLE DXF receipt independently identifies group 0 as
`ACAD_TABLE`. The generated compatibility source receipt records:

- source ID `autodesk.table.compatibility.2021`;
- Autodesk topic `GUID-4570302D-8416-402B-902C-5948068B4B7E`;
- evidence kind `applicability_list`;
- normalized one-row SHA-256
  `a5ec56f22f08d5110191e796d2b147ed5d34573b0a7dd742bb7872236b546e13`.

The source registry and applicability manifest remain generator inputs, and
their normalized receipts fail closed when source identity or facts drift.

The user-authorized legacy tree under `D:\SeaCad\tham khảo\New folder` was
searched read-only for behavioral risks. Its AutoCAD-derived inventory observes
`ACAD_TABLE` and not canonical topic label `TABLE` as an emitted entity marker,
but contains no complete version matrix. It therefore reinforces the exact-name
risk without establishing the minimum. No external or legacy code, fixture,
data, dependency, or unsupported fact was copied, translated, vendored, or
linked.

## Contract

- Exact alias `ACAD_TABLE` is `NotApplicable` for AC1009, AC1012, AC1014, and
  AC1015.
- Exact alias `ACAD_TABLE` is `Applicable` for AC1018, AC1021, AC1024, AC1027,
  and AC1032, with no reviewed maximum version.
- The descriptor exposes `AutodeskCompatibility` evidence and the exact API
  history GUID.
- Canonical topic label `TABLE` remains `NotYetReviewed` for all nine dialects;
  no topic-to-wire interchangeability is inferred.
- The independent draft-admission matrix tests all 59 canonical/alias names
  over all nine ASCII and Binary dialect pairs.
- Seven names now have reviewed ranges. The other 52 names remain
  `NotYetReviewed`.

## Nonclaims

This checkpoint does not add TABLE cell grammar, merges, styles, layout
extents, references, record encoding, insertion, update, clone, delete, or
`Complete` support. It changes no production dependency and does not claim
canonical `TABLE` as a valid entity marker.

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
| `README.md` | 252 | `c39aa15dd39ca7d1ab9da3ec299c49e8d61d2e57a9e1f4ed170089aa7483848e` |
| `schema/dxf/v1/sources.json` | 77 | `a3ea2f6d86ed5538e36f1f8454c2c63e3c05d8cb64a9a5b18a99f50e55e9d44e` |
| `schema/dxf/v1/entity_applicability.json` | 65 | `4f604034eb3d20f89fce7db853ab2b99f296a95bfbcb8b384c9ed17ed974b838` |
| `crates/seacad-dxf-core/src/generated/entity_schema.rs` | 1576 | `ebde39787799ae6dd10cd17f4d16124c951ceb8c1f66e8734e9b10733236e570` |
| `crates/seacad-dxf-core/src/generated/header_schema.rs` | 2392 | `54f3da75053c5a55187f9e7523107d4f3952fdf9a810bb5d68c8dc41996bfa7e` |
| `crates/seacad-dxf-core/tests/entity_schema_tests.rs` | 527 | `9d5da90e342eecfb3d2b4454031adc4ca9715d108214118af793a7a76b27dc6f` |
| `crates/seacad-dxf-core/tests/entity_draft_applicability_tests.rs` | 451 | `f761073a94affa2a9f4826fe7bc320fa0c50a500b17083092da67902f9fa72e1` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 959 | `439fcaaa48677fc59eefbb7922c575e9a1ab9b16b96b07779d2599402502c7fe` |
| `docs/IMPLEMENTATION_PLAN.md` | 2393 | `a4a28a6dc603424832091999875b52f8bb9643017036faabcc2dd58af2e81ea8` |
| `docs/SUPPORT_MATRIX.md` | 2044 | `4b4e3582ab6e8129dcd4e0981bb41d9d03c052af68c352a130300dab6c8a9353` |
