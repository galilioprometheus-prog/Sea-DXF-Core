# M14.3dq Entity XDATA Draft Post-Image Verification

Retrieved: 2026-08-08

## Contract

M14.3dq strictly verifies M14.3dp post-images. Existing family verification
must first prove exact transaction bytes, typed POINT postconditions, and an
executable inverse. The XDATA layer then uniquely resolves the inserted handle
in the strict-reparsed document, locates its indexed entity, rejects orphan
values, reads the complete contiguous raw XDATA span under the resource
profile, and compares it byte-for-byte with the retained expectation.

The receipt binds source, destination and post-image identities, inserted
handle, application count, and XDATA byte count. The journal retains family
verification and the inverse. No receipt is published from partial evidence.

## Verification boundary

Six focused draft/XDATA tests cover all four ASCII/Binary source-destination
pairings for all nine Core dialects plus AC1009/AC1032 cross-dialect pairs.
Exact checks cover non-empty and zero-XDATA payloads, parsed application shape,
raw bytes, inverse restoration, cancellation, foreign pre-images, tamper
rejection, compact public metadata, and non-disclosing debug output.

Create-new write cleanup under this XDATA wrapper, application-specific
interpretation, actual text transcoding, cross-container clone completion, and
POINT `Complete` remain open.

## Gate receipts

Focused draft/XDATA tests passed 6/6 and adjacent suites passed 36/36. The
workspace passed exactly 1,050 tests. Cargo-deny, formatting, schema and release-
evidence checks, workspace Clippy with warnings denied, production safety scan,
protected-surface diff, Markdown links, and `git diff --check` passed. No
manifest, dependency, lockfile, schema, corpus, legal, or release surface
changed.

Production adds 318 lines across insertion metadata, strict XDATA verification,
and public exports; focused tests add 159 lines. This audit intentionally omits
its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 159 | `3f8638d5d96a5712a25f08b84b4ccd3f2fe870bd43ad79e3a98ec62905b674a9` |
| `README.vi.md` | 158 | `c0dc5e46adccab523e2f34803e70cb889d3c759312bbcad5b5d1e96c3b7ce9ca` |
| `crates/seacad-dxf-core/src/entity_xdata_draft_insert.rs` | 190 | `f25a0fdc706488bc7dbb8d280ea24442f04912e987eb0bc63d403b335f2a8743` |
| `crates/seacad-dxf-core/src/entity_xdata_draft_verification.rs` | 304 | `9a8bc8e190bdbc8cc08ee473b4a1cbfd641014b6d9922b960177545742dbf238` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,231 | `3d194e0789e4b9c9e308778a8dec66ab994837b87d2dd1d5f4b69cea60d743f5` |
| `crates/seacad-dxf-core/tests/entity_xdata_draft_record_tests.rs` | 782 | `d024ce262e90aeeb56f1ad27ca0063d43c11a77008ce751880d96483b1d80b90` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `b07e66853ededd00a3576dffe18cc4494d860f1126f607212af693dc997c650e` |
| `docs/SUPPORT_MATRIX.md` | 2,805 | `458c685f5f82498a46b9cb9fa7648cc704879272d3165d096c80a51ef98bfe8c` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 1,746 | `404edef99613567fdaab638af78234153ebbdf8e398e583a6a2f76d101404bb4` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,168 | `88a7da5a508639410170b4761880590402f0c31dea8758b415edc678038cf95c` |
