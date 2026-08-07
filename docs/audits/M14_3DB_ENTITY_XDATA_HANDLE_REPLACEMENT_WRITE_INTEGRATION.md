# M14.3db Entity XDATA Handle Replacement Write Integration

Retrieved: 2026-08-07

## Contract

`DxfEntityXDataHandleReplacementTransactionPlan::write_reparse_verify_and_journal_to_new_file`
checks the exact M14.3cy source, destination, and set binding before creating an
output. It then uses the shared M12 create-new transaction writer to stream,
hash, flush, sync, and independently reopen the staged post-image in Strict
mode for the transaction's exact ASCII or Binary format.

The M14.3da verifier must prove every unchanged and replaced byte, reparse each
group-1005 target, and release an executable inverse. The write journal binds
the M12 source/output identity and patch count to the replacement destination,
post-image identity, and replacement count. Existing paths are never modified;
every error or unavailable result after file creation attempts to remove the
output, with cleanup failure replacing the primary result.

## Verification boundary

The focused six-test suite covers create-new writes for all nine Core dialects
in both same-format ASCII and Binary. It checks monotonic write progress,
source/destination/post-image receipts, patch/replacement counts, non-disclosing
debug output, strict reopen, and byte-identical inverse restoration. Foreign
set evidence, source mismatch, an existing destination, observer and token
cancellation, strict-reparse failure, and valid raw tampering fail closed
without leaving a partial output.

This checkpoint writes a verified staged source clone. It does not insert an
entity into the separately parsed destination document, implement
cross-container cloning, interpret application-specific XDATA payloads, or
advance POINT to `Complete`.

## Gate receipts

The focused suite passed 6/6 tests; the full workspace passed all 1,007 tests
across 191 targets. Schema and release-evidence checks, cargo-deny advisories,
bans, licenses, and sources, formatting, workspace Clippy with warnings denied,
the forbidden production scan, and `git diff --check` passed. No dependency,
lockfile, generated schema, fixture, corpus, or release artifact changed.
Production changed by 222 insertions and 10 deletions across the verification
module and public export. This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 151 | `e991b79dfb1668054b52f1a256e8a6f3c6bb116f17f54435efe5d3181f27b2e1` |
| `README.vi.md` | 150 | `8d285bf4e57d60a00b33c15bb9e52d6a3232670e30d8f4b5aa22be823eb04917` |
| `crates/seacad-dxf-core/src/entity_xdata_handle_replacement_verification.rs` | 421 | `1e38b13f0b70541ee8080d525dccddcbf7fa262cebf211045c450d322b75b3cd` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,156 | `33a26deb45397c19aae89a2cfd0ef722ecdcd08daf388fb6f12c632868c14d84` |
| `crates/seacad-dxf-core/tests/entity_xdata_handle_replacement_tests.rs` | 985 | `c16081a354b96ca85326360b956c3a0c247d9d209f736be67836d679942c200c` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,534 | `03e0a4f3e776e71acea08913b8e2629e21e5896d148e2868a1a4e1c81f74775e` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,961 | `3b64d2a6a82b7d04f04f098d1db8930a799616f09f24ad7ccb558115f4c392b4` |
| `docs/SUPPORT_MATRIX.md` | 2,587 | `5868b325424d938f9f91f4f9571c2cbfb15365c6d16b4da8f52bb931ae9703f6` |
