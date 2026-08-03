# M14.3cv Entity XDATA Handle Destination

Retrieved: 2026-08-04

## Normative and architectural basis

Autodesk assigns generic XDATA group 1005 database-handle semantics:

<https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-A2A628B0-3699-4740-A215-C560E7242F63.htm>

M14.3ct proves the source target and M14.3cu requires an explicit destination
handle mapping. A mapping alone cannot prove destination existence or
uniqueness, so this checkpoint resolves it against an independently parsed
destination document.

## Contract

`DxfEntityXDataHandleDestinationDirectory` owns the M14.3cu remap directory and
the destination document's handle identity directory. Remap-unavailable states
propagate exactly. Each mapped destination handle becomes `Missing`, `Unique`,
or `Ambiguous` with an exact candidate count. Only `Unique` can derive an exact
destination `DxfHandleIdentityMatch`; other states expose no target record.

Every compact entry carries source and destination identities plus a remap
ordinal. Destination target evidence is derived through the owned index rather
than copied per entry. Foreign entries with matching ordinals or states fail
lookup whenever either identity differs.

## Verification boundary

The focused suite covers ASCII-to-ASCII, ASCII-to-Binary, Binary-to-ASCII, and
Binary-to-Binary source/destination combinations for AC1009, AC1012, AC1014,
AC1015, AC1018, AC1021, AC1024, AC1027, and AC1032. It covers every remap-
unavailable state, destination missing/unique/ambiguous, duplicate destination
identities, exact target evidence, dual-source identity, cancellation, lookup
and metadata bounds, and non-disclosing debug output.

This checkpoint does not encode replacement group-1005 spelling, write or
clone XDATA, interpret application payloads, or advance any entity to
`Complete`.

## Gate receipts

The focused destination suite passed 3/3 tests and the full workspace passed
all 1,001 listed tests across 190 targets. Generated schema and release-evidence
checks, `cargo deny --locked check`, formatting, workspace Clippy with warnings
denied, the production forbidden-construct scan, and `git diff --check` all
passed. No manifest, lockfile, dependency, generated schema, locale source,
committed fixture, or external corpus changed.

Production adds one bounded 268-line module plus five root registration/export
lines; the focused target adds 370 lines. This audit intentionally omits its
own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 480 | `715a93c6ebcb7f61413bf05904180067203b7fbe43e5f03f7984579769855e51` |
| `crates/seacad-dxf-core/src/entity_xdata_handle_destination.rs` | 268 | `7512a887815ac6874878c03f25e197f1d0e863d1622cad86095588ad9d12e994` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,132 | `8c31e4498579b84e8d350b77f585e34f2c6a0199dd9a3ac5e6b151d00f325f20` |
| `crates/seacad-dxf-core/tests/entity_xdata_handle_destination_tests.rs` | 370 | `a6d6ae206e41a075e647c80421e82916b9452ad6b8e9bae9cb8b00bb8667297e` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,472 | `d122b18f5ed99017c3c4079266b2115af49dc84826c6b40048792193d61b1d49` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,891 | `04bbd6e5406fd60e85bd5c6522c010735f0b691a0bd192cdf88dfb7073552c13` |
| `docs/SUPPORT_MATRIX.md` | 2,525 | `65269679c2586eaeb00935300041bad51ca9510dd7f9d98ebfb91abc98b704c5` |
