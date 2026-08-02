# M14.3bj Pre-Release-13 Entity Applicability Receipt

## Scope

M14.3bj reviews the supported-dialect applicability of the 16 exact canonical
DXF names Autodesk identifies as introduced before AutoCAD Release 13.

## Normative evidence

Autodesk's current "About Adding an Entity to a Drawing (AutoLISP)" reference
contains a table titled "DXF names of entities introduced prior to AutoCAD
Release 13". It lists exactly:

- `3DFACE`, `ARC`, `ATTDEF`, `ATTRIB`, `CIRCLE`, and `DIMENSION`;
- `INSERT`, `LINE`, `POINT`, `POLYLINE`, and `SEQEND`;
- `SHAPE`, `SOLID`, `TEXT`, `VERTEX`, and `VIEWPORT`.

The generated source receipt records:

- source ID `autodesk.prer13.entities.compatibility.2026`;
- Autodesk topic `GUID-DFDAE6CD-E753-4D01-9D9B-4D1F66B1DE6E`;
- evidence kind `applicability_list`;
- normalized 16-row SHA-256
  `69b740e93788aaabb1be0ab69ce303eb7ca7f126dab0e167632d90b0f065cece`.

The existing reviewed dialect registry identifies AC1009 as AutoCAD R11/R12.
Because AC1009 is the Core 1.0 lower bound, an entity introduced before R13
cannot require a later supported dialect. The current Autodesk page still
recognizes the exact names, so the reviewed ranges have no maximum within
AC1009 through AC1032. No claim is made for a dialect older than AC1009.

The user-authorized legacy trees under `D:\SeaCad\tham khảo\New folder` were
searched read-only. Their public-repository intake and native evidence observe
the same exact markers and highlight family risks such as POLYLINE/ATTRIB
sequence closure, INSERT ownership, DIMENSION dependencies, and VIEWPORT
special handling. Those observations guided the nonclaims only; no external or
legacy code, fixture, data, dependency, or unsupported fact was copied,
translated, vendored, or linked.

## Contract

- All 16 exact canonical names are `Applicable` for AC1009, AC1012, AC1014,
  AC1015, AC1018, AC1021, AC1024, AC1027, and AC1032.
- Each descriptor exposes minimum AC1009, no reviewed maximum,
  `AutodeskCompatibility` evidence, the exact Autodesk GUID, and the normalized
  16-row source receipt.
- The independent draft-admission matrix tests all 59 names over all nine ASCII
  and Binary dialect pairs.
- Thirty-one names now have reviewed ranges; the other 28 remain
  `NotYetReviewed`.

## Nonclaims

This checkpoint does not imply that every listed entity can be inserted by the
current writer. Autodesk separately states on the same page that `entmake`
cannot create VIEWPORT entities; this receipt records file-format applicability
only and does not override that restriction. It does not add or close payload
semantics, tuple/sequence validation, references, geometry, encoding, insert,
update, clone, delete, or `Complete` support. It changes no production
dependency.

## Verification

Focused schema tests passed 9/9 and focused draft-applicability tests passed
4/4. The schema generator's 19 tests and the full workspace's 898 tests passed.
Schema `--check`, release-evidence `--check`, `cargo deny --locked check`,
formatting, workspace clippy with warnings denied, forbidden generated-
production macro scan, and `git diff --check` all passed. The first workspace
run exposed a stale test assumption that entry zero remained unreviewed; the
test now selects an actual `NotYetReviewed` row and again proves fail-closed
metadata validation. This checkpoint adds no handwritten production behavior;
its production delta is generated provenance/applicability data. No production
dependency changed. This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 253 | `214d91ee846c8702bec88e964622b56f6b524fd6e54301684905d5847bb1d3fc` |
| `schema/dxf/v1/sources.json` | 107 | `1d233ace387be55f8691699307de8d0e560255ba094d4279a6dec56d9564c028` |
| `schema/dxf/v1/entity_applicability.json` | 65 | `2c5e1929cd033961b2e226594cc3245c9fb265807874b750977a979f9cd36082` |
| `crates/seacad-schema-gen/src/main.rs` | 3006 | `4c6f5074fd5f7654d9ca74fe3f56d1386c3f4ffd75519e6ff52026f70c96fdb2` |
| `crates/seacad-dxf-core/src/generated/entity_schema.rs` | 1576 | `0872b9e0bb85361a549fc77ee5265d05d4420cc8796ddb75ffd2ce6c83eaa610` |
| `crates/seacad-dxf-core/src/generated/header_schema.rs` | 2392 | `34675e171859d3075f70344e80c96217ed7994c7dd4e38b6bf09de33b6699b7c` |
| `crates/seacad-dxf-core/tests/entity_schema_tests.rs` | 666 | `bbde0637bc17350e54468f2438dc66c90ddbc8b2b7d1ec605d533f9ac8f05dfb` |
| `crates/seacad-dxf-core/tests/entity_draft_applicability_tests.rs` | 487 | `82ff25583b1d744bad0cf7e54b21feea91af60bacbaf623258fdb6869f98781e` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 997 | `fda4b9696b039795781c264eb55180b493baed99c2140795e5827c83d509ed1f` |
| `docs/IMPLEMENTATION_PLAN.md` | 2431 | `58c3f687019f8b9326386bdce3fd742d9c62c598e3a97a042cc7864b789a5217` |
| `docs/SUPPORT_MATRIX.md` | 2082 | `bf008ad112afd0e5ad73de0b5b198bcf2974b3f815347033ba7460b30d11f471` |
