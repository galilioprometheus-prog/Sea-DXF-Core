# M14.3cf POINT Scalar Common Clone

Retrieved: 2026-08-03

## Scope

M14.3cf extends canonical semantic POINT clone with seven reviewed singleton
scalar common properties: paper/model space, indexed color, linetype scale,
visibility, true color, transparency, and shadow mode. The values remain typed
from source semantics through canonical draft encoding and post-image
verification. This checkpoint does not copy reference/text common properties,
proxy graphics, XDATA, extension dictionaries, or ownership graphs, and it does
not advance POINT to `Complete`.

## Contract

- Explicit groups 67, 62, 48, 60, 420, 440, and 284 are accepted only when
  their semantic singleton and reviewed public domain are valid.
- Defaulted or absent scalar properties remain omitted, preserving semantic
  state rather than manufacturing explicit groups.
- The POINT draft exposes typed setters/getters, canonical ASCII/Binary wire
  encoding, version checks, and non-negative finite linetype-scale validation.
- Clone preparation rejects duplicate, invalid, unsupported, or structurally
  ambiguous source groups without queueing a partial edit.
- Insert verification checks every requested scalar through the common-field
  semantic directory before a journal or inverse can escape.

## Verification boundary

The clone fixture inserts all four Core-wide scalar fields into every supported
version and both physical formats. AC1012+ fixtures additionally carry true
color, transparency, and shadow mode. All 18 dialect pairs allocate a fresh
handle, reparse strictly, recover every explicit typed value, satisfy the
semantic insertion verifier, and retain a byte-identical inverse. Existing
minimal/defaulted clone, explicit POINT payload, unsupported-group, partial
tuple, owner/placement, and pending-operation tests remain active.

## Nonclaims

This checkpoint does not clone linetype, material, color-book name, plot style,
proxy graphics, arbitrary application groups, XDATA, extension dictionaries,
reactors, ownership/reference closure, cross-container targets, or
cross-document data. Those sources remain typed fail-closed to prevent silent
loss or graph corruption.

## Verification

The focused entity insert session suite passed 11/11 tests and the POINT edit
suite passed 43/43 tests. The full workspace passed 960/960 tests, including all
17 locale tests. Generated-schema and release-evidence checks, `cargo deny
check`, formatting, workspace Clippy with warnings denied, the production
forbidden-construct scan, and `git diff --check` all passed. Production changes
are 488 insertions and 12 deletions; focused test changes are 182 insertions and
6 deletions. No manifest, lockfile, dependency, committed fixture, generated
schema, locale source, or external corpus changed. This audit intentionally
omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 375 | `f9a11586cb1366113d0cce8154c2f347f0a7d7074b802256b5b1f7f6a690a1cc` |
| `crates/seacad-dxf-core/src/entity_draft_record.rs` | 997 | `f3f78d2cbd2cc3121947bc6e24e75943d0df0858a6616ef85642befd16c37e5c` |
| `crates/seacad-dxf-core/src/entity_edit_session.rs` | 2,702 | `97e1cb6e16ebe489525f987111531753a93084fa52cd30545ada696d343bf816` |
| `crates/seacad-dxf-core/src/entity_edit_verification.rs` | 1,282 | `d0a0a8e63ea4c99b79ff428a380ecc78e74c50c6cefe6c18567dbc0fa899001c` |
| `crates/seacad-dxf-core/tests/entity_insert_session_tests.rs` | 1,274 | `0ddca60d22829a2dcbe13d8842b09f0e3b61379d75d1c7911e7fe01b2f03ef87` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,281 | `3c18e09bad9676b4ed22b06b16c540d81529b4687f23cbe84b636dd83a472647` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,708 | `400143999f9a99d7317c4603f4b785748650b0aea2d5ea893e30eeb759131b5a` |
| `docs/SUPPORT_MATRIX.md` | 2,358 | `15af422060106d059e4283de6b466073f6be3b62e472749c948678a24a3d99db` |
