# M9.1d LWPOLYLINE Vertex Semantics

Retrieved: 2026-07-29

## Normative boundary

- Autodesk [LWPOLYLINE (DXF)](https://help.autodesk.com/cloudhelp/2016/ENU/AutoCAD-DXF/files/GUID-748FC305-F3F2-4F74-825A-61F04D757A50.htm)
  identifies repeated OCS vertex coordinates `10/20`. Those rows are not
  marked optional.
- The same table marks per-vertex start width `40`, end width `41`, and bulge
  `42` optional with a default of zero.
- Autodesk [DXF Formatting Conventions](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-E50DB779-69AE-43C6-B004-85653A983AC0.htm)
  states that optional codes are explicitly identified as optional.
- The LWPOLYLINE table lists vertex identifier `91` without a documented
  default. M9.1d keeps its absence explicit rather than inventing a value.

M9.1d treats X/Y as required within an M9.1c vertex. Group `10` is always the
anchor but can still contain invalid numeric text; group `20` can be absent.
The zero defaults for `40/41` describe local variable-width fields only. The
Autodesk table also states that variable widths are not used when constant
width `43` is set, so this milestone does not publish an effective width.

## Implementation contract

`crates/seacad-dxf-core/src/lightweight_polyline_vertex_semantic.rs` provides:

- one lazy semantic directory owning the full M9.1c grouping directory;
- exact source identity and lookup by global or record-local vertex ordinal;
- required two-component OCS position with no invented coordinate;
- documented zero defaults for absent local start width, local end width, and
  bulge, each without raw provenance;
- explicit values with exact floating or signed-i32 provenance;
- optional identifier state with `Absent` rather than a fabricated default;
- typed `MissingRequiredValue`, `InvalidAsciiNumber`, and `MultipleValues`;
- tuple/value helpers that return values only when the relevant fields are
  usable;
- shared raw-document, ASCII-document, and Binary-document adapters;
- semantic construction only when a vertex is queried.

## Test evidence

`crates/seacad-dxf-core/tests/lightweight_polyline_vertex_semantic_tests.rs`
covers:

- ASCII/Binary parity across all supported AC1009-AC1032 physical dialects;
- exact signed-zero preservation for explicit position and bulge values;
- defaulted zero local widths and bulge with no raw provenance;
- constant width `43` coexisting with local defaults without an effective-width
  claim;
- explicit and absent identifiers;
- missing required Y, invalid X/width/identifier, and duplicate coordinates,
  widths, bulges, and identifiers;
- field/source provenance, global/record-local lookup, cancellation, and public
  traits.

The parity fixtures are physical decoding evidence only. They do not claim that
LWPOLYLINE is semantically applicable to every `$ACADVER` value.

## Non-claims

M9.1d does not resolve constant-versus-variable width precedence; publish
effective segment widths; validate non-negative widths or identifier ranges;
interpret flags or bulges; compare declared and observed counts; require unique
identifiers; validate version applicability; transform OCS; assemble straight
or arc segments; close a polyline; edit/write; render; snap; or infer topology.

## Required checkpoint gates

- `cargo deny --locked check`
- `cargo +1.97.1 fmt --all -- --check`
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`
- `cargo +1.97.1 clippy --locked --workspace --all-targets -- -D warnings`
- `cargo +1.97.1 test --locked --workspace`
- `git diff --check`

All checkpoint gates passed on 2026-07-29:

- dependency policy: advisories, bans, licenses, and sources all `ok`;
- formatting and generated-schema drift checks passed;
- workspace Clippy passed with warnings denied;
- workspace tests: 348 passed, 0 failed, 0 ignored, including all 3 focused
  M9.1d integration tests;
- `git diff --check` passed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/lightweight_polyline_vertex_semantic.rs` | 431 | `85bfa1d32c84282d304734f2562055659100f49373e63586a9a8de9b80136c36` |
| `crates/seacad-dxf-core/src/lib.rs` | 288 | `1d330380c4ce08ebd5d03542edfcac99b0436e6d98513899c94f25145fe73d27` |
| `crates/seacad-dxf-core/tests/lightweight_polyline_vertex_semantic_tests.rs` | 372 | `cb33547493073b3f10dc65d4a3ce76a9ec962f0841e2969296601b7495b5fb7c` |
| `docs/IMPLEMENTATION_PLAN.md` | 437 | `d51b3c125f236a985b5ccafe102fa96a5d4dca7f95f6a9a1eb6f7af62109bd59` |
| `docs/SUPPORT_MATRIX.md` | 350 | `c813248c1e1696defe2475988ef568f1b84dabd6264df68f845c7a9449649eb2` |
