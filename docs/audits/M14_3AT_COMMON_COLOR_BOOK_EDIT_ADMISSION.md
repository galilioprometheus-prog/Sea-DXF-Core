# M14.3at Common Color-Book Edit Admission

## Scope

M14.3at validates explicit common group-430 edits against the reviewed
color-book envelope and the exact target entity's existing color semantics,
then admits valid modern edits to the verified transaction pipeline.

## Sources

- Autodesk common entity codes define groups 62, 420, and 430:
  `https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm`
- Autodesk `acad_truecolordlg` defines group 430 as
  `colorbook$colorname`, returns groups 62/420/430 for a color-book choice, and
  publishes the `RAL CLASSIC$RAL 1003` example:
  `https://help.autodesk.com/cloudhelp/2018/DEU/AutoCAD-AutoLISP-Reference/files/GUID-E6FF435F-9E66-4F37-8770-2E3FB87B8E0B.htm`

The user-authorized legacy repository remained read-only and supplied no
usable color-book admission implementation. No external or legacy code,
fixture, data, dependency, or color-book payload was copied, translated,
vendored, or linked.

## Contract

- The public classifier is bound to one exact same-source entity. Non-color
  fields, wrong value kinds, missing/empty/multiple separators, unusable true
  color, and unusable indexed color remain distinct typed outcomes.
- A valid receipt retains only offsets and reviewed scalar values. Callers can
  recover the two components from the exact proposed byte slice; candidate
  bytes are not copied or exposed by debug output.
- `DxfEntityEditSession` builds and reuses the common-domain directory lazily
  only for an exact color-name proposal on an existing key. Rejection leaves
  the pending queue and source unchanged.
- Accepted edits use existing canonical wire encoding, insertion/replacement,
  strict reparse, structured read semantics, generic semantic/raw
  verification, and executable inverse restoration.

## Nonclaims

This checkpoint does not access `.acb` files, establish that an external name
exists, verify name-to-RGB/ACI mapping, normalize names, combine pending edits
to groups 62/420/430, review dialect applicability, add family graph CRUD, or
advance any entity topic to `Complete`.

## Verification

Paired ASCII/Binary tests cover all nine dialects, valid modern admission,
AC1009 relation rejection, every delimiter failure, wrong value/non-color
fields, missing/invalid true color, duplicate indexed color, source mismatch,
cancellation, public traits/bounds, rejected no-op sessions, modern strict
post-image semantics, and byte-identical inverse restoration. Final gate
counts, production diff, and artifact hashes are recorded after the release
gate. This audit intentionally omits its own hash.

The focused color-book and symbol regression suites passed 7/7 tests and the
full workspace passed 871/871 tests. Generated schema and release-evidence
checks, `cargo deny --locked check`, formatting, workspace Clippy with warnings
denied, workspace tests, and `git diff --check` all passed. The production diff
is 350 added and 18 removed lines: 289 in the new classifier, 56 added and 10
removed in session integration, 8 removed from the former symbol lock, and 5
public module/export lines. No production dependency changed.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 229 | `729bd5e7579ef74559339306385c296e9fd835983357d530a5dda38214233db0` |
| `crates/seacad-dxf-core/src/entity_common_color_book_edit.rs` | 289 | `8f63c97474f34d966f4cfa6af997f9fe9b113ac19b4f974b94da936224819e5f` |
| `crates/seacad-dxf-core/src/entity_common_symbol_edit.rs` | 199 | `3e6228d1042feace59e8cf0833b7b0a6737d57cc1e2efe25f2d69a4c402bb047` |
| `crates/seacad-dxf-core/src/entity_edit_session.rs` | 674 | `07b25f9a5229d20aab5c65048dc83611ff321d203ec7676dedcf697d6f7e8e97` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,036 | `2fef7fa33bf0dc21a4cacea27119d9baf0afabfe92fe30c84fb276bc4120ddfb` |
| `crates/seacad-dxf-core/tests/entity_common_color_book_edit_tests.rs` | 419 | `6f8ca2d84f4eb7090138450d32ace8cf4b01f9944d5dca8d5cc221e26a07ee42` |
| `crates/seacad-dxf-core/tests/entity_common_symbol_edit_tests.rs` | 472 | `1dbb097ad44e86c786b8bbe123c148f41cb4a18dd9f56f5e861fbed0bb966805` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 818 | `cba94b2a94ae3e88d6da4a43ba9425876b5ac973c4d664fe473f36129f726174` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,248 | `b16ec6c7bac2006b4741ccda9da51d7abe2f1b818c2aac4ee323122012a15f8b` |
| `docs/SUPPORT_MATRIX.md` | 1,908 | `a9dce812a3bd914e16622ca1d2e8a4ce7b535e5331fb2dd8beb489c7e4572875` |
