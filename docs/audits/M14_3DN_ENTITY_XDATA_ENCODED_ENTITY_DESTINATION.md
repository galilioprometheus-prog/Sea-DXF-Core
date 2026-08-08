# M14.3dn Entity XDATA Encoded Entity Destinations

Retrieved: 2026-08-08

## Contract

`DxfEntityXDataEncodedEntityDestinationDirectory` owns M14.3dm and composes its
exact application sets into one canonical destination-byte payload per indexed
source entity. Ready requires the M14.3dj payload envelope and every owned
application set ready. Zero-XDATA entities remain explicit ready entries with
empty bytes.

Unavailable entries retain the complete payload state, application and member
totals, unavailable counts, first unavailable application ordinal, and all
underlying application evidence. Aggregate entity bytes are published only for
a wholly ready payload. No bytes are inserted and neither document is mutated.

## Verification boundary

Three focused tests cover all four ASCII/Binary source-destination pairings for
all nine Core dialects plus AC1009/AC1032 cross-dialect pairs. Exact checks
cover ready and zero-XDATA entities, failed-handle and orphan payloads,
empty/nested applications, aggregate counts and bytes, cancellation,
dual-source identity, compact bounds, and non-disclosing debug output.

Application-specific interpretation, actual text transcoding, insertion,
mutation, cross-container clone, and POINT `Complete` remain open.

## Gate receipts

Focused M14.3dn tests passed 3/3; the complete encoded regression suite passed
10/10. The workspace passed exactly 1,044 tests. Cargo-deny, formatting, schema
and release-evidence checks, workspace Clippy with warnings denied, production
safety scan, protected-surface diff, and `git diff --check` passed. No manifest,
dependency, lockfile, schema, corpus, legal, or release surface changed.

Production adds 419 lines across the encoded-entity directory and public
exports; focused tests add 253 lines. This audit intentionally omits its own
hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 158 | `745e1f69a18a893a509b2f5e6f9283c1f6c080dc62df7d1849f60f597e79f3a9` |
| `README.vi.md` | 156 | `ad0fcb7ff9fb3c388647ccfd2557470e006b9d2d675638a839ee41287bb8b139` |
| `crates/seacad-dxf-core/src/entity_xdata_encoded_entity_destination.rs` | 414 | `8ad238f45655582faa7a54d23d0ded371150c913d1fd3989a5c560bd4a9f9ad6` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,220 | `6d101f7edb8c04e42c3599531e658fdf96b4df92b52f6a3fc4dc1d6059670a13` |
| `crates/seacad-dxf-core/tests/entity_xdata_encoded_destination_tests.rs` | 1,225 | `198a4f2be745da5e71ac51e543a544d3720ff82a6f94623c293d88d9d1fe3afd` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `5c628f493d19203eb09b7ce2587808c09350bd84f8904cc0d62e4270acc454ba` |
| `docs/SUPPORT_MATRIX.md` | 2,762 | `fe46277b9bb753e9780c43520bfcf4d9a2314cd548a8e6901a12f936602dba27` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 1,704 | `567cec13a589b539d8d1eae75cf8ba55de52093f784cdbc5d3f972e944d1c5c5` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,126 | `2610254e8482db999361bcaa35ee22302e42f15f196fa1980f2bd66e31a935b5` |
