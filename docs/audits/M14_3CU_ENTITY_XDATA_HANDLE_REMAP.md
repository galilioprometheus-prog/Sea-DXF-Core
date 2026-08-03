# M14.3cu Entity XDATA Handle Remap

Retrieved: 2026-08-04

## Normative and architectural basis

Autodesk assigns soft-pointer behavior to generic XDATA group 1005 and notes
that entity handles in XDATA must follow database handle lifecycle:

<https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-A2A628B0-3699-4740-A215-C560E7242F63.htm>

M14.3ct establishes exact document-local source targets. A destination handle
cannot be inferred from that source identity, so this checkpoint requires an
explicit caller-supplied mapping and refuses to guess when it is absent or
non-unique.

## Contract

`DxfEntityXDataHandleRemap::new` accepts only non-null source and destination
handles. `DxfEntityXDataHandleRemapDirectory` fallibly copies and sorts all
mapping candidates, then projects one source-bound result per M14.3ct entry.
Invalid, null, missing, and ambiguous source resolution remain separately
unusable. A unique source is `Unmapped` with no candidate, `Mapped` with exactly
one candidate, or `AmbiguousMapping` with an exact candidate count. Only
`Mapped` publishes a destination handle. The exact source identity match remains
reachable through the owned resolution directory.

Duplicate source candidates are intentionally not collapsed even when their
destination values agree. Entries include source identity so an ordinal/state
lookalike from another document cannot pass directory lookup.

## Verification boundary

The focused suite pairs ASCII and Binary fixtures for AC1009, AC1012, AC1014,
AC1015, AC1018, AC1021, AC1024, AC1027, and AC1032. It covers mapped, unmapped,
ambiguous-mapping, invalid/null/missing/ambiguous-source states; shuffled and
duplicate mappings; null source/target rejection; exact source target evidence;
source identity; cancellation; lookup bounds; metadata bounds; and
non-disclosing debug output.

This checkpoint does not prove that a mapped destination identity exists,
encode replacement group-1005 spelling, clone XDATA, interpret application
payloads, or advance any entity to `Complete`.

## Gate receipts

The focused handle-remap suite passed 3/3 tests and the full workspace passed
all 998 listed tests across 189 targets. Generated schema and release-evidence
checks, `cargo deny --locked check`, formatting, workspace Clippy with warnings
denied, the production forbidden-construct scan, and `git diff --check` all
passed. No manifest, lockfile, dependency, generated schema, locale source,
committed fixture, or external corpus changed.

Production adds one bounded 295-line module plus five root registration/export
lines; the focused target adds 274 lines. This audit intentionally omits its
own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 472 | `12ee8d72252874969245bd6de55a4c1fc0623c064564a7956c15c5448eea9c59` |
| `crates/seacad-dxf-core/src/entity_xdata_handle_remap.rs` | 295 | `6d16cc126887af2cf70e83e3b71218a91889343ea738c22ea36fb379af2647ff` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,127 | `9b637e4536f0ff269a0ad8d272103e8160dbafe48c871274bcdb8e7de2ed86d1` |
| `crates/seacad-dxf-core/tests/entity_xdata_handle_remap_tests.rs` | 274 | `bf984bce9f129c8b52429a6ccec935963001172923548df00d22276fb3da0628` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,459 | `3bce94eaa9997ff732c541f7ffb11c0d62213a91519aedc2f6a75180536d969a` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,877 | `4afe475808e57ada935aae54d343a1964ddef209b29ef150728c9b5415cecc71` |
| `docs/SUPPORT_MATRIX.md` | 2,514 | `08c9e2891330c2d76b55c6d72dfee97149563a3796029363bceeef36337eb79d` |
