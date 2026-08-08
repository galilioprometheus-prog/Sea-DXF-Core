# M14.3df Entity XDATA Application Destination Readiness

Retrieved: 2026-08-08

## Contract

`DxfEntityXDataApplicationDestinationDirectory` owns the M14.3de destination-
symbol directory and M14.3cn source structure directory for the same exact
source. It publishes one compact source-order entry per XDATA application.
`Ready` requires both a destination-unique APPID and every application-bound
LAYER occurrence plus valid group-1002 list structure.

Unavailable entries retain independent symbol and structure issue counts.
Callers derive every exact blocker from the two owned directories, so missing
and ambiguity states, list-control groups, issue order, and source spans are not
copied or normalized. Ready entries alone derive the exact owned destination
APPID through the composed evidence; LAYER entries remain available as the
exact application slice.

Orphan XDATA occurrences remain outside application entries and visible through
the underlying directories. Construction is cancellation-aware, allocation is
fallible, source and destination identities are independently bound, public
ordinals and metadata are bounded, and `Debug` does not disclose APPID names,
LAYER names, control payloads, or application data.

## Verification boundary

The focused three-test suite covers ASCII-to-ASCII, ASCII-to-Binary,
Binary-to-ASCII, and Binary-to-Binary source/destination combinations for all
nine Core dialects, plus AC1009/AC1032 cross-dialect boundary pairs. It proves
ready applications, symbol-only failure, structure-only failure, simultaneous
APPID/LAYER/structure failure, complete group-1002 lists, and exact unclosed-
list evidence.

The suite also covers independent owned symbol and structure issue derivation,
application and entity lookup, owned APPID/LAYER derivation, dual-source
binding, foreign source and destination rejection, cancellation, lookup and
entry-size bounds, and non-disclosing debug output.

This checkpoint does not compose per-entity 16-KiB capacity, transformed
coordinates, group-1005 handle remaps, application-specific payload semantics,
encoding, insertion, destination mutation, or cross-container cloning. It does
not create missing symbol records, choose among ambiguous records, repair list
structure, or advance POINT or another entity to `Complete`.

## Gate receipts

The focused application-destination suite passed 3/3 tests; the adjacent APPID,
LAYER, symbol, and application-destination suites passed 12/12. The standalone
full workspace gate passed all 1,019 tests with zero failures or ignored tests.
Generated schema and release-evidence checks, cargo-deny advisories, bans,
licenses, and sources, formatting, workspace Clippy with warnings denied, the
zero-match forbidden production scan, Markdown link validation, and
`git diff --check` passed. No dependency, lockfile, generated schema, fixture,
corpus, protected legal surface, or release artifact changed.

The combined local gate process reached its 300-second timeout near the end of
the still-passing workspace suite. The immediate standalone workspace rerun
completed with exit code zero in 14 seconds. This operational timeout changed
no repository content and is not used as passing evidence.

Production adds 364 lines across the new composition module and public exports.
The focused integration target adds 547 test lines. This audit intentionally
omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 155 | `7e4ac2e09fec9fc3f91e71e59a674b0b0e7ef2319ad329f0d11ad7bc1a0877c1` |
| `README.vi.md` | 154 | `e3fa31d45b54cf5562fdc686da2b5d9b3c76cb81020e271ca33b7ecb14004d4d` |
| `crates/seacad-dxf-core/src/entity_xdata_application_destination.rs` | 359 | `4f7c6a006810b2fca259eb3338d5637af12e58219ea5265b4272a09b8b701b70` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,178 | `d7f797981ba30b16c0a98bff36ea9920bfbe7430e299a1613ccf7c9aa160bc81` |
| `crates/seacad-dxf-core/tests/entity_xdata_application_destination_tests.rs` | 547 | `7afaf499110498c5db2815ae1b10aee71309180bc09685b58e17001bb56f7bb7` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `6163913461de5dcb5cc4a4221c3c254dc30c57591f4b327652d5a2322775f45a` |
| `docs/SUPPORT_MATRIX.md` | 2,648 | `03cab635982965fa45c94025664fede8fdf070b86ffaf4265a1230c585d4367a` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,022 | `0f0af5a8486ae651d59cfd9ca8b4b9d0ed84d81ab19188c1536c9f8596727023` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 1,597 | `d125a575ffed055eb1051eed7a9532b410a83916ab813c617628e896fc45d062` |
