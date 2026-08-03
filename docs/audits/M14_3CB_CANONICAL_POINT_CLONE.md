# M14.3cb Canonical POINT Clone

Retrieved: 2026-08-03

## Scope

M14.3cb adds the first semantic whole-entity clone operation to the unified
entity edit session. It reconstructs one admitted canonical POINT into a fresh
record and fresh handle while preserving every POINT payload field that the
current typed surface can represent. It does not copy unsupported raw groups,
cross entity-container boundaries, change modern owner identity, clone another
entity family, or advance POINT to `Complete`.

## Prior evidence

- M14.3i through M14.3r establish exact entity identity, placement, owner, and
  handle evidence.
- M14.3bk through M14.3bp establish canonical POINT drafting and insertion,
  deterministic handle allocation, owner binding, `$HANDSEED` maintenance,
  strict semantic verification, and executable inverse restoration.
- M14.3bq through M14.3bz establish public POINT field editing and mixed
  insert/update session composition.
- M14.3ca establishes reference-safe whole-record deletion.

No legacy or external parser code, fixture, data, dependency, generated
artifact, or private corpus content was copied, translated, vendored, linked,
or used at runtime.

## Contract

- `DxfEntityEditSession::clone_entity` accepts a source-bound POINT key, a
  target placement, and an owner handle.
- The source must be a canonical POINT whose record contains only the admitted
  envelope groups `0`, `5`, `8`, `10`, `20`, `30`, `39`, `50`, `100`, `210`,
  `220`, `230`, `330`, `370`, and `410` as applicable to its dialect.
- Exact layer and layout bytes, applicable lineweight, location, thickness,
  extrusion, and UCS X-axis angle are reconstructed through `DxfPointDraft`.
  Optional POINT payload retains its explicit-versus-defaulted state.
- The clone remains in the source entity container. For AC1012 and newer, the
  requested owner must equal the source owner exactly.
- The established insert path allocates a fresh handle, binds placement and
  owner, advances `$HANDSEED`, reparses the post-image, verifies typed POINT
  semantics, and produces an exact executable inverse.
- A queued source update is rejected so the clone cannot ambiguously select
  between source-preimage and pending semantic state.
- Missing, partial, duplicated, invalid, unsupported, or unrepresentable source
  evidence fails closed with a typed `DxfEntityCloneIssue`. Nested insertion
  failures remain typed rather than being flattened.

## Verification boundary

Paired in-memory fixtures exercise ASCII and Binary AC1009 through AC1032. The
defaulted-payload matrix covers all 18 dialect/encoding pairs. A modern AC1032
fixture independently proves preservation of explicit thickness, extrusion,
and UCS X-axis angle together with exact common fields. Negative coverage
rejects an unsupported common group, partial extrusion, and a queued source
update. Existing insertion and POINT edit suites continue to cover fresh
handle identity, placement/owner binding, strict semantic postconditions,
`$HANDSEED`, cancellation, and byte-identical inverse restoration.

## Nonclaims

This checkpoint does not copy XDATA, application groups, unknown common fields,
or entity-family-specific data outside the admitted POINT envelope. It does not
perform cross-container or cross-owner cloning, clone handleless records, mix or
batch deletes, calculate arbitrary graph closure, render POINT display
behavior, or claim complete entity editing.

## Verification

The focused entity insert session suite passed 9/9 tests and the existing POINT
edit suite passed 38/38 tests. The full workspace passed 953/953 tests,
including all 17 locale tests. Generated-schema and release-evidence checks,
`cargo deny check`, formatting, workspace Clippy with warnings denied, the
production forbidden-construct scan, and `git diff --check` all passed.
Production changes are 493 insertions and 10 deletions; focused test changes
are 214 insertions and 9 deletions. No manifest, lockfile, dependency,
committed fixture, generated schema, locale source, or external corpus changed.
This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 357 | `e3744dfa8740bf08b6710700d4784b409923ed34080f4dcc1355131050585950` |
| `crates/seacad-dxf-core/src/entity_edit_session.rs` | 2,402 | `d8daa5e6447d8f3ee29898ffdb8dc7558bc2930e19b23dee416031d6dfc17ea7` |
| `crates/seacad-dxf-core/src/entity_edit_verification.rs` | 1,200 | `e924044499ac0f7514ed93b4535f803ff7c2d29efde17882e82f2f7454361bfb` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,067 | `ed1594613135774715dfe0d318c9609ce67cb42a9fbd5251b4f79ab798247138` |
| `crates/seacad-dxf-core/src/point_edit.rs` | 1,084 | `bcef8c022f2c17e5bf672a06692a9c28787ef7e56fbc24194c0ab44a33c46e51` |
| `crates/seacad-dxf-core/tests/entity_insert_session_tests.rs` | 1,027 | `a58c0c0b979a176093ec5f15716c0d512d54d9f83c9592d60c2235328bfe69d8` |
| `crates/seacad-dxf-core/tests/point_edit_session_tests.rs` | 3,005 | `286656da41a43692d73dfe3fe3280f7f7f4ef331b40971941e7746bff6fc3d61` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,235 | `78cbb6deeef4be78e6c4c6671dbdc49fdd30dd92c5a1f199d0892d97552c6058` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,663 | `a844cfd86e3dd576aa9065b9af1ca829063de95dae2992dda492cc8bbe0c5092` |
| `docs/SUPPORT_MATRIX.md` | 2,314 | `c4c5d4e7f4174cdb56d64ff78ec0eb502e3e75f919697bd81217fa08c8252b9f` |
