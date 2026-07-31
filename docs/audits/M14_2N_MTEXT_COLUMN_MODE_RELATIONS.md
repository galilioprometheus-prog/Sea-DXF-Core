# M14.2n MTEXT Column Mode Relations

## Scope

M14.2n validates relationships among the typed modern embedded-column scalars
from M14.2m. It produces one source-anchored mode result without mutating or
discarding the scalar or raw evidence.

## Evidence

Autodesk's ObjectARX documentation defines no/static/dynamic column types,
requires positive column width and nonnegative gutter, restricts automatic
height to dynamic columns, and restricts individual column heights to dynamic
manual-height columns:
<https://help.autodesk.com/cloudhelp/2019/ENU/OARX-RefGuide/files/OREF-__MEMBERTYPE_Methods_AcDbMText.html>.

Autodesk's user documentation distinguishes static columns, dynamic automatic
height, and dynamic manual height:
<https://help.autodesk.com/cloudhelp/2020/ENU/AutoCAD-MAC-Core/files/GUID-6DF5368A-5F2F-44BE-8B80-F35FFEF80204.htm>.

Read-only inspection of
`D:\SeaCad\cad_2026-07-23_source\scratch\oracle_collection_20260702\external_fixtures\mozman_ezdxf\integration_tests\data\mtext_columns_R2018.dxf`
provided bounded behavioral examples:

- static: type 1, count 3, shared height 100, auto-height 0;
- dynamic automatic: type 2, count 0, shared height 50, auto-height 1; and
- dynamic manual: type 2, count 3, shared height 0, auto-height 0, individual
  heights 20, 30, and 0.

No external parser, source code, or fixture bytes were copied. Synthetic tests
exercise the observed field relationships across the current parser.

## Behavior locked

- No-column mode remains usable without requiring inactive column fields.
- Static mode requires usable width/gutter, positive count, and positive shared
  height. A true automatic-height flag or individual heights fail typed.
- Dynamic automatic-height mode requires usable width/gutter and a true
  automatic-height flag; individual height values fail typed.
- Dynamic manual-height mode requires usable width/gutter, a false
  automatic-height flag, and positive count.
- Dynamic manual height may use a positive shared height when no individual
  heights occur, or exactly one source-order individual height per column.
- Nonnegative individual heights are retained, including the observed terminal
  zero; negative or malformed values remain scalar failures.
- Missing required fields, scalar failures, mode violations, and height-count
  mismatch remain distinct typed issues with provenance.
- ASCII/Binary parity covers AC1009 through AC1032. Cancellation, lookup bounds,
  source identity, and public `Copy`/`Send`/`Sync` contracts are covered.

## Code-size discipline

The relation implementation is isolated from the scalar projection and remains
below 500 lines. Its focused integration test is also below 500 lines.

## Explicit nonclaims

M14.2n does not unify modern embedded storage with flat column groups or R2007
XDATA, interpret the legacy group-50 height stream, derive physical text
layout, resolve font/style metrics, edit, or write columns.

## Verification

All required gates passed on 2026-07-31: dependency policy, generated-schema
drift, release-evidence drift, formatting, workspace Clippy with warnings
denied, 666 workspace tests with zero failures or ignored tests, changed
production forbidden-macro scanning, and `git diff --check`. The six focused
scalar/relation tests also passed independently. No dependency manifest or
lockfile changed.

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 79 | `7c37f964181d554d166b05ebe8df7475e97b349d2f02ca2bbba3bd518487be66` |
| `crates/seacad-dxf-core/src/lib.rs` | 720 | `108d9bf2a8afc5ed93dbf01db96e99cf90df19248034fbfc5a08e95e3489d2fc` |
| `crates/seacad-dxf-core/src/mtext_column_semantic_project.rs` | 299 | `a7bd9e7062e427aa46e656c844b07bc227f840b24cf47880554cecda40690c8a` |
| `crates/seacad-dxf-core/src/mtext_column_relation.rs` | 347 | `4367da5df0a24bd4b4dbba8ab412f5c969925e15e476fc14ba0c920ff960b58e` |
| `crates/seacad-dxf-core/tests/mtext_column_semantic_tests.rs` | 334 | `48fca0fd517f3e31a4c8341c413ccff54066237f31bae69a1c4002b6d5b98867` |
| `crates/seacad-dxf-core/tests/mtext_column_relation_tests.rs` | 336 | `cb286b8c0f76d02f7563c9f8731a0696e84b47d444c95123af3c575d9f564841` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 179 | `aef27b779f7cddaf128321dd49b7c092180a7a4211433169b0f49bb9678f353b` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,516 | `37430944d893c8102ab8e808e73ad6616c27fd85e815c5783d2e90a2782c8512` |
| `docs/SUPPORT_MATRIX.md` | 1,209 | `16baf381104bb7b1870e9e2f4202a5c0cb3d6266549099fffbd22e828fd17dfe` |
| `docs/audits/M14_2M_MTEXT_COLUMN_SCALAR_SEMANTICS.md` | 93 | `9c4f166fcd4f64f0c560321e0180cac1c7101118f43c7b6eb8d054189bebda75` |
