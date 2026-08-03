# M14.3cx Entity XDATA Handle Replacement Patch

Retrieved: 2026-08-04

## Architectural basis

M14.3cw proves exact destination-dialect bytes but does not by itself identify
the complete source group those bytes supersede. Transaction or clone assembly
must not infer that association from a caller-provided ordinal. This checkpoint
therefore traces only owned M14.3cv/M14.3cu/M14.3ct evidence back to the typed
group-1005 occurrence.

## Contract

`DxfEntityXDataHandleReplacementPatch` is available only for an M14.3cw `Ready`
entry. It binds the replacement ordinal, exact `DxfRawGroup` with its complete
source span, mapped target handle, source identity, and destination identity.
The directory verifies that the traced group code is exactly 1005.

`replacement_bytes_for_patch` re-derives the patch from the directory-owned
entry and compares the complete value before returning bytes. A structurally
similar patch from another source or destination fails closed. Missing,
ambiguous, invalid, unmapped, and otherwise unavailable entries expose neither
a patch nor bytes. Payload bytes remain omitted from directory debug output.

## Verification boundary

The existing focused three-test suite now verifies exact source full-span
recovery, target and ordinal binding, byte lookup through the patch, absence of
patches for unavailable states, and foreign patch rejection. Those assertions
run inside the established ASCII/Binary source/destination matrix across all
nine Core dialects, with cancellation and metadata bounds retained.

This checkpoint does not compose a transaction, write or clone XDATA,
interpret application-specific payloads, or advance any entity to `Complete`.

## Gate receipts

The focused replacement suite passed 3/3 tests. The full workspace passed all
1,004 listed tests across 191 targets. Generated schema and release-evidence
checks, `cargo deny --locked check`, formatting, workspace Clippy with warnings
denied, the production forbidden-construct scan, and `git diff --check` all
passed. No manifest, lockfile, dependency, generated schema, locale source,
committed fixture, or external corpus changed.

Production adds one 80-line public binding and lookup contract inside the
existing bounded module plus one root export adjustment. The focused target
adds 36 net lines. This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 495 | `c1f2e501173d8e99615dc152c2af005e6b05c15a4c35d18a9b4dd8bc3090764a` |
| `crates/seacad-dxf-core/src/entity_xdata_handle_replacement.rs` | 409 | `1a2673fe996fb4f28d1bdc07a8e76a824a0e00cf4787dcb672d5999c50f40330` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,137 | `95d6f02c789bc1e814d2591deda4eb042ee1ec0491ad025c2c018a48eb7b388d` |
| `crates/seacad-dxf-core/tests/entity_xdata_handle_replacement_tests.rs` | 378 | `6aae145330fbaf925b81fe5a7f1696f0fdccd4f625d06400b821c6b0348f1180` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,493 | `4fcd354d265c5c18ec9b8717b50906f48794409ed29485b5906b2834f73efffb` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,914 | `d2587da1ca961557e47296f5bbd8f74fc1c0970f223aa82128aa0f76be30a5d7` |
| `docs/SUPPORT_MATRIX.md` | 2,545 | `826ad7f718882dc0cac61ce0e593ce27dc4061022a0d56f7c23331102c56676f` |
