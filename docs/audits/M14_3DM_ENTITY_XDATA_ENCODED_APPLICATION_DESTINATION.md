# M14.3dm Entity XDATA Encoded Application Destinations

Retrieved: 2026-08-08

## Contract

`DxfEntityXDataEncodedApplicationDestinationDirectory` owns M14.3dl and
composes its occurrence results into one exact source-order destination set per
source XDATA application. Ready requires the owning M14.3dj payload envelope
ready and canonical destination bytes for every application member. Exact
application ranges preserve the group-1001 name, following values, empty
applications, nested controls, and duplicate application identities while
excluding every orphan occurrence.

Unavailable sets retain the complete payload state, total member count,
unavailable member count, first unavailable member ordinal, exact application
evidence, and every underlying encoded result. Aggregate application bytes are
published only for a wholly ready set. This milestone does not insert bytes or
mutate either source or destination.

## Verification boundary

Four focused tests cover all four ASCII/Binary source-destination pairings for
all nine Core dialects plus AC1009/AC1032 cross-dialect pairs. Exact checks
cover source-order grouping, ready and unavailable siblings, payload-envelope
composition, aggregate byte counts, remapped-handle failure, non-ASCII decoder
mismatch, empty applications, nested controls, orphan exclusion, cancellation,
dual-source identity, compact bounds, and non-disclosing debug output.

Application-specific interpretation, actual text transcoding, per-entity
encoded aggregation, insertion, mutation, cross-container clone, and POINT
`Complete` remain open.

## Gate receipts

Focused M14.3dm tests passed 4/4; the complete encoded-destination regression
suite passed 7/7 and the adjacent encoder/handle/logical/application suites
passed 20/20. The workspace passed exactly 1,041 tests. Cargo-deny, formatting,
schema and release-evidence checks, workspace Clippy with warnings denied,
production safety scan, protected-surface diff, and `git diff --check` passed.
No manifest, dependency, lockfile, schema, corpus, legal, or release surface
changed.

Production adds 491 lines across the encoded-application directory, the
encoded-range helper, and public exports; focused tests add 376 lines. This
audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 157 | `d00841242127c76acadff6ede3087c6dc5b13f6e9c5dd0aef58e2952a63d32d2` |
| `README.vi.md` | 156 | `ce1f1dca97ace9c1568903f2fc1320b655a7911b7456e5184265b4d9602111e2` |
| `crates/seacad-dxf-core/src/entity_xdata_encoded_application_destination.rs` | 462 | `9b8d1dce1370fa2369c1c97296c3e8aa7e3f67d16a96b9ec1dbbf083f454784b` |
| `crates/seacad-dxf-core/src/entity_xdata_encoded_destination.rs` | 580 | `011b9f8771af6f62a652094283787990f48bbf32e248743a3a5d24ddaa4c9e33` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,215 | `7237a8af75a285a419dcac3887e6d4e62f84027ab816f2967385a4baef6cc55f` |
| `crates/seacad-dxf-core/tests/entity_xdata_encoded_destination_tests.rs` | 972 | `530a01c40da583606d4b35bba90751a37e78f1feb7b24b7269999889454dc920` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `43fb2d233bb694698f574d75f44cae1fb941c5f0f76a0ca736a01629e5856736` |
| `docs/SUPPORT_MATRIX.md` | 2,749 | `5e3b575cc1370a9b67f4c7d34c9bdd805c2d36088945f6d1b55e1ff883b5f4e6` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 1,691 | `44942461b0c11f3af95dc5f1fc65c055419389a0b8cde36a0c248286049faf52` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,113 | `959ecc6195a3b2da0f5ddb0ba8debae50e1575eae8ed16048cfa0e6e92a4c766` |
