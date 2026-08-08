# M14.3dg Entity XDATA Entity Destination Readiness

Retrieved: 2026-08-08

## Contract

`DxfEntityXDataEntityDestinationDirectory` owns the M14.3df application
destination directory and M14.3cr capacity directory. It publishes one compact
entry for every indexed source entity, including records with no XDATA. Ready
requires every application ready and exact source capacity within the 16,383-
byte limit.

Unavailable entries retain total and unavailable application counts plus the
unchanged `WithinLimit`, `Exceeded`, or `Indeterminate` capacity state. Exact
application and capacity entries/issues remain derived from the owned
directories. Destination-only failure therefore remains distinguishable from
source structural/capacity failure. Construction is cancellation-aware,
allocation is fallible, dual-source identity and compact lookup are enforced,
and `Debug` discloses no symbol or payload bytes.

## Verification boundary

The focused three-test suite covers all four ASCII/Binary source-destination
pairings for all nine Core dialects plus AC1009/AC1032 cross-dialect pairs. It
proves ready within-limit entities, destination-only unavailable state with
exact capacity, structural indeterminacy, independently exceeded capacity,
zero-XDATA indexed records, owned entity/application/capacity lookup,
dual-source rejection, cancellation, bounds, and debug redaction. The 70
bounded group-1000 values exercise exact capacity excess without an oversized
physical group.

This checkpoint does not compose coordinate transforms, group-1005 handle
remaps, payload semantics, encoding/insertion, destination mutation, or cross-
container cloning, and does not advance POINT or another entity to `Complete`.

## Gate receipts

Focused tests passed 3/3 and adjacent application/capacity/entity suites passed
9/9. The workspace passed 1,022 tests with zero failures or ignored tests.
Cargo-deny, formatting, generated schema and release-evidence checks, workspace
Clippy with warnings denied, production safety scan, Markdown links, and
`git diff --check` passed. No dependency, lockfile, schema, corpus, legal, or
release artifact changed.

Production adds 325 lines across the new module and exports; focused tests add
475 lines. This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 155 | `21b281752d54d37f6c5bb4cc7865d55e56a8cf1b5e690eb4a136203189974622` |
| `README.vi.md` | 154 | `73bbebc1ddcf8d3252b402fb0694e478bbee043f8be5a356fc6429f68a82122b` |
| `crates/seacad-dxf-core/src/entity_xdata_entity_destination.rs` | 320 | `67e0c54fe8c7998f302bfe681bddf092b561f6cb5b783440573a2ae4f8629df1` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,183 | `fcbf9efed5945d220138aef46e7a097df85947da4d86bba85aa2fdb66c9bd24e` |
| `crates/seacad-dxf-core/tests/entity_xdata_entity_destination_tests.rs` | 475 | `21a4697ffde052669d8d72613f35d02d494e9987beedeb05364fd0126062873f` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `2e021a1b232b84610f1b6924a0f4882041244e550a997c58449fea2f05a0714c` |
| `docs/SUPPORT_MATRIX.md` | 2,661 | `d4df6b83e31093385b78af7d2e9560880937de05418fd86b94d3204c6ae7fad3` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,034 | `d7e0d186dc3e6063488a4a65982ec414864fa760b5b86a00738af191fbc181d4` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 1,610 | `e61ae64fbb657882651956da9a6f422d130ed676d3e7c3a50bfd49572ca87e97` |
