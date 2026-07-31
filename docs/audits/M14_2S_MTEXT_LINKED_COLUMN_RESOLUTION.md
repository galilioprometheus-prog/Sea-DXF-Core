# M14.2s MTEXT Linked-Column Resolution

## Scope

M14.2s connects the exact linked-column evidence from M14.2r to a preceding
R2007 column-info envelope and to SeaCad's existing document-local handle
resolver. It adds no new handle grammar and makes no undocumented count or
graph-validity inference.

## Association and resolution

- A linked block associates only with the latest complete column-info block
  whose end marker precedes it in the same MTEXT record.
- Absence of such a block remains explicit rather than discarding the linked
  evidence or borrowing an entry from another record.
- Every retained group-1005 occurrence must exist in the generic handle
  resolution directory.
- Generic invalid, null, missing, and ambiguous states remain distinct.
- A unique target is classified by its exact raw record marker as MTEXT or
  another record kind, and the target record remains available.
- All owned evidence and resolver directories must share the source identity.

## Coverage and size discipline

ASCII/Binary parity covers unique MTEXT resolution for all nine supported
dialects from AC1009 through AC1032. A focused multi-target fixture covers
invalid, null, missing, ambiguous, unique MTEXT, and unique non-MTEXT states.
Another fixture covers absent column-info, cancellation, source identity, and
public trait bounds. Production and integration-test modules remain
independently below 500 lines.

## Explicit nonclaims

M14.2s does not:

- assert that a uniquely resolved MTEXT belongs to the same column graph;
- validate cycles, duplicate links, ordering, or reciprocal relationships;
- assert a formula between the declared column count and handle count;
- interpret direct flat group-50 framing;
- derive MTEXT column layout geometry; or
- edit or write MTEXT columns.

## Verification

All required gates passed on 2026-07-31: dependency policy, generated-schema
drift, release-evidence drift, formatting, workspace Clippy with warnings
denied, 677 workspace tests with zero failures or ignored tests, changed
production forbidden-macro scanning, and `git diff --check`. Three focused
linked-column resolution tests also passed independently. No dependency
manifest or lockfile changed.

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 79 | `5024b06ef28e085afd87a5719fb15aef24e36f3e31dd6570a14b767a9f196b2c` |
| `crates/seacad-dxf-core/src/lib.rs` | 735 | `9afce9b82122e5c4bab79a07c0e6db3328c6a7548a2eb02f91216a23950eda96` |
| `crates/seacad-dxf-core/src/mtext_xdata_linked_column_resolution.rs` | 292 | `37363e3b173ba44d2c9b549b01c0a40fa658fc4e851bcf4b0f5b7b526f3cbe83` |
| `crates/seacad-dxf-core/tests/mtext_xdata_linked_column_resolution_tests.rs` | 247 | `05a80ab6ee72df2784db02b4cd27f72c0d93c46575a843e2e27528b49b0793c8` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 209 | `e869f7b96eff8226a4ea2e0452b2eb8f16e8c5117ab50fa9ca13ca02d456460f` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,553 | `7eb74e00a4344b44da18da803e875845e15823163b8a9452965dac2fe73c6074` |
| `docs/SUPPORT_MATRIX.md` | 1,246 | `f89af3d9af167df39ac014da67935d5ec5ed753a7cd26a95e6e92bc475fe8d51` |
