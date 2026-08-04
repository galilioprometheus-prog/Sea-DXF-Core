# M14.3cy Entity XDATA Handle Replacement Set

Retrieved: 2026-08-04

## Architectural basis

M14.3cx binds each individually ready replacement to exact source evidence.
Cloning or assembling an entity one handle at a time would still permit a
partially remapped XDATA payload. M14.3cy adds an entity-level admission layer
that is ready only when every retained group-1005 member is ready.

## Contract

`DxfEntityXDataHandleReplacementSetDirectory` owns the M14.3cw replacement
directory and groups its source-ordered entries by exact `DxfEntityRef`. Each
compact set carries source and destination identities plus an exact half-open
member range. `Ready` publishes the complete replacement count. `Unavailable`
publishes total count, unavailable count, and the first unavailable global
replacement ordinal; every detailed member state remains available through the
owned directory.

Set slice lookup verifies the complete set entry. Patch lookup additionally
requires exact membership before delegating to the M14.3cx binding. Foreign
sets, foreign replacements, unavailable members, and ordinal lookalikes fail
closed. No payload bytes are copied into set metadata.

## Verification boundary

The focused three-test suite now exercises three source entities: one wholly
ready, one wholly unavailable because its destination is missing, and one with
two unavailable members caused by ambiguous destination identity and invalid
source spelling. Exact counts, first-unavailable ordinal, set slices, ready
patch admission, unavailable patch rejection, cancellation, public metadata
bounds, and non-disclosing debug behavior run across ASCII-to-ASCII,
ASCII-to-Binary, Binary-to-ASCII, and Binary-to-Binary for all nine Core
dialects.

This checkpoint does not compose a transaction, write or clone XDATA,
interpret application-specific payloads, or advance an entity to `Complete`.

## Gate receipts

The focused replacement/set suite passed 3/3 tests. The full workspace passed
all 1,004 listed tests across 191 targets. Generated schema and release-evidence
checks, `cargo deny --locked check`, formatting, workspace Clippy with warnings
denied, the production forbidden-construct scan, and `git diff --check` all
passed. No manifest, lockfile, dependency, generated schema, locale source,
committed fixture, or external corpus changed.

Production adds one bounded 311-line module plus five root registration/export
lines. The focused target adds 70 net lines. This audit intentionally omits its
own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 502 | `7057ffbc5ff0033ad1cf08c8990c7d7cd449fca6c1f0d69c724eb25e08ad2ba5` |
| `crates/seacad-dxf-core/src/entity_xdata_handle_replacement_set.rs` | 311 | `ab63cb2a8a3c5aebfefc129cb1eb1e46ae4de757beeed1fb4989d34cb4199ce6` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,142 | `d52e02201a3dafc79445c4d25b66c6603e862ea89f83c483c175c61bbeb80d4e` |
| `crates/seacad-dxf-core/tests/entity_xdata_handle_replacement_tests.rs` | 448 | `bf6b0ca0873724684a0af6b26df53325b760a0443d2797f6f538f90dfe249e52` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,503 | `253e9947df660a52669375d054592150d7cf0f93a9829e27bed6fa959cd57077` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,926 | `0e3b043e13e8ad7b6d9dd3e0e56a23d287628d4bebe6e92b5320cdc36fa41944` |
| `docs/SUPPORT_MATRIX.md` | 2,555 | `e5a2fe88f201bdc444afb5100fa189b148c3eaa907c630439ba6ffcde5bce568` |
