# M14.3dh Entity XDATA Coordinate Destination Readiness

Retrieved: 2026-08-08

## Contract

`DxfEntityXDataCoordinateDestinationDirectory` owns M14.3dg entity destination
readiness and M14.3cs affine transformed-point evidence for one caller-validated
transform. It publishes one entry per indexed entity. Ready requires the base
entity ready and every entity-bound tuple transformed successfully; entities
with no tuples remain ready.

Unavailable entries retain the complete entity state plus total and unavailable
tuple counts. Exact `PartialTuple`, `InvalidComponent`, and
`NonFiniteDerivedPoint` reasons remain derived from the owned transform
directory. Source/destination identities, entity and tuple slicing,
cancellation, fallible allocation, compact metadata, and debug redaction remain
explicit.

## Verification boundary

Three focused tests cover all four ASCII/Binary source-destination pairings for
all nine Core dialects plus AC1009/AC1032 cross-dialect pairs. One extreme
translation proves role-specific behavior: group-1010 Point remains unchanged
and ready while group-1011 WorldPosition fails typed non-finite derivation.
Coverage also includes destination-only failure without tuple issues, partial
tuples, zero-tuple indexed records, owned lookups, foreign source/destination,
cancellation, bounds, and non-disclosing debug output.

This checkpoint does not compose group-1005 handle remaps, application payload
semantics, encoding/insertion, destination mutation, or cross-container clone,
and does not advance POINT or another entity to `Complete`.

## Gate receipts

Focused tests passed 3/3; coordinate-transform/entity-destination/composition
suites passed 9/9. The workspace passed 1,025 tests. Cargo-deny, formatting,
schema and release-evidence checks, workspace Clippy with warnings denied,
production safety scan, and `git diff --check` passed. No manifest, dependency,
lockfile, schema, corpus, legal, or release surface changed.

Production adds 317 lines across the module and exports; focused tests add 457
lines. This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 155 | `70c614d3d3fa83f0411b950502bcafe803413bbb01ca1bc1c1d5036ab3ef7037` |
| `README.vi.md` | 154 | `e740bc7547b556de1c5a5b32dd72911d807496ecbdabc45db2668dc1446e5174` |
| `crates/seacad-dxf-core/src/entity_xdata_coordinate_destination.rs` | 312 | `063576469eb7b093a9129b74f22a38c703229dcd473c43d5188f2226c5130118` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,188 | `acc67765569ce1ab4f864986f6fc535d10c35298ae781aa00616125cafc9e9bf` |
| `crates/seacad-dxf-core/tests/entity_xdata_coordinate_destination_tests.rs` | 457 | `ec21af152e6ef2c54b070d0b0a2d7827cae2c8c7f58a7a6ccaf866b5a59f8ec8` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `e40fe949c33744a4d5c746ac61547fce7215d430417411caea7c4090f150229a` |
| `docs/SUPPORT_MATRIX.md` | 2,673 | `b75193dec9e44b433f55f7beb19632982b7234727f051bfda113efd4af93b6e5` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,044 | `7b7d3d351104fc10912d0d9011a1a97e0d6ebb5434f3c3624985ce8b36fbc24a` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 1,621 | `28783c41f89f4977847eef584efc10be43d8a502f93f3841d1b81d7b11e0d5fd` |
