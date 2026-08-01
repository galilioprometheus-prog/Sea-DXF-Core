# M14.3g Entity Alias Schema

## Scope

M14.3g adds the reviewed on-wire names that do not equal one of M14.3f's 45
canonical Autodesk topic labels. The generated API classifies exact bytes as a
canonical topic, a reviewed alias/specialization, or unknown.

## Evidence boundary

Five mappings are normative Autodesk facts:

| Exact name | Canonical topic | Autodesk source |
|---|---|---|
| `MPOLYGON` | HATCH | `GUID-C6C71CED-CE0F-4184-82A5-07AD6241F15B` |
| `ACAD_TABLE` | TABLE | `GUID-D8CCD2F0-18A3-42BB-A64D-539114A07DA0` |
| `DGNUNDERLAY` | UNDERLAY | `GUID-3EC8FBCC-A85A-4B0B-93CD-C6C785959077` |
| `DWFUNDERLAY` | UNDERLAY | `GUID-3EC8FBCC-A85A-4B0B-93CD-C6C785959077` |
| `PDFUNDERLAY` | UNDERLAY | `GUID-3EC8FBCC-A85A-4B0B-93CD-C6C785959077` |

Autodesk's HATCH page states that its codes apply to hatch and MPolygon
entities, its TABLE page identifies group-zero `ACAD_TABLE`, and its UNDERLAY
page identifies the three format-specific group-zero names.

Nine additional names are behavioral observations from the read-only AutoCAD
2027 inventory oracle: `ARC_DIMENSION`, `LARGE_RADIAL_DIMENSION`,
`MULTILEADER`, `SECTIONOBJECT`, `EXTRUDEDSURFACE`, `LOFTEDSURFACE`,
`PLANESURFACE`, `REVOLVEDSURFACE`, and `SWEPTSURFACE`. The reviewed legacy note
has SHA-256
`C413731DBBFCD74D25CEAF0AFB435DDFD44D1149EC5587150A1D65510C2F1870`.
These descriptors expose `BehavioralOracle`; they are not relabeled as
normative. No parser, fixture, corpus file, or implementation was copied from
the legacy workspace.

## Contract

- Alias ordinals follow append-only schema order.
- Exact-name lookup is allocation-free, byte-exact, and case-sensitive.
- `DxfEntityTopic::from_exact_name` remains canonical-only for migration
  safety; `classify_exact_dxf_entity_name` is the explicit combined API.
- Alias descriptors retain the exact wire name, canonical topic, evidence
  kind, source id/reference, and normalized source-facts SHA-256.
- Each source receipt hashes only the mappings attributed to that source.
- Generator validation rejects count, namespace, id/name collision, canonical
  name collision, missing topic/source, wrong evidence kind, and stale receipt.
- The alias registry receipt is
  `c95364127df561284a935f0fc989eff13bf7fc813a9e381a00e7e986d27829eb`.

## Nonclaims

This checkpoint does not scan raw records, decide section or dialect legality,
add `WrongSection`, define subclass/field applicability, expose common entity
properties, parse a new family, project semantics or geometry, edit, write, or
advance any entity support state. The term alias includes concrete
specializations for inventory purposes and does not claim interchangeability
of their family-specific fields.

## Verification

The focused generator suite passed 17/17 tests and the public entity-schema
contract passed 5/5. The complete workspace passed 737 tests. Generated schema
freshness, release-evidence freshness, formatting, workspace Clippy with
warnings denied, dependency policy, and `git diff --check` passed.

The first dependency-policy attempt could not lock the advisory database in
the read-only sandbox; the approved rerun passed advisories, bans, licenses,
and sources. The first combined workspace-test command reached its 120-second
process limit after its preceding gates had passed; an isolated 300-second
rerun completed successfully. No dependency, lockfile, CLI JSON/i18n contract,
support claim, private-corpus evidence, or six-native receipt changed.

Handwritten generator production changed by 450 additions and 14 removals,
within the 200--500 changed-line review target. Generated Rust added 310 lines;
the registry schema is 20 lines. This audit omits its own hash so its receipt
is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 86 | `2887228e80d0ce137089b996894141005bdab5a2b2f7943840845d33a30c081f` |
| `crates/seacad-dxf-core/src/generated/entity_schema.rs` | 847 | `29dc423b35ffd686e1b43dcc9e9e513d3cc1b339a6a2066a0a323a8ade7205f5` |
| `crates/seacad-dxf-core/src/generated/header_schema.rs` | 2,392 | `4c72cafd69ace0498848526a6fd26a840e1a4fba34c396f178f637cfddcb7b46` |
| `crates/seacad-dxf-core/src/lib.rs` | 845 | `a9bffba847a209f6bfa5de8e19b9e60fca3f596ecffb62f88be535283b9ad2a7` |
| `crates/seacad-dxf-core/tests/entity_schema_tests.rs` | 204 | `192aee056e34758ad076b9987080b98e6f2d17f4edd57c2d1b5baf916395249b` |
| `crates/seacad-schema-gen/src/main.rs` | 1,977 | `4faa3a36d5890ecfa415d3621de578aa9216454cf4733e85d6224e73d111cdb8` |
| `schema/dxf/v1/entity_aliases.json` | 20 | `c33115a6c96e7004bfc98b1f308c154757e7d2ccc1c6137ad1e9eb435656a858` |
| `schema/dxf/v1/manifest.json` | 9 | `999b23ea2c56ec9fcbb11c521c84df234a4a643e1c87b2f3b2802a0bf0b4fc55` |
| `schema/dxf/v1/sources.json` | 41 | `e3a7d88996d4ed9b751cd348e04cddf1397dd986eeeef4432f1600183124a621` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 360 | `69c2b0237ff529a474a8e4360f2c57daab23ccfa3f862fc80ccbff647a97c499` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,741 | `8f1be1fc3c400cd87c413eb053bf15f4e3e7a7d062994832a649208ed38f0190` |
| `docs/SUPPORT_MATRIX.md` | 1,421 | `2515a71c69c15f8bb3255c4131b1387c00a64f9ece087265f7519d00ac0d095b` |
