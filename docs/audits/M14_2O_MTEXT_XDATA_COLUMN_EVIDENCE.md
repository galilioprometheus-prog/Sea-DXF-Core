# M14.2o MTEXT XDATA Column Evidence

## Scope

M14.2o adds a bounded evidence directory for R2007-era MTEXT column information
stored in `ACAD` XDATA. This checkpoint retains the encoding exactly enough for
later semantic unification; it does not yet publish a unified column mode.

## Behavioral evidence

Read-only inspection of the owner's older project located three column modes in
`mtext_columns_R2007.dxf`. Each MTEXT record uses:

- group 1001 `ACAD`;
- group 1000 `ACAD_MTEXT_COLUMN_INFO_BEGIN`;
- group 1070 field IDs paired with group 1070 integer or 1040 double values;
- field ID 50 followed by a 1070 height count and that many 1040 heights; and
- group 1000 `ACAD_MTEXT_COLUMN_INFO_END`.

The observed field IDs match Autodesk's published flat MTEXT column IDs 75,
79, 76, 78, 48, 49, and 50:
<https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-5E5DB93B-F8D3-4433-ADF7-E92E250D2BAB.htm>.

The read-only oracle path was:
`D:\SeaCad\cad_2026-07-23_source\scratch\oracle_collection_20260702\external_fixtures\mozman_ezdxf\integration_tests\data\mtext_columns_R2007.dxf`.
No external parser, source, or fixture bytes were copied.

## Behavior locked

- App name and begin/end marker matching is byte-exact.
- Only MTEXT records in an active exact `ACAD` XDATA scope are considered.
- Complete blocks retain source-order type, auto-height, count, reversed-flow,
  width, gutter, declared-height-count, and repeated-height evidence.
- Each value exposes both the original 1070 field-ID group and its original
  1070/1040 value group.
- Field 50 consumes its declared number of consecutive 1040 heights without
  confusing a height payload with another field identifier.
- Unknown or wrong-wire fields are skipped as one bounded pair; they are never
  reinterpreted as known selectors.
- Wrong app names, inexact markers, interrupted scopes, and unclosed blocks
  leave no partial entry or value slice.
- ASCII/Binary parity covers AC1009 through AC1032, including AC1009's extended
  group-code escape for XDATA codes above 255.
- Cancellation, source identity, lookup bounds, entry slicing, and public
  `Copy`/`Send`/`Sync` contracts are covered.

## Code-size discipline

The production evidence module and focused test each remain below the 500-line
micro-milestone target.

## Explicit nonclaims

M14.2o does not:

- treat the undocumented XDATA envelope as Autodesk-normative;
- project XDATA values into the modern M14.2m/n scalar/mode types;
- validate duplicate field IDs or declared-height cardinality as semantics;
- index `ACAD_MTEXT_COLUMNS_BEGIN` linked-column handles or
  `ACAD_MTEXT_DEFINED_HEIGHT_BEGIN`;
- resolve direct flat group-50 framing outside XDATA; or
- derive geometry, edit, or write columns.

## Verification

All required gates passed on 2026-07-31: dependency policy, generated-schema
drift, release-evidence drift, formatting, workspace Clippy with warnings
denied, 669 workspace tests with zero failures or ignored tests, changed
production forbidden-macro scanning, and `git diff --check`. The three focused
XDATA-column tests also passed independently. No dependency manifest or
lockfile changed.

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 79 | `d1b478d48bf07e0498c50177ac92afa227ba8d4d6dcf72f8c29f76c89f1264ea` |
| `crates/seacad-dxf-core/src/lib.rs` | 725 | `3f96cd64b9230087ea82debd454e6d7870fe6c39b0e93c075ba12f218a568c0f` |
| `crates/seacad-dxf-core/src/mtext_xdata_column_evidence.rs` | 425 | `62864db895da8009cbd80b173d563c439f0f0593238bb18fe88c40856f5acec9` |
| `crates/seacad-dxf-core/tests/mtext_xdata_column_evidence_tests.rs` | 269 | `90b9e57a1d7a3ecc4f572bf15f10b44ef82250f03ccae9b09409f7ba04cadcb6` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 185 | `292d846861a0a9df9ae8bf36b617dfec26bcfec5c19eb394d637ca962f19242c` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,523 | `4411d98b2ce8b1a3b8036262b77c5305659c8a13166cb3d229acfe95c8679c07` |
| `docs/SUPPORT_MATRIX.md` | 1,216 | `7213e16f220e7a66fdeeb44b778a0fb2f83b0bf0b746c421bbd32f87c74abc54` |
