# M14.3k Common Entity Field Evidence and Cards

## Scope

M14.3k applies the M14.3j 19-role registry to the unified M14.3i entity index.
It produces exact occurrence evidence and deterministic cardinality cards
without interpreting field values.

## Contract

- Canonical topics, reviewed aliases, and unknown records in `BLOCKS` or
  `ENTITIES` receive cards. Reviewed markers classified `WrongSection` remain
  diagnostic index entries and receive no semantic field cards.
- Every accepted group is retained as `DxfEntityFieldOccurrence` with its
  `DxfEntityRef`, generated field id, and exact `DxfRawGroup` spans.
- Each semantic entity receives exactly 19 cards in generated descriptor
  order. States are `AbsentRequired`, `AbsentOptional`, `Unique`, `Duplicate`,
  or positive `Sequence`, and duplicate/sequence counts are explicit.
- Card members point back to occurrence ordinals; values are not copied or
  selected. Occurrences remain globally source ordered.
- Entity-preamble fields are accepted only before a reviewed subclass context;
  ordinary common fields are accepted in legacy or exact `AcDbEntity`
  context. Fields following another subclass marker are not confused with
  common properties.
- Group-102 content is excluded except group 360 inside exact
  `ACAD_XDICTIONARY`. That occurrence retains the application group's start-
  control ordinal, and the directory resolves it back to the complete exact
  entry and its `Closed`, `Interrupted`, or `Unclosed` state.
- Group 330 inside `ACAD_REACTORS`, group 360 outside `ACAD_XDICTIONARY`, and
  colliding codes in family subclasses are excluded without changing raw
  evidence.
- Construction uses checked compact ordinals, fallible reservations, bounded
  constant-factor cards, and cancellation checks before/during/after scans.
  Cross-source queries fail with the existing typed identity error.

## Dialect boundary

ASCII and Binary fixtures cover AC1009, AC1012, AC1014, AC1015, AC1018,
AC1021, AC1024, AC1027, and AC1032. AC1009 Binary's one-byte group-code framing
cannot represent later common codes above 255, so its paired ASCII fixture
omits those fields rather than inventing an encoding. Both physical formats
therefore expose the same evidence and card states within each dialect.

## Nonclaims

The directory does not decode text, numbers, handles, or binary chunks; apply
M14.3j defaults; validate field domains, handles, application-group closure,
or proxy graphics byte counts; select a duplicate; migrate older family APIs;
edit; or write. `AbsentRequired` describes schema/cardinality shape only while
field applicability remains unreviewed. No entity advances to `Complete`.

## Verification

Focused tests cover all nine dialects in ASCII and Binary, fixed card order,
required/optional absence, duplicate singleton and sequence counts, exact
source values, application closure provenance, subclass/application collision
exclusion, unknown and wrong-section behavior, lookups, source mismatch,
pre-scan and mid-read cancellation, debug redaction, and public metadata
bounds.

- `cargo test -p seacad-dxf-core --test entity_field_evidence_tests`: 4 passed.
- `cargo test --workspace`: 753 passed.
- Generated schema `--check`: passed.
- Release evidence `--check`: passed.
- `cargo deny --locked check`: advisories, bans, licenses, and sources passed.
  The first sandboxed run could not acquire the user-level advisory database
  lock; the approved rerun outside that restriction passed without policy
  findings.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `git diff --check`: passed.
- New production module: 500 lines, within the checkpoint target; `lib.rs`
  contains only module registration and public exports for this checkpoint.

## Artifact receipts

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 82 | `4e095e2a04fa633e53a7daf53220a9b2e8dab0856d49fc95d4ede395e3465f46` |
| `crates/seacad-dxf-core/src/entity_field_evidence.rs` | 500 | `ba324fd212485a0b39f712dc2cd1dab87be0cc3b469a25e60cbf1c0c4a847fc8` |
| `crates/seacad-dxf-core/src/lib.rs` | 856 | `c5d8930212eae9ba9d08ee2942d07f3e352f36a6213832e3af3d89b11b0df81f` |
| `crates/seacad-dxf-core/tests/entity_field_evidence_tests.rs` | 524 | `9b261296a6ca3393bd23a51d9a5203df122f6f5b381ccb1695c440e56618c3dd` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 373 | `238e8cf10466064b1ddc07087c74e4a68793556cfd6b0dcdc9ee603bd8ae97e4` |
| `docs/IMPLEMENTATION_PLAN.md` | 1785 | `e80fd058d462d127ebd53c3067ef20ab846f59b6729aac647cf94f56c948c70c` |
| `docs/SUPPORT_MATRIX.md` | 1413 | `b385ca7bc5996fc8c826014fcafdff5f62b7969f022f76e72a6099dbc4e920a2` |

This audit intentionally omits its own hash.
