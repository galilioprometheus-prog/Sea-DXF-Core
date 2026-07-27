# M6.5d HEADER Double Tuple Audit

## Scope

M6.5d adds shape-reviewed `Double2` and `Double3` schema storage and connects
the first eight HEADER coordinate rows to the schema-driven numeric semantic
directory:

- `$EXTMAX`, `$EXTMIN`, `$INSBASE`, `$PEXTMAX`, `$PEXTMIN`, and `$PINSBASE`
  use ordered group codes 10, 20, and 30;
- `$LIMMAX` and `$LIMMIN` use ordered group codes 10 and 20.

The generic names `Double2` and `Double3` describe only the reviewed wire
shape. This checkpoint does not assign point versus vector meaning, apply WCS
or UCS transforms, calculate extents, fill defaults, or validate geometric
ranges.

## Normative evidence

Autodesk's published HEADER table names every selected variable and its ordered
component group codes:

<https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-A85E8E67-27CD-4C59-BE61-4DC9FADBE74A.htm>

Autodesk's group-code value-type reference classifies codes 10-39 as
double-precision point components:

<https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-2553CF98-44F6-4828-82DD-FE3BC7448113.htm>

Autodesk's Binary DXF reference specifies that group/value pairs retain their
ASCII meaning and that double values are little-endian IEEE binary64:

<https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-FC1C3C69-DBC2-49E4-893A-000D6538C0FE.htm>

No external parser implementation was copied, translated, or ported.

## Schema and generator contract

- `Double2` accepts exactly `[x, x + 10]`; `Double3` accepts exactly
  `[x, x + 10, x + 20]`, with `x` in the reviewed 10-18 X-component family.
- Field ids remain strictly sorted and evidence anchors must exactly match the
  Autodesk variable name.
- Generated Rust is committed, deterministic, and checked for no diff in CI.
- All eight rows remain `shape_only`, optional, with applicability and default
  policy explicitly not yet reviewed.

## Semantic contract

`DxfHeaderNumericValue::Double2` and `Double3` contain two or three independent
`DxfSemanticValue<DxfDouble, DxfHeaderNumericIssue>` components. Consequently:

- every component keeps its exact binary64 bits, field provenance, group
  occurrence, and value byte span;
- a missing or wrong component is `Invalid` without discarding valid siblings;
- an absent variable makes every component `Absent`;
- duplicate variables make every component invalid with duplicate evidence;
- an unexpected extra component invalidates the tuple with an explicit
  expected/observed count and points to the first extra group;
- tuple state is `Explicit`, `Defaulted`, or `Absent` only when all components
  share that state; all mixed or invalid states report `Invalid`.

The entry also publishes its reviewed group-code slice. Callers must select the
matching `Double2` or `Double3` variant explicitly; there is no silent scalar
or dimension coercion.

## Bounds

Construction still resolves the raw schema directory once and walks generated
fields once. A tuple reads at most three already-framed value payloads, using
the existing bounded ASCII reader or fixed-width Binary reader. Tuple decoding
allocates no per-component heap storage and checks cancellation between
components.

## Verification coverage

Tests cover all supported AC1009-AC1032 dialects in ASCII and Binary, all eight
new fields, schema order and group codes, exact component bits, absence,
per-component wrong code and missing values, unexpected extra components,
duplicate variables, empty variables, provenance, aggregate state, public
`Copy`/`Send`/`Sync` bounds, and redacted debug output.

Final gate results and artifact SHA-256 receipts are appended after the
checkpoint verification run.

## Verification result

Rust 1.97.1 passed:

- `cargo fmt --all -- --check`;
- deterministic schema generation in `--check` mode with no diff;
- Clippy for the workspace and all targets with warnings denied;
- 228 workspace tests: 200 core unit tests, 6 numeric integration tests,
  12 CLI tests, and 10 schema-generator tests;
- `git diff --check`.

The authoritative schema/generator/generated/core implementation diff adds 467
lines and removes 23, within the checkpoint's approximate 200-500-line
production target. Test-only updates add 181 lines and remove 13. The reviewed
production paths contain no unsafe block, `panic!`, `unwrap`, `expect`, `todo!`,
or `unimplemented!`.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `schema/dxf/v1/header.bootstrap.json` | `2eecf6660766473e740584e922ecdee895d578fa75d496c3534c98a409b65a73` |
| `crates/seacad-schema-gen/src/main.rs` | `15bb9eb046d2c1a02a8e7fb94bc6cb428e1b89b07300b3bc80b7ffb3c2d011d0` |
| `crates/seacad-dxf-core/src/generated/header_schema.rs` | `5fff01eb7a1993fa64c9f2b4fea286f462d94b7a13d8f2b6d6551dcac5a4a385` |
| `crates/seacad-dxf-core/src/header_numeric.rs` | `f5114bf2cfd1d83c79c85cc65bab8b86051d5e47c051bff9f16370eeea6c127d` |
| `crates/seacad-dxf-core/src/header_schema_directory.rs` | `aee1be92fd77bb942f8c1b9289961c53e32069bc18ab6918bcb478a934606969` |
| `crates/seacad-dxf-core/tests/header_numeric_tests.rs` | `10713c5b2744e87e330b4ea682b46497937cbd737230896f778efc26e668dfdf` |
| `docs/IMPLEMENTATION_PLAN.md` | `1e2cac7c17a0c26b8d2f5154a442033507f924b39ba94c333ca896ccf802dc40` |
