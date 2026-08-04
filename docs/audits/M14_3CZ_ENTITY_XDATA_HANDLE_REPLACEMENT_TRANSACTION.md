# M14.3cz Entity XDATA Handle Replacement Transaction

Retrieved: 2026-08-04

## Contract

`plan_entity_xdata_handle_replacement_set` converts one complete M14.3cy ready
set into an immutable source-bound staging transaction only when source and
destination format and supported `$ACADVER` state match exactly. Every member
replaces its complete M14.3cx source group with M14.3cw canonical destination
bytes. The shared transaction builder preserves source ordering, conflict
checks, resource bounds, and exact inverse capture.

Unknown or unavailable sets, format mismatch, dialect mismatch, foreign source
identity, and cancellation fail before a transaction escapes. The plan retains
both document identities and the admitted set. This is an intermediate staging
primitive; it does not claim that the transformed source is a verified
destination clone.

## Verification boundary

The focused suite covers same-format ASCII and Binary transaction creation plus
all cross-format rejection for every Core dialect. It checks typed unavailable
set rejection, source preconditions, canonical replacement bytes, exact original
group inverse bytes, source/destination identities, and set binding.

## Gate receipts

The focused suite passed 3/3 tests and the full workspace passed all 1,004 tests
across 191 targets. Schema and release-evidence checks, cargo-deny, formatting,
workspace Clippy with warnings denied, the forbidden production construct scan,
and `git diff --check` passed. No dependency or generated artifact changed.

This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 509 | `3955be85667957fd8c0b555d56b2c1b8123e343276516732c557ed0956fb1ebb` |
| `crates/seacad-dxf-core/src/entity_xdata_handle_replacement_transaction.rs` | 192 | `0ec9534d52a4f6afd183a220504b7e8eb7d255b68405e14e8f8ec5bec4fe3c8b` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,148 | `4d648cdbdd53f7f68dbea83996591fca93ed4af3d2dc245d8576e3f19db11f0c` |
| `crates/seacad-dxf-core/tests/entity_xdata_handle_replacement_tests.rs` | 529 | `63ec5bd425ccc2eca49c1bb752ef165ffc0e7f27a8cb7059cb8b7b0efe87630c` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,513 | `ea83c176b2065e92f84b4d64a586710058a73b964f67c91419401759ead57bdc` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,937 | `67930fa81dce54841fb513183b543d368869bdd2e55534cadefc1bd4eb04e8a6` |
| `docs/SUPPORT_MATRIX.md` | 2,565 | `e5be62a06c8808aa68fd986c948239fda6974351b0ac54c483cd5254d4eadd97` |
