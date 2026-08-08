# M14.3dr Entity XDATA Draft Create-New Write

Retrieved: 2026-08-08

## Contract

M14.3dr streams an M14.3dp atomic transaction only to a nonexistent path,
strictly reparses the new ASCII or Binary document, and runs complete M14.3dq
family-plus-XDATA verification. The write journal binds both receipts and
retains the executable inverse.

Existing destinations are never modified. Pre-cancelled work creates nothing;
any failure after creation removes the destination. No partial or unverified
file is published as success.

## Verification boundary

Seven focused draft/XDATA tests cover all four ASCII/Binary source-destination
pairings for all nine Core dialects plus AC1009/AC1032 cross-dialect pairs.
Exact checks cover non-empty and zero-XDATA writes, byte-identical output,
strict reparse, receipt identities, inverse restoration, existing-file
rejection, pre-cancellation, final-progress tampering, cleanup, public traits,
and non-disclosing debug output.

Application-specific interpretation, actual text transcoding, generic source-
entity-to-family draft projection, cross-container clone completion, and POINT
`Complete` remain open.

## Gate receipts

Focused draft/XDATA tests passed 7/7 and adjacent suites passed 37/37. The
workspace passed exactly 1,051 tests. Cargo-deny, formatting, schema and release-
evidence checks, workspace Clippy with warnings denied, production safety scan,
protected-surface diff, Markdown links, and `git diff --check` passed. No
manifest, dependency, lockfile, schema, corpus, legal, or release surface
changed.

The preceding cumulative M14.3dl-M14.3dq Antigravity batch independently
returned PASS in report SHA-256
`b27b982faee5ad86a47ca87346b4ac383a210097596ab80c9ed9c41ceec8e665`.

Production adds 210 lines across create-new write journaling and public exports;
focused tests add 216 lines. This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 160 | `925c1748f3800994fe453e7669b436d5e0a49b3044435d3fd42baf0a5afbd967` |
| `README.vi.md` | 158 | `d8f1454a238f238f014a69be60beb9cab89df49e9e7c27262fce49b9b67bda95` |
| `crates/seacad-dxf-core/src/entity_xdata_draft_write.rs` | 206 | `bbcf63d4f01a6f0d6baa3eae84bd92f71cd21371e36b9bd6b69cc8e9b80691ee` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,235 | `6470f8a7ae74f59f0c11c1371354fd0ade87a95235d3b511532c4991dddfdde3` |
| `crates/seacad-dxf-core/tests/entity_xdata_draft_record_tests.rs` | 998 | `f64a367bc754303c3105a87d85875238394aa736770b52a2d716769657d578ec` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `b438c8ea2ed3d500692d0335858288fd2583bb4c811a1c76a2d31a269853593e` |
| `docs/SUPPORT_MATRIX.md` | 2,819 | `61daa0885f4de6c70530705f2e520d90f0d4947eeddce37c8e899df0a5d4ed26` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 1,759 | `aefee03139d693f7c4958c1d8c2801568512e408421c63ce915d00cbc8b53d3b` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,182 | `95097ff98bf813932a3615214edb01855285f5f9b892f14465bb7cdc0349cda6` |
