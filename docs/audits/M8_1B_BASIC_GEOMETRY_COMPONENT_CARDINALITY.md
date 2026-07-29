# M8.1b Basic-Geometry Component Cardinality

## Scope

M8.1b layers a deterministic per-role card directory over the M8.1a exact
`POINT` and `LINE` occurrence evidence. Every recognized `POINT` has six cards:
WCS location X/Y/Z and extrusion X/Y/Z. Every recognized `LINE` has nine cards:
WCS start X/Y/Z, WCS endpoint X/Y/Z, and extrusion X/Y/Z.

Each card records `Absent`, `Unique`, or `Multiple { occurrence_count }` and a
half-open member slice. Members contain compact ordinals back into the single
owned M8.1a evidence directory; component values and source metadata are not
duplicated.

## Contract

- Cards follow raw-record order and a fixed documented role order.
- Cardinality counts source occurrences independently of lexical validity.
- `Unique` can therefore point to either an exact double or one typed invalid
  ASCII number; `Multiple` retains every valid or invalid occurrence.
- `POINT` does not receive endpoint cards because codes `11/21/31` are not part
  of its reviewed M8.1a role set.
- Missing optional extrusion components remain `Absent`; the documented
  `(0, 0, 1)` default is not applied at this evidence layer.
- Every public document adapter uses the same ASCII/Binary implementation and
  observes cancellation.

## Evidence

- `crates/seacad-dxf-core/src/basic_geometry_card.rs` implements the fixed role
  registry, compact card/member indexes, cardinality states, source identity,
  cancellation, and public queries.
- `crates/seacad-dxf-core/tests/basic_geometry_tests.rs` verifies card and
  member parity for all nine AC1009-AC1032 dialects in ASCII and Binary,
  absent/unique/multiple states, source-order duplicate values, invalid member
  retention, role applicability, bounds, cancellation, and public trait bounds.
- Normative role meanings remain those recorded in
  `docs/audits/M8_1A_AUTODESK_BASIC_GEOMETRY_REFERENCE.md`.

## Non-claims

M8.1b does not choose a primary value, declare a record complete or valid,
apply defaults, normalize extrusion, assemble a point or line, transform a
coordinate system, validate subclass/version rules, or implement edit/write,
rendering, snapping, topology, CIRCLE, or ARC semantics.

## Checkpoint verification

- `cargo deny --locked check`: advisories, bans, licenses, and sources all OK.
- `cargo +1.97.1 fmt --all -- --check`: exit 0.
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`: exit 0.
- `cargo +1.97.1 clippy --locked --workspace --all-targets -- -D warnings`:
  exit 0.
- `cargo +1.97.1 test --locked --workspace`: 300 passed, 0 failed.
- `cargo +1.97.1 test --locked -p seacad-dxf-core --test basic_geometry_tests`:
  4 passed, 0 failed, including card parity across all nine supported dialects
  and both physical encodings.
- `git diff --check`: exit 0.

## Artifact receipt

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/basic_geometry_card.rs` | 297 | `FE290F93E75F22D70E2275109F14B061DB30DCBDF724F382402A40950ED9A0F7` |
| `crates/seacad-dxf-core/src/lib.rs` | 199 | `240D43F57379C276078E0B9D386617000417EA3FD1717735D4463DD49BFF9569` |
| `crates/seacad-dxf-core/tests/basic_geometry_tests.rs` | 454 | `B7E5213D4D66BC9080867E62320FCA2B9AD744E4ED288E0EF6E8A5B93408DD5E` |
| `docs/IMPLEMENTATION_PLAN.md` | 314 | `5180539E580A2FCD49A7031017DA676DE327EC453309E9537DE83BFCF1BDD399` |
| `docs/SUPPORT_MATRIX.md` | 198 | `F812A02096F79DD01BE2AA2570845429878E18928743767EA1DC58F1A0C92B69` |

The receipt excludes its own self-referential hash. Its final SHA-256 is
reported in the checkpoint handoff before any commit or tag decision.
