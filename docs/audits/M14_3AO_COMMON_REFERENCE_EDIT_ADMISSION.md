# M14.3ao Common Reference Edit Admission

## Scope

M14.3ao admits explicit common handle edits only when the generic singleton
operation is structurally safe and every reviewed reference resolves to its
exact public target kind.

## Sources

- Autodesk common entity codes assign handle identity 5, owner 330, extension
  dictionary 360, material 347, and plot style 390:
  `https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm`
- Autodesk defines `DICTIONARY`, `MATERIAL`, and `ACDBPLACEHOLDER` object
  records:
  `https://help.autodesk.com/cloudhelp/2015/ENU/AutoCAD-DXF/files/GUID-40B92C63-26F0-485B-A9C2-B349099B26D0.htm`,
  `https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-E540C5BB-E166-44FA-B36C-5C739878B272.htm`, and
  `https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-3BC75FF1-6139-49F4-AEBB-AE2AB4F437E4.htm`
- Autodesk's managed `PlaceHolder` reference states that placeholder objects
  are used in the Plot Style Name Dictionary:
  `https://help.autodesk.com/view/OARX/2026/ENU/?guid=OARX-ManagedRefGuide-Autodesk_AutoCAD_DatabaseServices_PlaceHolder`

The user-authorized legacy repository was searched read-only. Its R3 evidence
kept extension-dictionary and owner topology candidate-only with zero editable
rows, supporting the conservative specialized-operation boundary. No legacy
or external code, fixture, data, or dependency was copied, translated,
vendored, or linked.

## Contract

- `classify_entity_common_reference_edit` is source-bound, cancellation-aware,
  and returns `NotReference`, a target-bearing `Valid` value, or a typed issue.
- Group-5 identity requires handle remap; group-330 owner requires a placement
  and ownership operation. Neither can use a generic singleton update.
- Groups 360, 347, and 390 require a typed handle, reject null, preserve
  missing/ambiguous lookup, and require the exact target marker plus `OBJECTS`
  section before admission.
- `DxfEntityEditSession` builds the handle-identity directory only on the first
  non-null reviewed reference request and reuses it. Every rejection leaves
  the queue and immutable source unchanged.
- Accepted modern edits continue through replacement/insertion planning,
  strict semantic post-image verification, and executable exact inverse.

## Dialect boundary

The classifier has paired ASCII/Binary coverage for all nine dialects because
it validates an already-typed proposed handle against document targets.
Materialized reference updates cover AC1012 through AC1032. AC1009 Binary
cannot encode group 347 and fails through the existing typed physical wire
gate. AC1009 ASCII can spell the numeric group code; this checkpoint does not
promote that fact into version applicability.

## Nonclaims

This checkpoint does not implement handle remapping, owner placement,
dictionary membership, one-owner conformance, pointer lifecycle,
cross-document remap, clone/delete closure, version applicability, family
graphs, or `Complete` entity support.

## Verification

Focused tests cover all three accepted kinds, null, missing, duplicate target,
wrong marker, correct marker in the wrong section, wrong value kind, identity
and owner specialized-operation failures, lazy session rejection, cancellation,
nine-dialect classifier parity, eight-modern-dialect ASCII/Binary
materialization, strict target semantics, and exact inverse restoration. Final
gate counts, production diff, and artifact hashes are recorded below after the
release gate. The focused suites passed 16/16 tests and the full workspace
passed 858/858 tests. Generated schema and release-evidence checks, `cargo deny
--locked check`, formatting, workspace Clippy with warnings denied, workspace
tests, and `git diff --check` all passed. The full gate identified one legacy
semantic-verification fixture whose proposed MATERIAL handle had no target;
the fixture now contains an exact `OBJECTS`/`MATERIAL` target and passes the
new admission contract. The production diff is 300 added and 15 removed lines:
238 in reference edit admission, 34 net in edit-session integration, 8 net in
shared target-kind matching, and 5 module/exports. This audit intentionally
omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 209 | `b6b8bbb55e9ff900cd599c8c6cf9e39035ce382090754f39bd7f41097f59bbe4` |
| `crates/seacad-dxf-core/src/entity_common_reference_edit.rs` | 238 | `1055c69519d5fbad862641aec03d43b798529c7c1fffc8d1292f2594c22fda03` |
| `crates/seacad-dxf-core/src/entity_common_reference_target.rs` | 350 | `c8db4a1df9601cc76471263d7e56e820717a7ccdfae827f08e2bd55c8b4fef03` |
| `crates/seacad-dxf-core/src/entity_edit_session.rs` | 538 | `dcffa658b8f4cdf2da5f7410ff8d1e902301d994daaa17e207ff16c0a61b716b` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,016 | `40dea995e534d249a192286e239391ec6806d91969efcdb623ad382e9f4f7a21` |
| `crates/seacad-dxf-core/tests/entity_common_reference_edit_tests.rs` | 558 | `56dcc1c05fa81b8fa93a813659db918cb94ad19202f798c06ae7ba4df0cf2ea1` |
| `crates/seacad-dxf-core/tests/entity_edit_verification_tests.rs` | 474 | `8c4a1969453f25244092ff89f8faeab26ce0cc2cda923f19878472bd1eecb97a` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 753 | `b7eb04231902f4e45f1c79d7a3e0b43621cc603da5ec0f6a26f13c46e18cae21` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,183 | `2140268575a80d1e741fcf743dbdd7538c10c881381a5de090ae0b7fbfda3f49` |
| `docs/SUPPORT_MATRIX.md` | 1,844 | `df99fab775e889b3e34da7db37b166e064a8f696fbad18b9d28e3e7996d5157f` |
