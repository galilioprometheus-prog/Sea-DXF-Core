# M14.3dp Entity XDATA Draft Insertion Planning

Retrieved: 2026-08-08

## Contract

`DxfEntityXDataDraftInsertPlan` composes M14.3do with the existing destination
draft-insertion transaction without writing. It retains the exact expected
XDATA suffix, source entity, encoded entry/state, destination identity, and
family edit plan. The atomic transaction reserves the new handle and inserts
the complete canonical family-plus-XDATA record. Zero-XDATA retains an explicit
empty expectation.

Cancellation and foreign destination documents fail before a plan is
published. This checkpoint does not write a destination or claim post-write
XDATA verification.

## Verification boundary

Two focused insertion-plan tests plus the three M14.3do composition tests cover
all four ASCII/Binary source-destination pairings for all nine Core dialects
plus AC1009/AC1032 cross-dialect pairs. Exact checks cover two-patch atomic
transactions, canonical replacement suffixes, retained evidence, zero-XDATA,
cancellation, destination identity, public traits, and non-disclosing debug
output.

Destination writes, post-write XDATA verification, application-specific
interpretation, actual text transcoding, cross-container clone completion, and
POINT `Complete` remain open.

## Gate receipts

Focused draft/XDATA tests passed 5/5 and adjacent suites passed 35/35. The
workspace passed exactly 1,049 tests. Cargo-deny, formatting, schema and release-
evidence checks, workspace Clippy with warnings denied, production safety scan,
protected-surface diff, Markdown links, and `git diff --check` passed. No
manifest, dependency, lockfile, schema, corpus, legal, or release surface
changed.

Production adds 183 lines across the insertion-plan module and public exports;
focused tests add 106 lines. This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 159 | `b58ebb54f6b96931fdd5d82f09a28880cdf12e0128e550520b9b609a907cbb75` |
| `README.vi.md` | 157 | `54b53bacc52db3d4dd56c5e2650e3de85f02a001a25824b4acd2feaeec6195a4` |
| `crates/seacad-dxf-core/src/entity_xdata_draft_insert.rs` | 181 | `7024a3f27369e2849c35daf417900f1188bd291b36bcdf48c8678e1510c77b31` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,226 | `6f887bf7dc4346174dda49436363c81cd70dcbdd57f5585d2b92e86081c8a564` |
| `crates/seacad-dxf-core/tests/entity_xdata_draft_record_tests.rs` | 623 | `4c67312f4e11e105f252e3c4db843b638a1847c88e2221cc61b94b3b1966304c` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `4e72f1d3bf20cdf7138f5dd148ade1f96f943db11848d4b1ccf1e966a4226c2c` |
| `docs/SUPPORT_MATRIX.md` | 2,790 | `88aa6362ae378dc8f5b8bf85d711aace1c140398b4b86a228eae71d76c0c753a` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 1,731 | `4954bb27e45b59951eb8937746d61f8b561be026a19023ffee8447869555c2c5` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,152 | `23d1463cb94af970568b3a942f706da1cfeb3728de469527629cc3b5cda80ee3` |
