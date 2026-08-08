# M14.3di Entity XDATA Handle-Composed Destination Readiness

Retrieved: 2026-08-08

## Contract

`DxfEntityXDataHandleComposedDestinationDirectory` owns M14.3dh coordinate
readiness and M14.3cv destination-validated group-1005 handle evidence created
from caller-supplied remaps. It publishes one compact source/destination-bound
entry per indexed source entity. Ready requires the complete coordinate state
ready and every group-1005 occurrence for that entity uniquely present in the
destination identity index. A coordinate-ready entity with no group-1005
occurrences remains ready.

Unavailable entries retain the complete coordinate state plus total and
unavailable handle counts. Exact invalid, null, source missing/ambiguous,
unmapped, ambiguous mapping, destination missing, and destination ambiguous
states remain derived from the owned handle directory. The handle directory now
also exposes exact entity slicing and source-entity recovery while revalidating
the full typed/resolution/remap chain.

## Verification boundary

Three focused tests cover all four ASCII/Binary source-destination pairings for
all nine Core dialects plus AC1009/AC1032 cross-dialect pairs. Coverage keeps
coordinate readiness independent from destination-missing, destination-
ambiguous, and unmapped handle failures; also covers a coordinate failure with
a unique handle, zero-handle readiness, base destination failure, owned lookup,
dual-source foreign rejection, cancellation, compact metadata, bounds, and
non-disclosing debug output.

This checkpoint does not interpret application-specific payload semantics,
encode or insert XDATA, mutate either parsed document, complete a cross-
container clone, or advance POINT or another entity to `Complete`.

## Gate receipts

Focused composition tests passed 3/3; coordinate/handle/composition regression
suites passed 9/9. The workspace passed exactly 1,028 tests. Cargo-deny,
formatting, schema and release-evidence checks, workspace Clippy with warnings
denied, production safety scan, protected-surface diff, and `git diff --check`
passed. The initial combined gate command reached its 120-second shell timeout
after Clippy while workspace tests were running; a separate 300-second rerun
completed successfully in about 170 seconds. No manifest, dependency,
lockfile, schema, corpus, legal, or release surface changed.

Production adds 368 lines across entity slicing, the composition module, and
public exports; focused tests add 562 lines. This audit intentionally omits its
own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 155 | `fe599b817d6548c877e89fd6368485c001c3d54421854ebb05de8a6b1ea304f0` |
| `README.vi.md` | 154 | `c0421f63c25f1267a778dcdbbdeac161fb501d085427cd860e9c7a9bfec7925a` |
| `crates/seacad-dxf-core/src/entity_xdata_handle_destination.rs` | 312 | `ab269305667247c88e72a7be5fe497fc986d489b8d50c65cfbc19d6d71cfb59c` |
| `crates/seacad-dxf-core/src/entity_xdata_handle_composed_destination.rs` | 317 | `283fbb1ae7f7292315ad1b79057e295b5bbdf0a8935b331d4da90c89374d154f` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,193 | `d26076f570aa1dfc35ceb89bf351547376ca9ec7dc87aa32bb2c68fe15ac9a6d` |
| `crates/seacad-dxf-core/tests/entity_xdata_handle_composed_destination_tests.rs` | 562 | `e0bf6216cfa9d1cc9b482b1fe1ed31427b1f2101e4bf58e89ca31f4ef1b9245f` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `97283f71168a176fd5537a90777249ed781192b3fb2b040b584eee2129457eab` |
| `docs/SUPPORT_MATRIX.md` | 2,687 | `f19552f529fc5d5840f6210d03a99d00604ad785ef1f560491cd9da96eea61d7` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,057 | `b4a92802c38ca16eb33e5694ce45a78232cb3826878a5532753c75e0435da153` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 1,634 | `594ff18b95c3821b00a28ca9e47aa1e6d7bf931535b5d898445eb1c24c4ed10c` |
