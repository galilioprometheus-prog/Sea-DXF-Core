# DXF-STRUCT-R1c HATCH Edge-Card Core

DXF-STRUCT-R1c removes the duplicated cardinality storage and collection
mechanics from the HATCH boundary Line and CircularArc edge-card families. The
new `hatch_boundary_edge_card` module owns the shared card state, member range,
role-typed member, role-typed card, directory, and collector.

Each family still owns its exact role enum, fixed role order, group-code
mapping, edge-type filter, and ASCII/Binary/format-neutral document entry
points. Existing family-specific public names remain available as aliases. The
role marker keeps Line and CircularArc members, cards, and directories distinct;
only the domain-neutral cardinality state and member range are shared. Five
neutral generic types are exported for the shared contract.

No policy moves into the common collector. It selects no duplicate member and
applies no requiredness, defaults, numeric decoding, semantic validation, OCS
geometry, or WCS geometry. Raw bytes, exact source order, provenance, typed
issues, cancellation, source identity, bounds, compact traits, and debug
redaction remain unchanged.

The two focused card suites pass 8/8 tests and all ten Line/CircularArc
card-through-WCS suites pass 40/40. Workspace passes 1,213/1,213 tests and all
required gates pass. The first workspace invocation reached the command
harness's 120-second limit with no observed failure; the required rerun with a
600-second limit completed successfully in 190 seconds. No dependency,
license, fixture, schema, release, corpus, CI, support-matrix, or support-claim
change occurs. Production changes four files, adds 280 lines, deletes 472, and
therefore removes 192 net lines.

The repository batch note and cumulative external `Status: READY` packet are
synchronized through DXF-STRUCT-R1c at
`D:\SeaCad\AntigravityReports\seacad-m14.3dr-dxf-struct-r1c-hatch-edge-card-core-2026-08-10.yaml`.
No independent Antigravity PASS is claimed. This audit omits its own hash and
the self-referential active batch-note hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/hatch_boundary_edge_card.rs` | 239 | `09f49ce72b5e55e70474f213d171050610da468f81cb2ac748278c3e19b51f12` |
| `crates/seacad-dxf-core/src/hatch_boundary_line_edge_card.rs` | 76 | `8002fc740ac7eed56ddf4f4b47bfa392096ab9ecc2b05df1d6971beeca8f48d1` |
| `crates/seacad-dxf-core/src/hatch_boundary_circular_arc_edge_card.rs` | 84 | `f381609a40eee309ea725c44f94c4fc6e18e4dc9a417722f2fb81b7ec7330f33` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,464 | `815f43a2ceb6ef1027f6e885ab1eb5a9819a8d203f4e2f5bc78aaeda96b1b19a` |
| `docs/ARCHITECTURE.md` | 212 | `20c2364fb471153020c5842f1dbec3a985a0a76882cefa4889b00bb3785a8e67` |
| `docs/IMPLEMENTATION_PLAN.md` | 424 | `56496adf1bf160f94279409f70952cc65c2d5bf794a4fc533a0a5913e122016f` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,273 | `f634b490727919a078ff483e0c3166c96e9d73ce47a4b61c6a3c0ff2ddfbc750` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,698 | `dfab34878dd697a8e3c98981556b78688a2763d6cddf830f18e7cbd0d343d31b` |
