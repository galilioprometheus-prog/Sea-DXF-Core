# M14.3cw Entity XDATA Handle Replacement

Retrieved: 2026-08-04

## Normative and architectural basis

Autodesk assigns database-handle string semantics to generic XDATA group 1005:

<https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-A2A628B0-3699-4740-A215-C560E7242F63.htm>

M14.3cv proves that a caller-mapped handle names exactly one record in a
separately parsed destination. M14.3cw supplies the canonical destination wire
group required before any transaction or clone operation can replace the
source occurrence.

## Contract

`DxfEntityXDataHandleReplacementDirectory` owns the M14.3cv destination
directory and one bounded, flattened replacement-byte store. Missing,
ambiguous, and remap-unavailable destination states propagate exactly and
publish no byte slice. A unique destination is encoded through the existing
`DxfEntityGroupEncoder` as one complete group 1005 in the destination format
and supported `$ACADVER` dialect.

Ready groups use uppercase hexadecimal across the full 64-bit handle domain.
ASCII output includes the group-code and value line framing. Binary output
includes the dialect-specific group code and NUL-terminated handle payload;
AC1009 uses the required `0xFF` extended-data escape followed by the little-
endian 1005 code. Compact entries bind source identity, destination identity,
and their exact M14.3cv entry. Foreign entries cannot retrieve destination
evidence or bytes, and directory debug output omits the byte store.

## Verification boundary

The focused suite covers ASCII-to-ASCII, ASCII-to-Binary, Binary-to-ASCII, and
Binary-to-Binary source/destination combinations for AC1009, AC1012, AC1014,
AC1015, AC1018, AC1021, AC1024, AC1027, and AC1032. It verifies exact complete
group bytes for `u64::MAX`, uppercase spelling, AC1009 escape framing, unique,
missing, ambiguous, invalid-source propagation, absence of bytes for every
unavailable state, dual-source identity, cancellation, lookup and metadata
bounds, and non-disclosing debug output.

This checkpoint does not create a transaction patch, clone or write XDATA,
interpret application-specific payloads, or advance any entity to `Complete`.

## Gate receipts

The focused replacement suite passed 3/3 tests and the preceding destination
suite passed 3/3 tests. The full workspace passed all 1,004 listed tests across
191 targets. Generated schema and release-evidence checks,
`cargo deny --locked check`, formatting, workspace Clippy with warnings denied,
the production forbidden-construct scan, and `git diff --check` all passed. No
manifest, lockfile, dependency, generated schema, locale source, committed
fixture, or external corpus changed.

Production adds one bounded 329-line module plus five root registration/export
lines; the focused target adds 342 lines. This audit intentionally omits its own
hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 488 | `6c7f8be8d0fe66ffc8f558d89000e57141df0229019dec5b0005cfbb9e2f16f3` |
| `crates/seacad-dxf-core/src/entity_xdata_handle_replacement.rs` | 329 | `2f710fb59f63c5045ac0a5e939bf64b152e5240135125f3f0741cbf5ddbcd6ac` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,137 | `5a89f6255af1111e6a4cf0fce421e02a8a34dd8ff6c23adfac1b104a834fd0db` |
| `crates/seacad-dxf-core/tests/entity_xdata_handle_replacement_tests.rs` | 342 | `cc3f19264906d37c34eb6abe9e13aea2e452c6669956ef89fb3d90e98b74b86d` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,483 | `5becb5b2d47337db1976f2edefd8e6b5a1cc95160f49a7d624731a737ce1e1bc` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,903 | `b9269901e5dd69b5785fd32b2db19c5618eacc338673c13ef2f5afa9efc66446` |
| `docs/SUPPORT_MATRIX.md` | 2,536 | `80f3578fc7528d14c362490988384e62f33f4125db90103d9d7e3b82f08bc4ff` |
