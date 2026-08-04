# M14.3da Entity XDATA Handle Replacement Verification

Retrieved: 2026-08-04

## Contract

`DxfEntityXDataHandleReplacementTransactionPlan::verify_post_image` first uses
the shared streaming transaction verifier to prove every unchanged and replaced
byte and materialize an executable inverse. It then reparses every staged source
occurrence as a handle and requires the exact M14.3cx destination target.

The verified journal retains a compact source/destination/post-image receipt,
replacement count, and exact inverse plan. Foreign set evidence, missing or
non-handle occurrences, invalid/wrong handles, cancellation, length/format
drift, and any byte tampering fail closed.

## Verification boundary

The focused three-test suite performs strict same-format ASCII/Binary post-image
verification for all nine Core dialects, checks receipt identities and count,
applies the inverse back to byte-identical source data, rejects cancellation,
and detects valid-handle tampering. Cross-format planning remains typed
unavailable under M14.3cz.

## Gate receipts

The focused suite passed 3/3 tests; the full workspace passed all 1,004 tests
across 191 targets. Schema/release checks, cargo-deny, formatting, workspace
Clippy, forbidden production scan, and `git diff --check` passed. This audit
intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 515 | `c5359fc5ca355d0ad21f906734e34c4c12e94746be7109873cd2c4aadb2c6a98` |
| `crates/seacad-dxf-core/src/entity_xdata_handle_replacement_verification.rs` | 210 | `f1ee466a10c83f0a5ab6661fb8734f1f5c3a31c2984007bf642628d9025e0ce1` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,155 | `88ce593c5b1381e53b0fffdd00c837d0a04e5d37b3e49f238d41482b59e96905` |
| `crates/seacad-dxf-core/tests/entity_xdata_handle_replacement_tests.rs` | 605 | `194e6ed726becebf33c2be0ef046b3f6685e682af4f22469347725039cdd76c5` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,522 | `c74cd1b745bb67be18f338d88b9efd98faef8a18fc32282365869cba4ee2e0c5` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,947 | `8c0e9f0a0c569c292760958e2d0dd514127d407ea0436859cead8e0e0b451b88` |
| `docs/SUPPORT_MATRIX.md` | 2,574 | `8aa3a5e78daca00d0013b77e7ad26c3e8b83399ca1aa5c491b6b832b46c51626` |
