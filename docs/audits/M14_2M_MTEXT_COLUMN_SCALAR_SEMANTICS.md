# M14.2m MTEXT Column Scalar Semantics

## Scope

M14.2m projects the modern embedded-column evidence introduced by M14.2l into
typed, source-anchored scalar values. Raw groups remain authoritative and
available through the retained evidence directory.

## Normative and behavioral basis

- Autodesk defines the MTEXT column types as no columns, static columns, and
  dynamic columns:
  <https://help.autodesk.com/view/OARX/2025/ENU/?guid=OARX-ManagedRefGuide-Autodesk_AutoCAD_DatabaseServices_ColumnType>
- Autodesk documents positive column width, nonnegative gutter width, Boolean
  automatic-height and flow-reversal settings, and the restriction of
  per-column heights to dynamic manual-height columns:
  <https://help.autodesk.com/cloudhelp/2019/ENU/OARX-RefGuide/files/OREF-__MEMBERTYPE_Methods_AcDbMText.html>
- Autodesk documents the legacy/flat DXF column fields and the repeated column
  height structure:
  <https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-5E5DB93B-F8D3-4433-ADF7-E92E250D2BAB.htm>
- The M14.2l AutoCAD 2027 oracle and read-only legacy-project observations
  establish the modern embedded group mapping. No external implementation or
  fixture was copied.

## Behavior locked

- Embedded column type codes 0, 1, and 2 map to no columns, static, and dynamic.
- Count is retained as a nonnegative `u16`; zero is not rejected before the
  column type is considered by a later cross-field checkpoint.
- Column width must be positive and gutter width may be zero but not negative.
- Automatic-height and flow-reversal accept only exact integer 0 or 1.
- Shared and individual height measurements are nonnegative scalar values;
  their mode-dependent usability is decided by M14.2n. Failed values remain
  exact raw evidence.
- Repeated individual heights retain source order and have a bounded slice for
  each embedded object.
- Missing column type is invalid. Other missing singleton fields remain absent.
- Invalid ASCII numerics, duplicate singleton roles, unsupported type codes,
  and out-of-domain values remain distinct typed issues with raw provenance.
- Cancellation, source identity, marker lookup bounds, and public
  `Copy`/`Send`/`Sync` contracts are covered.

## File-size discipline

The implementation is deliberately split by responsibility:

- `mtext_column_semantic.rs` owns public types and the directory;
- `mtext_column_semantic_project.rs` owns internal projection helpers; and
- `mtext_column_semantic_tests.rs` owns the focused contract.

Each file remains well below the repository's 500-line micro-milestone target.

## Explicit nonclaims

M14.2m does not:

- require particular fields based on no/static/dynamic mode;
- validate count against the number of repeated height groups;
- decide whether shared versus individual heights are canonical for a mode;
- enforce the dynamic-manual-only height rule across fields;
- unify modern embedded fields with flat groups or R2007 XDATA;
- resolve legacy group-50 rotation/height ambiguity; or
- derive layout geometry, edit, or write columns.

## M14.2n refinement

Read-only R2018 behavioral evidence contains a dynamic-manual sequence whose
last individual height is zero. M14.2n therefore refined the scalar layer to
retain zero shared/individual heights and moved positive-height requirements
to the relevant cross-field strategy. The M14.2m receipt below remains the
historical receipt for its checkpoint commit.

## Verification

All required gates passed on 2026-07-31: dependency policy, generated-schema
drift, release-evidence drift, formatting, workspace Clippy with warnings
denied, 663 workspace tests with zero failures or ignored tests, changed
production forbidden-macro scanning, and `git diff --check`. The three focused
column-semantic tests also passed independently. No dependency manifest or
lockfile changed.

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 79 | `c1bc173260c1193f4afd26d8d000a907d4be9634892ac787ba4fff4434892816` |
| `crates/seacad-dxf-core/src/lib.rs` | 715 | `11a72d7e043f46dd63861e0eda83caac18c7befb5d350569041dd5c4354562f5` |
| `crates/seacad-dxf-core/src/mtext_column_semantic.rs` | 257 | `684f3cb73eb4c95dc6077d12a7c8bfe477efeb4bb41e825ea819f77b5d3acccf` |
| `crates/seacad-dxf-core/src/mtext_column_semantic_project.rs` | 282 | `5889b32df1f54a00ee81f3dd44c0839353f2031d9e3eae969e6f5314d743e537` |
| `crates/seacad-dxf-core/tests/mtext_column_semantic_tests.rs` | 327 | `f82e505d17173e70dcefb2721217d387e552c206e4bba1543a576673dfd79b18` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 173 | `4cfaad6a81f49d5f3ea2f243f34062446269a6a0b6d9212a5b1d69acf969eb7e` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,511 | `9b8b4dce46a82da0511d9e3d58578d4eaeea702f6ed81d5a8a2b8800553e6b76` |
| `docs/SUPPORT_MATRIX.md` | 1,203 | `d9d2ca6f609e701833c6216d0a28453dbee2921f5f7307efd6933cc5a61c6cbe` |
