# M14.3ci POINT Color-Book Clone

Retrieved: 2026-08-03

## Scope

M14.3ci extends canonical semantic POINT clone with the reviewed AC1012+
indexed-color, true-color, and color-book tuple in groups 62, 420, and 430. An
explicit color-book name is treated as a relation rather than independent text,
then remains exact through typed draft encoding, strict reparse, semantic
verification, and inverse generation.

## Contract

- The POINT draft accepts an optional exact group-430 value and requires both
  related typed colors whenever that name is explicit.
- A valid name contains exactly one `$` separator with non-empty book and color
  components; malformed syntax fails typed before insertion admission.
- Clone preparation accepts group 430 only in the reviewed common-field scope,
  enforces the value-byte resource limit, and retains its exact source bytes.
- Missing, duplicate, invalid, or wrong-scope tuple members fail closed without
  queueing a partial insertion.
- AC1009 rejects group 430 as not applicable because its Binary wire cannot
  represent the group code.
- Post-image verification requires exact explicit values for all three tuple
  members before a journal or inverse can escape.
- Optional text and reference expectations are grouped behind one bounded
  allocation so the edit expectation enum remains compact under denied Clippy
  warnings.

## Verification boundary

The positive fixture carries indexed color 40, true color 16235019, and exact
name `RAL CLASSIC$RAL 1003`. Every supported AC1012+ version in ASCII and Binary
clones the POINT to a fresh handle, retains all three values, satisfies strict
semantic verification, and produces an exact inverse. A dedicated malformed
name case proves a missing separator returns the typed color-book issue and
leaves the edit queue empty. Existing minimal, scalar-common, linetype,
object-reference, POINT-payload, scope-decoy, invalid-domain, owner/placement,
and mixed-session tests remain active.

## Nonclaims

This checkpoint does not load or validate external `.acb` files, prove that a
book/name maps to the RGB value, clone proxy graphics, XDATA, extension
dictionaries, reactors, or ownership/reference graphs, or support cross-
container or cross-document clone. POINT is not yet `Complete`.

## Verification

The focused entity insert session suite passed 16/16 tests and the POINT edit
suite passed 43/43 tests. The full workspace passed 965/965 tests, including all
17 locale tests. Generated-schema and release-evidence checks, `cargo deny
check`, formatting, workspace Clippy with warnings denied, the production
forbidden-construct scan, and `git diff --check` all passed. Production changes
are 105 insertions and 19 deletions; focused test changes are 117 insertions and
8 deletions. No manifest, lockfile, dependency, committed fixture, generated
schema, locale source, or external corpus changed. This audit intentionally
omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 384 | `fe2e5bcbca8bccd3da6ac109f322d44add9212fa432a3c8476b44e461be4cb3f` |
| `crates/seacad-dxf-core/src/entity_draft_record.rs` | 1,241 | `40afd0f9be2c23053b4eb880428f9af421d3813a0a3be7e1b8ae6dc78caf7dc5` |
| `crates/seacad-dxf-core/src/entity_edit_session.rs` | 2,771 | `b9bb175bfa1c8dd955b923bcb1177f0b092849063d3cb4bc6359499e35bf5fd3` |
| `crates/seacad-dxf-core/src/entity_edit_verification.rs` | 1,306 | `2b3a144a9ad316909ba9f61a77590a225d498943c4dbfdd3d61786fa02da8ad0` |
| `crates/seacad-dxf-core/tests/entity_insert_session_tests.rs` | 1,603 | `c092603c8bebff94939a2a548e71061427ac10173606fe3eb8d8e9689c74a7e6` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,305 | `c48307e795759586976cbff0f5963afaa98825dd324d950fd4810d66bebda165` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,731 | `c016df74dbc46f5e51df784ce434021ca081a59ecb013c7aab49907866e1205e` |
| `docs/SUPPORT_MATRIX.md` | 2,382 | `49691bd3917e8dcffb139ce5ef41343ce6150bddbc8a2bef8874fbeb7fe692e8` |
