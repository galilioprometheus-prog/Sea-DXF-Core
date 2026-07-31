# M14.2r MTEXT XDATA Linked Columns

## Scope

M14.2r retains the R2007-era `ACAD_MTEXT_COLUMNS` XDATA envelope as bounded,
source-anchored evidence. A read-only legacy fixture showed selector 47,
a declared count, and group-1005 handles; no legacy parser source or fixture
bytes were copied.

Autodesk documents group 1005 as a database-entity handle in XDATA and states
that such handles are translated with their corresponding entities. SeaCad
therefore reuses its existing group-1005 soft-pointer parser rather than
creating an MTEXT-specific handle grammar.

## Fail-closed framing

- Recognition occurs only inside an exact `ACAD` XDATA application scope.
- A candidate requires the exact begin marker, selector 47 in group 1070, a
  group-1070 declared count, zero or more contiguous group-1005 handles, and
  the exact end marker.
- Wrong apps, markers, selectors, count wires, intervening groups, application
  boundaries, and missing end markers publish no entry or partial handle
  slice.
- The declared count remains typed even when its ASCII integer spelling is
  invalid.
- Every handle preserves its source group, soft-pointer class, original raw
  spelling, and parsed or lexically invalid result.

## Coverage and size discipline

ASCII/Binary parity covers all nine supported dialects from AC1009 through
AC1032, including AC1009's extended Binary group-code framing. Negative tests
cover every exact-envelope boundary; separate checks cover invalid counts,
invalid handles, cancellation, source identity, marker lookup, and public
`Copy`/`Send`/`Sync` traits. Production and integration-test modules remain
independently below 500 lines.

## Explicit nonclaims

M14.2r does not:

- associate a linked-column envelope with column-info semantics;
- assert a relationship between declared count and handle count;
- resolve a handle to a target MTEXT or validate graph membership;
- interpret direct flat group-50 framing;
- derive MTEXT column layout geometry; or
- edit or write MTEXT columns.

## Verification

All required gates passed on 2026-07-31: dependency policy, generated-schema
drift, release-evidence drift, formatting, workspace Clippy with warnings
denied, 674 workspace tests with zero failures or ignored tests, changed
production forbidden-macro scanning, and `git diff --check`. Three focused
linked-column tests also passed independently. No dependency manifest or
lockfile changed.

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 79 | `88835583cfde90a21cb9815787e855590f0e27f38ee31f06c752c4af6d04ebbf` |
| `crates/seacad-dxf-core/src/lib.rs` | 730 | `6e2bc4f0461004440d8b79c4f88ca91eafef97a5fe14c9864c94749d0ea6fa61` |
| `crates/seacad-dxf-core/src/mtext_xdata_linked_column.rs` | 318 | `a4cf634aad5230e145a55e5c19e2833be3b109c3c3c1cb13f13e07c87eb1d7b9` |
| `crates/seacad-dxf-core/tests/mtext_xdata_linked_column_tests.rs` | 231 | `5eeab3328ef52b413dfc76062b91d03ebefe1af3c7674400cb50a37ab05c25e0` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 204 | `db0ecbf507b3d704a524b52427db6d315bbab380af767934b93d28dfed25105f` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,545 | `ef275fb393d12b966b01fa3d0adf8dd80760923bff48874c0469951473e6a30e` |
| `docs/SUPPORT_MATRIX.md` | 1,238 | `63f63dfcacce6ad7606863874b699fcb39729b80ad8a4e9162cf4752e575a21a` |
