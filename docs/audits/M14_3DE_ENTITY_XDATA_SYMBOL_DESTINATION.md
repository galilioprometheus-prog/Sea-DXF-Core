# M14.3de Entity XDATA Symbol Destination Readiness

Retrieved: 2026-08-08

## Contract

`DxfEntityXDataSymbolDestinationDirectory` owns the M14.3dc APPID destination
directory and M14.3dd LAYER destination directory for the same exact source and
independently parsed destination. It publishes one compact source-order entry
per XDATA application. `Ready` requires one unique destination APPID and a
unique destination LAYER target for every application-bound group-1003
occurrence.

Unavailable entries retain one deterministic issue range. The APPID blocker,
when present, precedes every application-bound LAYER blocker; each issue keeps
the exact source/destination missing or ambiguity state and source application
identity. Multiple blockers accumulate rather than short-circuit. Ready entries
derive the owned destination APPID and exact LAYER occurrence slice through the
owned evidence directories.

Orphan group-1003 occurrences remain visible through the owned LAYER directory
and are not assigned to an application. Construction is cancellation-aware,
allocation is fallible, source and destination identities are independently
bound, public ordinals and metadata are bounded, and `Debug` does not disclose
APPID names, LAYER names, or XDATA payload bytes.

## Verification boundary

The focused three-test suite covers ASCII-to-ASCII, ASCII-to-Binary,
Binary-to-ASCII, and Binary-to-Binary source/destination combinations for all
nine Core dialects, plus AC1009/AC1032 cross-dialect boundary pairs. It proves
ready applications with zero and multiple LAYER occurrences, APPID source and
destination missing states, destination APPID ambiguity, source LAYER
ambiguity, destination LAYER missing and ambiguity, case-near rejection, and
deterministic accumulation of three simultaneous symbol blockers.

Malformed destination APPID and LAYER tables produce two exact missing issues
rather than readiness. The suite also covers application and entity lookup,
owned APPID/LAYER derivation, dual-source binding, foreign source and
destination rejection, cancellation, lookup and entry-size bounds, and
non-disclosing debug output.

This checkpoint is symbol-only readiness. It does not compose structural
validation, 16-KiB capacity, coordinate transforms, group-1005 handle remaps,
application-specific payload semantics, encoding, insertion, destination
mutation, or cross-container cloning. It does not create missing APPID or LAYER
records, choose among ambiguous records, or advance POINT or another entity to
`Complete`.

## Gate receipts

The focused symbol-destination suite passed 3/3 tests; the adjacent APPID,
LAYER, and symbol-destination suites passed 9/9. The full workspace passed all
1,016 tests with zero failures or ignored tests. Generated schema and release-
evidence checks, cargo-deny advisories, bans, licenses, and sources, formatting,
workspace Clippy with warnings denied, the zero-match forbidden production
scan, Markdown link validation, and `git diff --check` passed. No dependency,
lockfile, generated schema, fixture, corpus, protected legal surface, or release
artifact changed.

Production adds 469 lines across the new composition module and public exports.
The focused integration target adds 624 test lines. This audit intentionally
omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 155 | `1c7647bceedb7ea76cad68861851bc8d472f27cc6a61cc29a83c74cd9a0a9902` |
| `README.vi.md` | 154 | `a40ca52b116044924beae189fd5ce331afc945e0633504bfb0aa3b5f011ed324` |
| `crates/seacad-dxf-core/src/entity_xdata_symbol_destination.rs` | 463 | `187db01d6c9a20cee113c8702ec705da8603cd3bee3d2f737d8372cad184db8c` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,173 | `e96d08e449b9b900bc77634f37ab7454afc2724bce2506d5566012d43f09bf78` |
| `crates/seacad-dxf-core/tests/entity_xdata_symbol_destination_tests.rs` | 624 | `d0d0600b266167df3fa65a7cc835eb47e52dfc88621432444c8a0aea634a7d7e` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `ead2b28d12541f57181abcdb0d50cf5a7cd132d771ed70868c7cd7a68fbe6ee6` |
| `docs/SUPPORT_MATRIX.md` | 2,634 | `7b2383b5c009d5dc7138fd6e1268f50e5bd4dc43a900c5fd8220f60a50efc3d6` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,009 | `abf04d37e8fa0fa2efd3e5ed600dcbadca01caf8bf8f73ecb710fc74daa7b317` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 1,583 | `e71ebbc5545b695436c01b3e10c4e2bb30f7252c7bd22fc3d52fc948c6f347e7` |
