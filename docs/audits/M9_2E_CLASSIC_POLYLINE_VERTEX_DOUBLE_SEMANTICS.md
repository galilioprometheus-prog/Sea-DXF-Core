# M9.2e Classic POLYLINE VERTEX Double Semantics

Retrieved: 2026-07-30

## Normative boundary

- Autodesk [VERTEX (DXF)](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-0741E831-599E-4CBF-91E1-8ADBCFD6556D.htm)
  documents location `10/20/30`, optional start/end width `40/41` with zero
  defaults, optional bulge `42` with a zero default, and curve-fit tangent
  direction `50`.
- The same reference states that location is OCS for 2D vertices and WCS for
  3D vertices. M9.2e does not choose that coordinate system before vertex
  classification is proven.
- The curve-fit tangent has conditional meaning when flag bit `2` is set.
  M9.2e keeps an absent tangent absent and does not evaluate that condition.

## Implementation contract

`crates/seacad-dxf-core/src/polyline_vertex_semantic.rs` provides:

- lazy semantic projection over the seven M9.2d double-role cards;
- required location X/Y/Z with typed missing-component failures;
- exact explicit start/end width and bulge or documented zero defaults only
  when their cards are absent;
- explicit, absent, or invalid curve-fit tangent direction without invention;
- typed invalid ASCII and duplicate states with first available raw provenance;
- tuple helpers that publish position or width pairs only when every component
  is usable;
- retained M9.2a-e source, parent-POLYLINE, VERTEX, sequence, card, member, and
  raw-value evidence;
- lookup by VERTEX raw ordinal, exact entry, or parent-POLYLINE plus sequence-
  local vertex ordinal, with ASCII/Binary adapters and bounded cancellation.

## Test evidence

`crates/seacad-dxf-core/tests/polyline_vertex_semantic_tests.rs` covers:

- exact ASCII/Binary semantic parity across every supported AC1009-AC1032
  dialect;
- explicit signed-zero location/bulge, explicit widths and tangent direction,
  plus defaulted zero widths/bulge and absent tangent;
- invalid ASCII, missing required location, duplicate required location,
  duplicate width/bulge, and the separation of defaults from failures;
- exact field/raw provenance and tuple-helper fail-closed behavior;
- semantics retained in closed, interrupted, and unclosed sequences across
  `BLOCKS` and `ENTITIES`;
- entry/raw-ordinal/parent lookup bounds, cancellation, and public traits.

The fixtures prove only these reviewed double-field semantics. They do not
claim that every field applies to every vertex family or `$ACADVER` dialect.

## Non-claims

M9.2e does not decide OCS versus WCS; interpret flags; require or interpret a
curve-fit tangent from flag bit `2`; project identifiers or polyface indices;
classify 2D, 3D, polygon-mesh, mesh-coordinate, or polyface-face vertices;
validate coordinate, width, bulge, or angle domains; resolve polyface faces;
transform coordinates; assemble geometry; edit; write; render; or diagnose
conformance.

## Required checkpoint gates

- `cargo deny --locked check`
- `cargo +1.97.1 fmt --all -- --check`
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`
- `cargo +1.97.1 clippy --locked --workspace --all-targets -- -D warnings`
- `cargo +1.97.1 test --locked --workspace`
- `git diff --check`

All checkpoint gates passed on 2026-07-30:

- `cargo deny --locked check`: advisories, bans, licenses, and sources passed;
- `cargo +1.97.1 fmt --all -- --check`: passed;
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`: generated
  schema matched the committed artifacts;
- `cargo +1.97.1 clippy --locked --workspace --all-targets -- -D warnings`:
  passed with warnings denied;
- `cargo +1.97.1 test --locked --workspace`: 379 passed, 0 failed, 0
  ignored, including all four focused M9.2e integration tests;
- `git diff --check`: passed.

No dependency manifest or lockfile changed. A focused production-source scan
also found none of the prohibited `unsafe`, panic, unwrap, expect, todo, or
unimplemented constructs in the new module.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/polyline_vertex_semantic.rs` | 366 | `ea6990c19ed32ec3682b38e466edfacab8bbfab4b4c79a94d3097d575c4a72ed` |
| `crates/seacad-dxf-core/src/lib.rs` | 337 | `439b678ad83a38945f4add15c7445f7945baba307b7bdad2ce65c09ddd1264ae` |
| `crates/seacad-dxf-core/tests/polyline_vertex_semantic_tests.rs` | 369 | `32354185bdd32e4b3a022101267f4e11e36fa091429b216d5c1495f6a5c0fec5` |
| `docs/IMPLEMENTATION_PLAN.md` | 543 | `72a64297d4ef3ec9e1529f95d4420e4483da6125c3485caea222ee33b5de02e1` |
| `docs/SUPPORT_MATRIX.md` | 456 | `0223fcbf00fb0c356982ad9cddd088f271acaeca3658ab035136c5ffeb095431` |
