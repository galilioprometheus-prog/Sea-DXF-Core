# M9.1e LWPOLYLINE Record Cardinality

Retrieved: 2026-07-29

## Normative boundary

- Autodesk [LWPOLYLINE (DXF)](https://help.autodesk.com/cloudhelp/2016/ENU/AutoCAD-DXF/files/GUID-748FC305-F3F2-4F74-825A-61F04D757A50.htm)
  identifies vertex count `90`, flags `70`, OCS elevation `38`, thickness `39`,
  constant width `43`, and extrusion components `210/220/230` as record-level
  LWPOLYLINE fields.
- The same table identifies coordinates `10/20`, start/end widths `40/41`,
  bulge `42`, and identifier `91` as per-vertex values. M9.1e excludes them from
  record-level cards.
- Autodesk [Group Code Value Types](https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-2553CF98-44F6-4828-82DD-FE3BC7448113.htm)
  assigns group `70` to the signed 16-bit range and group `90` to the signed
  32-bit range. M9.1e resolves those cards back to the typed M9.1b evidence.

The Autodesk table documents valid fields, defaults, and meanings. This
milestone records occurrence cardinality only. A present but lexically invalid
value remains a card member and is not treated as absent.

## Implementation contract

`crates/seacad-dxf-core/src/lightweight_polyline_record_card.rs` provides:

- one owned directory over the complete M9.1a floating and M9.1b integer
  evidence directories with source-identity and record-alignment checks;
- eight stable cards per exact recognized LWPOLYLINE record;
- `Absent`, `Unique`, or `Multiple { occurrence_count }` independently from
  the retained numeric result;
- compact members carrying only record role and original group occurrence;
- typed resolution back to floating evidence for `38/39/43/210/220/230` and
  integer evidence for `90/70`;
- record, card, role, member, and underlying-value lookup without copying raw
  values;
- shared raw-document, ASCII-document, and Binary-document adapters;
- cancellation checks and fallible allocation throughout construction.

## Test evidence

`crates/seacad-dxf-core/tests/lightweight_polyline_record_card_tests.rs`
covers:

- ASCII/Binary parity across all supported AC1009-AC1032 physical dialects;
- stable eight-role ordering and exact floating-bit/i16/i32 evidence;
- vertex fields and identifier `91` excluded from record cards;
- absent, unique, and duplicate cardinality;
- invalid ASCII integer and floating values retained behind unique or multiple
  cards;
- a record containing only vertex-scoped evidence producing eight absent
  record-level cards;
- source identity, typed member resolution, lookup bounds, cancellation, and
  public traits.

The parity fixtures are physical decoding evidence only. They do not claim
that LWPOLYLINE is semantically applicable to every `$ACADVER` value.

## Non-claims

M9.1e does not select canonical values; apply elevation, thickness, width,
flag, or extrusion defaults; require or validate count/flag values; compare
declared and observed vertex counts; interpret closure or polyline-generation
flags; resolve constant-versus-variable width precedence; validate ranges or
version applicability; transform OCS; assemble segments; edit/write; render;
snap; or infer topology.

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
- workspace tests: 351 passed, 0 failed, 0 ignored, including all 3 focused
  M9.1e integration tests;
- `git diff --check` passed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/lightweight_polyline_record_card.rs` | 499 | `dffaec4165a47ce4f09f5ecad4080cc8c42c09bc68d41f2cf20f62d8abd68a39` |
| `crates/seacad-dxf-core/src/lib.rs` | 295 | `69d4f685ba466b4c60653e163d159c818efda813e72b8b8b0a091944544af085` |
| `crates/seacad-dxf-core/tests/lightweight_polyline_record_card_tests.rs` | 398 | `7a40663741206709face7f0e39702aae444ad4134324e281f7f8d02269462b7d` |
| `docs/IMPLEMENTATION_PLAN.md` | 447 | `b8a83ec985113c5434e9a7e68a73c51ef026abb45c95128ffc07d51765560970` |
| `docs/SUPPORT_MATRIX.md` | 362 | `1ce12e8196899585fb95a726080fde3e2697921d02dca6c7ff5fca9d34af6fb9` |
