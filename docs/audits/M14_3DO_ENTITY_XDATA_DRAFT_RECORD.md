# M14.3do Entity XDATA Draft Record Composition

Retrieved: 2026-08-08

## Contract

`DxfEntityXDataDraftRecordPlan` composes one M14.3dn ready source-entity XDATA
payload with a canonical destination-bound entity draft record. Exact encoded-
entry membership and destination source identity are required. Canonical XDATA
groups are appended after the family record groups; source-entity and encoded-
state evidence remain available. Zero-XDATA is an exact no-op.

Unavailable payloads, cancellation, foreign entries, foreign destination
drafts, and bounded record growth fail closed. This checkpoint does not insert
the composed record or mutate either document.

## Verification boundary

Three focused tests cover all four ASCII/Binary source-destination pairings for
all nine Core dialects plus AC1009/AC1032 cross-dialect pairs. Exact checks
cover draft-prefix and XDATA-suffix bytes, zero-XDATA, orphan/unavailable
payloads, cancellation, dual-source identity, foreign entries, compact public
metadata, and non-disclosing debug output.

Insertion, post-write XDATA verification, application-specific interpretation,
actual text transcoding, cross-container clone completion, and POINT `Complete`
remain open.

## Gate receipts

Focused tests passed 3/3; draft/XDATA regression suites passed 20/20. The
workspace passed exactly 1,047 tests. Cargo-deny, formatting, schema and release-
evidence checks, workspace Clippy with warnings denied, production safety scan,
protected-surface diff, Markdown links, and `git diff --check` passed. No
manifest, dependency, lockfile, schema, corpus, legal, or release surface
changed.

Production adds 174 lines across the draft append helper, composition plan, and
public exports; focused tests add 517 lines. This audit intentionally omits its
own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 158 | `d502b50ef66930108571b45503d064dde81cd253b4f6f18becb0cf1d524adb79` |
| `README.vi.md` | 157 | `11908f60a8da8ac58ca0b90658e33c035b04969719604df0c85952bb521ce9c7` |
| `crates/seacad-dxf-core/src/entity_draft_record.rs` | 1,329 | `a9862c284937154a8d7680abc17376abf71e3dd67709ff984415bb94fadecb7a` |
| `crates/seacad-dxf-core/src/entity_xdata_draft_record.rs` | 142 | `ab56e23fd3cb7e1a524f0c4d958cff1e51f12f6cdcd8f6a44020bbdd98e9aea6` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,224 | `61728d85e1c4752015acd360633feffe10e046cc71927e749194626335e10edc` |
| `crates/seacad-dxf-core/tests/entity_xdata_draft_record_tests.rs` | 517 | `8af4aebc756aa50042a1d5de9814dc6d6ca43b125dbf21a92d2d63c7bedf2960` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `90bf59a072167a7979a9a35a550af5f42b51d8af68d62bd41bdcdf7366223403` |
| `docs/SUPPORT_MATRIX.md` | 2,776 | `0d184e0bcffe5c07670e993a6341b52bc40c29af67be4e10182b0f4c5130153e` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 1,718 | `6cdc09ce0a495aae7191e1c5eeda15b305e5823191953158157a2ecc6eea1702` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,139 | `84091fdb9a981edb60665e8bbca65f7c5e85b0e964359102cf3386b86c4c1b25` |
