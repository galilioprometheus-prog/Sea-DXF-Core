# M6.5c Schema-Driven Numeric Directory Audit

## Scope

This checkpoint adds one lazy, schema-ordered semantic directory for generated
HEADER fields whose reviewed wire storage is `Double` or `Int16`. It retains the
existing six-field `DxfHeaderNumericView` as a compatible convenience
projection.

This is an implementation scalability checkpoint. It makes no new claim about
DXF defaults, version applicability, enum meaning, valid ranges, or units.

## Contract

- `DxfHeaderNumericDirectory` is bound to the immutable document `SourceId` and
  publishes the generated schema version.
- Entries retain the original schema ordinal, stable field id, DXF variable
  name, four-state semantic value, field provenance, and raw provenance.
- Numeric storage selects the public value variant. A caller cannot silently
  read an `Int16` as a `Double`, or the reverse.
- Text and handle schema rows remain outside this numeric directory and are not
  reinterpreted.
- ASCII and Binary documents use the same directory and typed API. Existing
  `DxfHeaderNumericView` accessors are projected from it without changing their
  public result types.
- Cancellation is checked before and during directory construction. Allocation
  failure becomes a typed I/O error; production code adds no panic path.

## Complexity and bounds

The existing raw schema directory is resolved once. Numeric construction then
walks generated `HEADER_FIELDS` once and accesses each corresponding raw match
by schema ordinal. Construction is O(F) for F generated schema fields, plus
bounded source reads for explicit numeric values. It does not perform a
field-by-variable or field-by-field schema scan.

The entry vector reserves exactly the generated numeric-field count and is
converted to an immutable boxed slice. Lookup by schema ordinal uses binary
search over the ordered numeric entries; lookup by stable field id is linear in
the numeric directory size.

## Evidence

The wire-family and group-code facts remain those already audited in M6.5a and
M6.5b from Autodesk's HEADER variables and group-code value type references.
No external implementation source was copied or ported for this checkpoint.

Integration tests cover:

- schema order and stable ids for all supported AC1009-AC1032 versions;
- ASCII/Binary typed-value parity and source provenance;
- exclusion of non-numeric schema rows;
- wrong group-code invalidity through the directory API;
- cancellation and public `Copy`/`Send`/`Sync` bounds;
- redacted directory debug output.

Full workspace gate results and final file hashes are recorded in the checkpoint
commit and tag.

## Verification result

Rust 1.97.1 passed:

- `cargo fmt --all -- --check`;
- deterministic schema generation in `--check` mode with no diff;
- Clippy for the workspace and all targets with warnings denied;
- 227 workspace tests: 200 core unit tests, 5 numeric integration tests,
  12 CLI tests, and 10 schema-generator tests.

The production diff adds 295 lines and removes 92 across the numeric module and
public re-export, within the checkpoint's approximate 200-500-line production
change target. The reviewed numeric production module contains no `unsafe`,
`panic!`, `unwrap`, `expect`, `todo!`, or `unimplemented!`.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `crates/seacad-dxf-core/src/header_numeric.rs` | `54aae27ffead480231ac3a20f4c7c57e3a428ebd570d582b56a8415c9e8aa65f` |
| `crates/seacad-dxf-core/src/lib.rs` | `b0b3add5efa2d78738b4534f710971a1f3d9c0c1d3718337c6b9b92c6e62fd5e` |
| `crates/seacad-dxf-core/tests/header_numeric_tests.rs` | `0d7765c620ac118ca2e5300438ed3137563b2a45727c21067b9a46cc221d11a8` |
| `docs/IMPLEMENTATION_PLAN.md` | `c694b7f82fab32a176f2fc770565f07dcc95a867ef6cc9fb1cf520438142835f` |
