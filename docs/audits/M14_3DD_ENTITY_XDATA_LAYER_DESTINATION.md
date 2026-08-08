# M14.3dd Entity XDATA LAYER Destination

Retrieved: 2026-08-08

## Contract

`DxfEntityXDataLayerDestinationDirectory` owns the M14.3cq source LAYER
resolution directory and an independently parsed destination document's
named-symbol directory. `SourceMissing` and
`SourceAmbiguous { target_count }` propagate without destination selection.
Only a source-unique group-1003 layer name is compared against exact group-2
names admitted from exact, completely closed destination LAYER tables.

The shared `NamedSymbolDestinationIndex` builds a sorted SHA-256 digest index
for one exact symbol-table kind. Digest equality only narrows candidates;
authoritative source and destination name spans are compared byte-for-byte in
bounded 4-KiB chunks. Destination results are `DestinationMissing`,
`DestinationUnique { target }`, or
`DestinationAmbiguous { target_count }`. Only the unique state derives the
exact destination `DxfNamedSymbolTableEntry` owned by the directory.

Compact entries bind source identity, destination identity, and exact source-
resolution ordinal. Entry, source-resolution, application, entity-slice, and
destination-target lookups revalidate owned evidence. Construction and
comparison are cancellation-aware, allocation is fallible, public ordinals and
metadata remain bounded, and `Debug` does not disclose LAYER names or payload
bytes.

## Verification boundary

The focused three-test suite covers ASCII-to-ASCII, ASCII-to-Binary,
Binary-to-ASCII, and Binary-to-Binary source/destination combinations for all
nine Core dialects, plus AC1009/AC1032 cross-dialect boundary pairs. It proves
source missing and ambiguity precedence; exact destination missing, unique, and
ambiguous results; exact ambiguity counts and owned target evidence; orphan and
application-bound occurrences; case-near rejection; and cross-document
comparison for a layer name longer than one 4-KiB chunk.

Malformed, wrong-table, duplicate-group-2, and unclosed destination LAYER
evidence fails closed. The suite also covers dual-source binding, foreign source
and destination rejection, source-resolution/application/entity lookup,
cancellation, lookup and entry-size bounds, and non-disclosing debug output.

This checkpoint does not validate layer-name syntax or a version-dependent
length limit, create or edit a missing destination LAYER record, choose among
ambiguous records, or perform case folding, trimming, decoding, or
normalization. It does not interpret application-specific payloads, compose
layer evidence with coordinate or handle transformations, encode or insert
XDATA into the destination, mutate either document, implement cross-container
cloning, or advance POINT or another entity to `Complete`.

## Gate receipts

The focused destination-LAYER suite passed 3/3 tests; the adjacent source-LAYER
and destination-APPID suites passed 6/6. The full workspace passed all 1,013
tests with zero failures or ignored tests. Generated schema and release-evidence
checks, cargo-deny advisories, bans, licenses, and sources, formatting, workspace
Clippy with warnings denied, the zero-match forbidden production scan, and
`git diff --check` passed. No dependency, lockfile, generated schema, fixture,
corpus, or release artifact changed.

The first sandboxed cargo-deny attempt could not acquire its Cargo-home advisory
lock; the authorized standalone rerun passed. A combined local gate command
reached its 120-second process timeout while tests were still passing; the
standalone workspace-test rerun completed with exit code zero. Neither event
changed repository content or weakens the standalone gate receipts.

Production changes contain 506 insertions and 73 deletions across the shared
destination symbol index, the APPID consumer refactor, the new destination
LAYER module, and public exports. The focused integration target adds 565 test
lines. This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 154 | `b512ca35a20034c3349660d2f62d7480a3abf498d0473f17cb13ddc412fbcf91` |
| `README.vi.md` | 153 | `263ce8c5484508c049d74140abd53612672ab904c902d417916d4a53b05462db` |
| `crates/seacad-dxf-core/src/entity_xdata_appid_destination.rs` | 313 | `e3c65574d17d4f8c0fddaadb9bfd945949ebe8ccf23835a62bf0c7be5571374b` |
| `crates/seacad-dxf-core/src/entity_xdata_layer_destination.rs` | 360 | `12b3ce4ce1e278500de4a63f36687df1dd3539a244c69160b57ddd3ca28d8c75` |
| `crates/seacad-dxf-core/src/named_symbol_destination.rs` | 129 | `474b3f8d48af6c00d62046c059adc476dc619630424f90059be9c41694b3807d` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,167 | `ba28cf10c9ab49125667284a4c61e34896ea1b9d6d1c5857189e3e97d740ff53` |
| `crates/seacad-dxf-core/tests/entity_xdata_layer_destination_tests.rs` | 565 | `6bc34d766d03666585042020e5b1158a4dfe013a757eb90a2161d76e3fd2090d` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `61a5bcf2d7a97a97d0eb25fe63c11a4cf80ac09d6866bb1c006b609e4e4eea17` |
| `docs/SUPPORT_MATRIX.md` | 2,618 | `bb9c628f3845c3605d9892cd0473398d423e2548fb4e7c4036938c111f2b375a` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 2,994 | `f9b54d71f3c3a00441d2c1e378ca097483cdc70a84ed8a1738a6249e1467272c` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 1,567 | `438d7870a3b2688563ce65a989c4a8e90cd18dc848de4fc9611834043b83c5e4` |
