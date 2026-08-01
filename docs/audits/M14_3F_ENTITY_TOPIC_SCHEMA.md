# M14.3f Entity Topic Schema

## Scope

M14.3f freezes the 45 canonical entity topics linked by Autodesk's 2024
"About the DXF ENTITIES Section" page into reviewed schema input and generated
public Rust descriptors.

## Contract

- Manifest order is the stable topic ordinal order.
- Every topic has one lowercase schema id and one exact uppercase DXF marker.
- Every descriptor retains the Autodesk topic GUID and normalized-facts
  SHA-256 recorded by the source registry.
- Lookup is byte-exact and case-sensitive; aliases and unknown markers return
  no canonical topic.
- The generator requires exactly 45 unique topic ids and names under the exact
  `entity` namespace and a `topic_list` evidence source.
- Generated output participates in the existing deterministic `--check` gate.

The normalized source facts are the deterministic compact JSON serialization
of the ordered `topics` array; their SHA-256 is
`a7d540ad9109768edd57db100743148756c21966854b634dc04450066fd3e4f7`.
The generated entity-schema receipt, which additionally binds the source
registry entry, is
`7a492812fa00c44ca58016753dd5aba682b09a1951dfd7e885e251d9be10d68e`.

## Coverage and size

The focused core suite verifies the exact 45-name order, stable ordinals,
descriptor provenance, exact-name round trips, case rejection, alias
neutrality, unknown-name rejection, lookup bounds, and public traits. Generator
tests separately cover determinism, duplicate id/name rejection, and stale
normalized-source receipt rejection.

## Nonclaims

This checkpoint does not scan or classify raw records, map aliases, define
version applicability or entity fields, expose common properties, add semantic
support for any entity, construct geometry, edit, or write. Raw preservation
and all existing entity support claims remain unchanged.

## Verification

All required local gates passed on 2026-08-01: dependency policy, generated
schema drift, release-evidence drift, formatting, workspace Clippy with
warnings denied, 734 workspace tests, and `git diff --check`. The focused
generator/core runs passed 16/16 and 3/3 tests respectively.

The first final `cargo deny` attempt could not take the advisory database lock
through the read-only sandbox. The approved rerun passed advisories, bans,
licenses, and sources. No dependency, manifest package, lockfile, user-facing
CLI/i18n string, private-corpus claim, or cloud/six-host receipt changed.

Handwritten generator production changed by 362 additions and 20 removals,
within the checkpoint's 200--500-line review target. The 522-line entity
registry is generated code; the focused public contract test is 115 lines.
The audit omits its own hash so the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 70 | `0f962751dc8cecc2986bcc3266cc7c16274a2c4eff330565426351d35e67f799` |
| `crates/seacad-dxf-core/src/generated/entity_schema.rs` | 522 | `877bd50ccbfd4fc1c639a4006d188180b030ef66c9bcf786dd950df0103b6e2f` |
| `crates/seacad-dxf-core/src/generated/header_schema.rs` | 2,387 | `0cddda7dc3ae2a6796bc1228134935de54237802a35f68eb47e89d763cec56f9` |
| `crates/seacad-dxf-core/src/generated/mod.rs` | 2 | `0c9e0a99f7a9166d67431b77573ac05143626ca9a41761311be16a5f774eef03` |
| `crates/seacad-dxf-core/src/lib.rs` | 838 | `5b1ce88330f4336fc854f004926cac057c4b19a861d1d608a31407ae984e8ee9` |
| `crates/seacad-dxf-core/tests/entity_schema_tests.rs` | 115 | `edf5f0bf9bb06e44a30f85f69723a1f017337064fa0768dc645f8776cabca25c` |
| `crates/seacad-schema-gen/src/main.rs` | 1,461 | `ad2cff7edc989206d23ca832cbe569b0045ed897334e8c3b32f66b8f69094eb4` |
| `schema/dxf/v1/entity_topics.json` | 52 | `2e1cf3853b6fd1b35094719b8d76233fd3e946476c7c3113f082188b453a314a` |
| `schema/dxf/v1/manifest.json` | 8 | `b0b137f559a901a8b6ecb8be4c3fe7f9208b23ff915ab4bcc4f3c52cf253143b` |
| `schema/dxf/v1/sources.json` | 17 | `a637b82599ffd83762f2c1cf5334e3d010efebe14e32a75c6f33f2ad241fcd04` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 330 | `20174a2b37de59d1602b5c92398a4a52dea25d6021b41da3df4aeaccc3e689e3` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,728 | `fcb0ed3d5308dfc9ee1ce8122affbcf9001c8b60a587896d2afeafb7b91be13a` |
| `docs/SUPPORT_MATRIX.md` | 1,364 | `5452d3c336d276ec82a3f4094f8dec6270546107665d085394fff6180d16232f` |
