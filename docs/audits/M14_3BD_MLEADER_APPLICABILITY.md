# M14.3bd MLEADER Applicability Receipt

## Scope

M14.3bd adds the second reviewed canonical-topic applicability range to the
generated entity registry. It admits the exact canonical `MLEADER` draft only
for AC1021 and later supported dialects.

## Normative evidence

Autodesk's 2024 earlier-version compatibility page states that multileaders
display as proxy objects in versions before AutoCAD 2008. The existing
Autodesk-backed `$ACADVER` registry maps AutoCAD 2008 to AC1021. The generated
source receipt records:

- source ID `autodesk.mleader.compatibility.2024`;
- Autodesk topic `GUID-CE870800-C598-483B-81A0-5AA0208F1851`;
- evidence kind `applicability_list`;
- normalized one-row SHA-256
  `e64a64c850bd5212a0247a0cb0adebceb0073dce85aa4463b2223511f741ea1a`.

The current Autodesk DXF inventory separately defines the canonical MLEADER
topic. The source registry and applicability manifest remain generator inputs,
and their normalized receipts fail closed when source identity or facts drift.

The user-authorized legacy tree under `D:\SeaCad\tham khảo\New folder` was
searched read-only for behavioral risks. AutoCAD-derived inventory there
observes the exact `MULTILEADER` marker, while the normative DXF topic is named
MLEADER. This checkpoint therefore does not transfer the canonical range to
that behaviorally classified alias. No external or legacy code, fixture, data,
dependency, or unsupported fact was copied, translated, vendored, or linked.

## Contract

- Canonical `MLEADER` is `NotApplicable` for AC1009, AC1012, AC1014, AC1015,
  and AC1018.
- Canonical `MLEADER` is `Applicable` for AC1021, AC1024, AC1027, and AC1032,
  with no reviewed maximum version.
- The descriptor exposes `AutodeskCompatibility` evidence and the exact source
  GUID.
- Exact alias `MULTILEADER` remains `NotYetReviewed` for all nine dialects; no
  family-to-wire minimum is inferred.
- The independent draft-admission matrix tests all 59 canonical/alias names
  over all nine ASCII and Binary dialect pairs.
- Five names now have reviewed ranges. The other 54 names remain
  `NotYetReviewed`.

## Nonclaims

This checkpoint does not add MLEADER fields, nested contexts, content, lines,
breaks, MLEADERSTYLE resolution, geometry, record encoding, insertion, update,
clone, delete, or `Complete` support. It changes no production dependency and
does not infer applicability for `MULTILEADER`, MLEADERSTYLE, or any other
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
| `README.md` | 252 | `d436baa67ddc8a0a1a88000ff9ee8a8aacbeea6fdfd082ba377028a7e356c1db` |
| `schema/dxf/v1/sources.json` | 65 | `f1c39cd89a03569c16713765326b4e55a342713ffcfbf19945768bfe5d0e664b` |
| `schema/dxf/v1/entity_applicability.json` | 65 | `94c3c18ccfa4f201c6e3196508e1da97e44c16ca7b4c319a5a94cdc40f06c1cd` |
| `crates/seacad-dxf-core/src/generated/entity_schema.rs` | 1,576 | `a9c723295ff943c189cffcafc493e0ee9e4075551931726ff7434c637b3c104b` |
| `crates/seacad-dxf-core/src/generated/header_schema.rs` | 2,392 | `15259dc62e90d1d1a91fd51eee8e1bd8f51019edfaf16a2515247351b1b0e625` |
| `crates/seacad-dxf-core/tests/entity_schema_tests.rs` | 474 | `6cb920c99f619b56969c9b4a59e3368341de2ab6374858c995f832201f74036e` |
| `crates/seacad-dxf-core/tests/entity_draft_applicability_tests.rs` | 445 | `8873a47d7349c6503976f8e62547509f8674d55814b11bf89aefc9ad73636835` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 943 | `c79c4f548fe4822a78bd23c9e08748377d98e1b4a46802e32675cceff43d8191` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,376 | `ed926e67ae155a3d5750e52075a0a8562cbf846158d8590a00819dbcb6a0a183` |
| `docs/SUPPORT_MATRIX.md` | 2,026 | `fa575969b6c3ac314f2741b38bac9beb60a1e06a03d95017408a640532616144` |
