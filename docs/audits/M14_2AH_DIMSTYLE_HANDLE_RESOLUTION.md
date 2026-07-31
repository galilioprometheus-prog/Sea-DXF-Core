# M14.2ah DIMSTYLE Handle Resolution

## Scope

M14.2ah composes the five M14.2af DIMSTYLE handle-field cards with SeaCad's
existing exact document-local handle identity/resolution graph. Each admitted
named DIMSTYLE record exposes fixed roles for text style, leader arrow block,
common arrow block, first arrow block, and second arrow block.

This layer reports generic identity targets only. It does not claim that a
unique target record is located in the expected STYLE or BLOCK_RECORD table.

## Normative basis

Autodesk defines DIMSTYLE groups 340 through 344 as handles for DIMTXSTY,
DIMLDRBLK, DIMBLK, DIMBLK1, and DIMBLK2 respectively:
<https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-F2FAD36F-0CE3-4943-9DAD-A9BCD2AE81DA.htm>.

M7.2 already establishes the context-neutral hard-pointer wire classification,
record identity candidates, and document-local missing/unique/ambiguous target
resolution. This checkpoint composes those retained contracts rather than
implementing another parser or handle index.

## Contract

- Every admitted DIMSTYLE record has five stable role entries in group-code
  order 340 through 344.
- A missing field is `Absent`; duplicate field occurrences are
  `MultipleValues` and no occurrence is selected.
- One occurrence reuses the generic resolver's exact `Invalid`, `Null`,
  `Missing`, `Unique`, or `Ambiguous` state.
- Unique and ambiguous identity matches remain reachable through the retained
  generic handle directory; duplicate targets are never collapsed.
- Group-102 application content cannot impersonate a field because the lower
  M14.2af cards already exclude it.
- AC1009 ASCII/Binary parity fixtures omit groups above 255 because the
  one-byte Binary group-code header cannot encode them.
- Source identity, cancellation, record/role/ordinal lookup, and public
  `Copy` or `Send + Sync` bounds remain explicit.

## Coverage and size

The focused 2-test suite runs ASCII and Binary across all nine dialects. It
covers all five roles, absent fields, duplicate field occurrences, invalid and
null handles, missing/unique/ambiguous targets, duplicate identity targets,
application-group decoys, target retrieval, lookup misses, cancellation,
source identity, and public traits.

The production module is 328 lines and the integration test is 273 lines.
Both remain below 500 lines; `lib.rs` changes by five declarative
module/re-export lines only. No user-facing/i18n string, dependency, manifest,
or lockfile changes.

## Nonclaims

This checkpoint does not validate target record type or table envelope,
resolve a target name, select duplicate fields or identities, apply DIMSTYLE
defaults, interpret tolerance strings, construct glyph geometry, edit, or
write DIMSTYLE records.

## Verification

All required local gates passed on 2026-07-31: dependency policy,
generated-schema drift, release-evidence drift, formatting, workspace Clippy
with warnings denied, 717 workspace tests with zero failures or ignored tests,
production forbidden-macro scanning, and `git diff --check`. The focused suite
passed 2/2 tests. No dependency manifest or lockfile changed.

GitHub Actions remains externally blocked before every first step by the
account billing/spending-limit annotation independently confirmed on M14.2af.
No cloud or six-platform success is claimed until billing is corrected and the
workflows are rerun.

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 79 | `d5bb3fd09be05b66ac0a601f5fa6454a9cd6a90a343e5caef06fdbf893d9a4b6` |
| `crates/seacad-dxf-core/src/lib.rs` | 811 | `f2846358e908431e00843f7d57576c899d2bb400ced9c0b494debf25f2a871f0` |
| `crates/seacad-dxf-core/src/dimstyle_handle_resolution.rs` | 328 | `2fed79d9c071df8713d52a1b4bad9e426bc8c0e6a707d1f8a18f25e5b7374168` |
| `crates/seacad-dxf-core/tests/dimstyle_handle_resolution_tests.rs` | 273 | `7a87b2979bf44da76b423168e0af3c78b12db1be3af950fe11a50774b7041df7` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 302 | `eab480f2c53503102e4e3be3d0c59cdd645e4dcae113ad93c67cbf1f8df041f1` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,664 | `1148399dffb63034c7abbf4d707a1a7f369ce1c9e2dce7c8a89132b22d53a179` |
| `docs/SUPPORT_MATRIX.md` | 1,358 | `2224377e184eed4089056b25735da62e273b24988b9c68af5152f0749e6bef28` |
