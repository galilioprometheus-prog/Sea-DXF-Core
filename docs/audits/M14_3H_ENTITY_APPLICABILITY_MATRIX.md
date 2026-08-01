# M14.3h Entity Applicability Matrix

## Scope

M14.3h adds a deterministic applicability descriptor for each of the 45
canonical entity topics and 14 reviewed alias/specialization names. Every
descriptor evaluates all nine supported dialects from AC1009 through AC1032.

## Evidence boundary

Autodesk compatibility guidance states that DWF underlays are unsupported
before AutoCAD 2007, DGN underlays before AutoCAD 2008, and PDF underlays
before AutoCAD 2010. AC1021 is the shared AutoCAD 2007--2009 file dialect, so
the DWF and DGN wire names use an AC1021 minimum; PDF uses AC1024. The source is
`GUID-BF215599-C96C-4FFF-A2DB-21DEFFAC71C0`, and the normalized three-row facts
receipt is
`1f8904aba0b1891d9d7faf3c0d3a9ce7a1d99ec815d268f60c7425c69b7cdaea`.

The read-only legacy ODA matrix and public open-source corpora were inspected
as behavioral evidence. Their absence of a name in a converted fixture is not
proof of non-applicability, and conflicting third-party version floors were
not promoted into schema facts. No legacy or third-party implementation was
copied.

## Contract

- Registry order is exactly the 45 canonical topics followed by the 14 aliases.
- Each row is either an inclusive source-backed range or `NotYetReviewed`.
- A reviewed row exposes its minimum/maximum dialect, evidence kind, source
  id/reference, and normalized source-facts SHA-256.
- `DxfEntityNameClassification::applicability_descriptor` is allocation-free
  and preserves canonical versus alias identity.
- Unknown names have no descriptor. Known but unreviewed names return
  `NotYetReviewed` for every supported dialect.
- Generator validation rejects row count/order drift, unsupported or inverted
  ranges, metadata on unreviewed rows, missing/wrong-kind sources, stale source
  receipts, and stale generated Rust.
- The generated applicability registry receipt is
  `d6adb21246d5fcca917399bd89969c173744a3b17fd8e6669dc9d445c7aee963`.

## Nonclaims

This checkpoint does not infer the introduction version of the remaining 56
names. It does not scan raw records, classify wrong-section occurrences, gate
insert/update/write, add subclass or field applicability, expose common entity
properties, parse semantics, construct geometry, add CRUD, or change any
entity support status.

## Verification

Focused generator and public API tests cover the 59-by-nine matrix, exact
classification-to-row lookup, underlay boundaries, unknown names, and explicit
unreviewed behavior. The generator suite passed 18/18 and the public
entity-schema target passed 7/7. The complete workspace passed 740 tests.
Schema freshness, formatting, workspace Clippy with warnings denied,
dependency policy, Cargo.lock identity, and `git diff --check` passed.

The first dependency-policy attempt could not acquire the advisory database
lock inside the read-only sandbox; the approved rerun passed advisories, bans,
licenses, and sources. The first Clippy run rejected the eight-argument render
function and three test `expect` calls; receipts were bundled into a typed
input and tests now propagate failure, after which Clippy passed. The Agent Hub
handshake task remained unclaimed, so no Antigravity result was used as
evidence. Codex ran and verified every gate directly.

Handwritten generator production added 488 lines and removed 23; the added
production remains within the 200--500-line review target. Generated Rust added
199 lines, and the applicability registry is 65 lines. This audit omits its own
hash so its receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 74 | `d5d57c07f1e9939172566a49667a6040cd8b86d2268321fc9bc481c2892692f6` |
| `crates/seacad-dxf-core/src/generated/entity_schema.rs` | 988 | `8278551cbb81dba392cc8d5d7ce49ae9139d91254579470d6a575e4f6f29146c` |
| `crates/seacad-dxf-core/src/generated/header_schema.rs` | 2,387 | `204c7fba461f41e9bb69cae0e16c032551e135a486d1573b3b4c19203b77ebc4` |
| `crates/seacad-dxf-core/src/lib.rs` | 842 | `f4bd2f75567264833ac2eeddaf9ac599a0977be2bdd89af8dc71cc23c9a672d3` |
| `crates/seacad-dxf-core/tests/entity_schema_tests.rs` | 271 | `d50c90e3c624bf0eea5ad897ca82ff7dc4eff1a26e487122a3ef865d830e5492` |
| `crates/seacad-schema-gen/src/main.rs` | 2,330 | `6dbe3d734a3639357ea449053aeefdb564ace227323fc66c437697c14a49d52a` |
| `schema/dxf/v1/entity_applicability.json` | 65 | `3a7ad7daddef8757485cb576cb716a7104b699eeb6524832bf987463f6e865a4` |
| `schema/dxf/v1/manifest.json` | 10 | `cd532755b29b8785e9766bcca296b6eb0bd77acffcc71da057d5822884648d58` |
| `schema/dxf/v1/sources.json` | 47 | `7a7f1946e50219cf0f7645adcd0e7306f4f3b3dcfcaaf71cbe7f1ec1a7be0793` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 347 | `299ea099337e6e83df2ab8c554b11ffb3394a738d87dba391637f0862d483793` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,748 | `98ddf673948f04cee59896f92356c0754705002d2be2c7aa6e65cbf1bbf550f1` |
| `docs/SUPPORT_MATRIX.md` | 1,382 | `b0bb0fe10fec6ffa600bcf0d28ea5e2bf4063b7a38a21e4bdd6276b3d07b890a` |
