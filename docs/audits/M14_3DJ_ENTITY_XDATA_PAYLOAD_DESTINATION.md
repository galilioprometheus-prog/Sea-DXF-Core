# M14.3dj Entity XDATA Payload-Envelope Destination Readiness

Retrieved: 2026-08-08

## Contract

`DxfEntityXDataPayloadDestinationDirectory` owns M14.3di handle-composed
destination readiness and reuses the exact typed occurrence/application
evidence already owned by its coordinate chain. It publishes one compact,
dual-source-bound entry per indexed source entity. Payload-value counts exclude
group-1001 application names, and orphan XDATA values receive a direct exact
count.

Ready requires the complete M14.3di state ready and zero orphan values.
Unavailable retains the complete handle-composed state plus total payload and
orphan counts. Callers can recover exact typed and application slices without
rebuilding another source directory. The reusable typed directory also adds an
exact source-identity-checked per-entity slice.

## Verification boundary

Three focused tests cover all four ASCII/Binary source-destination pairings for
all nine Core dialects plus AC1009/AC1032 cross-dialect pairs. Coverage includes
an enclosed application with text, tuple, and remapped handle values; a direct
orphan classification; handle and coordinate failures without orphans; empty
payloads; exact typed/application slices; foreign source/destination rejection;
cancellation; compact metadata; bounds; and non-disclosing debug output.

This checkpoint establishes payload-envelope readiness only. It does not
interpret application-specific meaning, project each logical value into its
destination representation, encode or insert XDATA, mutate either document,
complete a cross-container clone, or advance POINT or another entity to
`Complete`.

## Gate receipts

Focused payload tests passed 3/3; capacity/handle-composition/payload regression
suites passed 9/9. The workspace passed exactly 1,031 tests. Cargo-deny,
formatting, schema and release-evidence checks, workspace Clippy with warnings
denied, production safety scan, protected-surface diff, and `git diff --check`
passed. The complete gate run finished in about 279 seconds. No manifest,
dependency, lockfile, schema, corpus, legal, or release surface changed.

Production adds 368 lines across typed entity slicing, payload-envelope
composition, and public exports; focused tests add 498 lines. This audit
intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 155 | `27861aa017c6e78eda4917717d34f7401c00b9283f91ba2002e5258410f9ba57` |
| `README.vi.md` | 154 | `56820c860b9779d3a18eae40d660bc57fe2c728793dc0327386b41395b2501b3` |
| `crates/seacad-dxf-core/src/entity_xdata_value.rs` | 531 | `53bba08f8303b9f7f8dbf3e5b21bcdced8c266e5b9c2e502ec2b01d73ae5f63d` |
| `crates/seacad-dxf-core/src/entity_xdata_payload_destination.rs` | 334 | `f2de80d95679d457d589901784c59bba006d4cc9b0c8f3e3a8a76967e9ee4777` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,198 | `89766745ec427bcdbe14d975d48aafb15f719000607fec0f74f96715ee8cf384` |
| `crates/seacad-dxf-core/tests/entity_xdata_payload_destination_tests.rs` | 498 | `6b8f0457754162ba5b484fa3e8939a350cf3337328eaa99e1a95e34c54661305` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `485b47cc5883bd92b34ca49d8f63635aa020145a2427631adbeb84fd9eb89832` |
| `docs/SUPPORT_MATRIX.md` | 2,701 | `2f5221cd1f517ec8b123ee8bd13896f0b1936e60c19fbf9724f2be1f2e561441` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,070 | `ab032a89900bd85991d560935d60d76e9d888d0666731be3431867aa18a68905` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 1,648 | `94d43a791de32b19fb7b896775f5f91e6616d363090bbc72fca1a30ca8e63984` |
