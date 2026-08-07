# M14.3dc Entity XDATA APPID Destination

Retrieved: 2026-08-07

## Contract

`DxfEntityXDataAppIdDestinationDirectory` owns the M14.3cm source-resolution
directory and the independently parsed destination document's named-symbol
directory. `SourceMissing` and `SourceAmbiguous { target_count }` propagate
without destination selection. Only a source-unique application is compared
against exact group-2 names admitted from exact, completely closed destination
APPID tables.

A sorted SHA-256 digest index narrows candidates, but authoritative name spans
are then compared byte-for-byte across documents in bounded 4-KiB chunks;
digest equality alone never proves equality. Destination results are
`DestinationMissing`, `DestinationUnique { target }`, or
`DestinationAmbiguous { target_count }`. Only the unique state can derive the
exact destination `DxfNamedSymbolTableEntry` owned by this directory.

Compact entries bind source identity, destination identity, and the exact
source-resolution ordinal. Application, entity-slice, source-resolution, and
destination-target lookup revalidate the owned evidence. Construction and
comparison are cancellation-aware, allocation is fallible, public ordinals and
metadata remain bounded, and `Debug` does not disclose APPID names or payload
bytes.

## Verification boundary

The focused three-test suite covers ASCII-to-ASCII, ASCII-to-Binary,
Binary-to-ASCII, and Binary-to-Binary source/destination combinations for all
nine Core dialects, plus AC1009/AC1032 cross-dialect boundary pairs. It proves
source missing and ambiguity precedence; exact destination missing, unique, and
ambiguous results; exact ambiguity counts and owned target evidence; duplicate
applications; case-near rejection; and cross-document comparison for a name
longer than one 4-KiB chunk.

Malformed, wrong-table, duplicate-group-2, and unclosed destination APPID
evidence fails closed. The suite also covers dual-source binding, foreign source
and destination rejection, application/entity lookup, cancellation, lookup and
entry-size bounds, and non-disclosing debug output.

This checkpoint does not validate APPID/application-name syntax or the 31-byte
limit, create or edit a missing destination APPID record, choose among ambiguous
records, or perform case folding, trimming, decoding, or normalization. It does
not validate group-1002 structure, interpret application-specific payloads,
compose APPID evidence with coordinate, layer, or handle transformations,
encode or insert XDATA into the destination, mutate either document, implement
cross-container cloning, or advance POINT or any entity to `Complete`.

## Gate receipts

The focused destination-APPID suite passed 3/3 tests. The full workspace passed
all 1,010 tests across 192 targets with zero failures or ignored tests. Generated
schema and release-evidence checks, cargo-deny advisories, bans, licenses, and
sources, formatting, workspace Clippy with warnings denied, the zero-match
forbidden production scan, and `git diff --check` passed. No dependency,
lockfile, generated schema, fixture, corpus, or release artifact changed.

Production changed by 392 insertions and 2 deletions across the new destination
APPID module, the shared cross-document span comparison, and public exports.
The focused integration target adds 549 test lines. This audit intentionally
omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 152 | `d051b14ae84c45f0e6c39f2cae19edd9b0ce1bd29a092a9f0ca2ae77e20e2f1d` |
| `README.vi.md` | 151 | `36e422d6158c411f741a70d30f0f887714a27b60838160431abb761cdc0b4ad2` |
| `crates/seacad-dxf-core/src/entity_xdata_appid_destination.rs` | 375 | `d9a3787ebd6cdea97a53f70fed6dc0b973fb91bfd5ad33f4ea571274226439f6` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,161 | `2bfa5e252406a2032abcc9394297a925de729763cdd610ab9d5288b81ed76af3` |
| `crates/seacad-dxf-core/src/source_span.rs` | 131 | `73c4e4ab7ee20cffb0efae9460573492bbb075209ece7be083d10644ba1d4a1d` |
| `crates/seacad-dxf-core/tests/entity_xdata_appid_destination_tests.rs` | 549 | `ca68832f3ced39a62b429cfd18f1280e7149371b9b2c8b9323bb794f8f9c578e` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,550 | `c3c622870f0176a3138beba293f7013cdcd41279451b0a2942ec78ff22ca1584` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,977 | `a3b180c50812c8ec8b1d605a819c89ab752a30dea6053ada96c2389344cb5a52` |
| `docs/SUPPORT_MATRIX.md` | 2,602 | `39c7fec8aee41e7b27b4f7a6e0d6563b3119f97fab18d474d6667398b5fc158a` |
