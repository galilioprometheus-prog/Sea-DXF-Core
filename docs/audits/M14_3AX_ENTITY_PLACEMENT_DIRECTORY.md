# M14.3ax Entity Placement Directory

## Scope

M14.3ax adds one immutable, source-bound directory of insertion placements for
entity-bearing `ENTITIES` sections and BLOCK definition member lists.

## Evidence boundary

This checkpoint introduces no new entity field or wire encoding. It composes
the already reviewed M9 raw section/record topology, M10 BLOCK/member/ENDBLK
topology, and M11 zero-width transaction insertion contract. Autodesk section
and entity rules remain normative. The user-authorized legacy tree under
`D:\SeaCad\tham khảo\New folder` was searched read-only for related placement
behavior; no equivalent reusable primitive was found. No external or legacy
code, fixture, data, or dependency was copied, translated, vendored, or linked.

## Contract

- `DxfEntityPlacementTarget` names one exact structure-section ordinal or one
  exact BLOCK marker raw-record ordinal.
- A completely indexed `ENTITIES` section anchors before its first content
  group. For an empty section that following marker is exact `ENDSEC`.
- A closed BLOCK definition anchors before its first member record. For an
  empty member list that following marker is exact `ENDBLK`.
- Choosing the start of the container prevents insertion in the middle of an
  existing POLYLINE or INSERT/ATTRIB sequence.
- Each ready placement retains source identity, target, section kind, following
  group occurrence, and an empty byte span at the marker's exact start.
- Interrupted or unclosed `ENTITIES` sections and interrupted or unclosed BLOCK
  definitions retain typed unavailable assessments and expose no placement.
- A nonzero orphan group at the beginning of `ENTITIES` content is typed as an
  invalid anchor so it cannot be silently attached to newly inserted bytes.
- Assessments follow container source order. Construction is cancellation-aware
  and inherits raw record resource bounds; every additional retained assessment
  uses fallible allocation.
- ASCII, Binary, and format-neutral document views expose the same API.

## Integration proof

Across AC1009 through AC1032 in both physical formats, fixtures contain a
member-bearing BLOCK, an empty BLOCK, a nonempty `ENTITIES` section, and an
empty `ENTITIES` section. All four placements are source-bound and group-zero
anchored. A minimal POINT record is inserted independently at every placement
through the M11 transaction builder; every post-image strict-reparses and adds
exactly one raw record.

## Nonclaims

This checkpoint does not encode `DxfEntityDraft`, infer or validate owner group
330, allocate or reserve handles, verify family semantics after insertion,
remap references, or implement insert/clone/delete closed sets.

## Verification

Focused tests cover all 18 dialect/format pairs, populated and empty containers,
target lookup, executable placements, strict post-image parsing, interrupted and
unclosed sections/BLOCK definitions, orphan groups, cancellation, public
Send/Sync bounds, and source identity. Full gate results and final artifact
hashes are recorded below. The focused suite passed 4/4 tests and the full
workspace passed 882/882 tests. Generated schema and release-evidence checks,
`cargo deny --locked check`, formatting, workspace Clippy with warnings denied,
workspace tests, forbidden-production-macro scan, and `git diff --check` all
passed. Production adds 338 lines: 333 in the placement module and five module
declaration/export lines. No production dependency changed. This audit
intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 246 | `aeb6159c8510e7aec199113ff5ab1d59eb4b9b855a5e3192d49ff39f4f314ea4` |
| `crates/seacad-dxf-core/src/entity_placement.rs` | 333 | `4a14a56a33f5bb85b79ff6fd48e62df7f57177e2ad07b7fb917dc83c3b32ceb8` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,043 | `2c28a810ebf9c33ac7d3e27fd90d18ff0fb9622212656fb5d361503ca002b886` |
| `crates/seacad-dxf-core/tests/entity_placement_tests.rs` | 382 | `ad896f9456efdf1eef60c7637081a3039de403fec6e07e73b2ebbea6d6a39497` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 871 | `d78f526192c1639029cc16fbf55b827978e6bd2c7bf82181c0099cc004a0f936` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,299 | `9597141ef16a6540e14ccd3dee4359a9ae3a28806c75c32d5856269aef9f6959` |
| `docs/SUPPORT_MATRIX.md` | 1,956 | `ae1cce8be2e1fbabb3b4ccfd402a2c8a33c6decbb5257c9923f046ba2a8f6808` |
