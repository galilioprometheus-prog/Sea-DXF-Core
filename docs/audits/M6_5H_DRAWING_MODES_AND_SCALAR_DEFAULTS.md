# M6.5h Drawing Modes and Scalar Defaults Audit

## Scope

M6.5h appends twelve shape-reviewed scalar HEADER rows:

- `Int16` at group 70: `$LIMCHECK`, `$LUNITS`, `$LUPREC`, `$MAXACTVP`,
  `$MEASUREMENT`, `$MIRRTEXT`, `$ORTHOMODE`, `$PDMODE`, and `$PLIMCHECK`;
- `Double` at group 40: `$PDSIZE`, `$PELEVATION`, and `$PLINEWID`.

This checkpoint preserves the exact source scalar and provenance only. It does
not interpret enum domains, booleans, units, precision, distances, defaults,
version applicability, or cross-variable relationships.

## Normative evidence

Autodesk's published HEADER table lists every selected variable and group code:

<https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-A85E8E67-27CD-4C59-BE61-4DC9FADBE74A.htm>

The physical `Int16` and `Double` representations remain those audited in
M6.5a and M6.5b. No external parser implementation was copied, translated, or
ported.

## Append-only ordinal contract

The prior 49 field ids remain frozen at ordinals 0-48. The new rows append at
ordinals 49-60 in the documented scope order. Generated Rust preserves
manifest order deterministically, and tests freeze every prior id before
checking the first new ordinal.

Field ids remain the canonical persisted identity. An ordinal is only an
append-only position in this schema generation and must not replace a field id
in durable external data.

## Semantic and bounds evidence

No field-specific production parser branch is added. Existing schema-wide
matching and generic scalar decoders resolve all twelve rows in ASCII and
Binary documents. Each scalar reads at most one bounded value payload and
retains cancellation checks, exact IEEE-754 bits for doubles, and exact raw
provenance.

Construction remains O(F) in generated field count. Adding these fields does
not change raw framing, source limits, record limits, or value-size limits.

## Verification coverage

The AC1009-AC1032 ASCII/Binary matrix includes all twelve rows. Tests verify
schema count 61, numeric count 58, unchanged old ordinals, appended new
ordinals, exact scalar values, raw provenance, and independent malformed
`Double` and `Int16` evidence.

Final gate results and artifact SHA-256 receipts are appended after the
checkpoint verification run.

## Verification result

Rust 1.97.1 passed:

- `cargo fmt --all -- --check`;
- deterministic schema generation in `--check` mode;
- Clippy for the workspace and all targets with warnings denied;
- 228 workspace tests: 200 core unit tests, 6 numeric integration tests,
  12 CLI tests, and 10 schema-generator tests;
- `git diff --check`.

The authoritative schema/generated/implementation-plan diff adds 285 lines and
removes 4. Test-only assertions and fixtures add 114 lines and remove 6. The
schema plus generated production registry adds 278 lines and removes 2, within
the approximate 200-500-line production target.

No dependency, unsafe block, field-specific production parser branch,
`panic!`, `unwrap`, `expect`, `todo!`, or `unimplemented!` was added.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `schema/dxf/v1/header.bootstrap.json` | `7ad20d8f338ec71ba2aa46ea48a05f6df84b36df8941f1219bc10c5b0bbb7202` |
| `crates/seacad-schema-gen/src/main.rs` | `45a7e8f3b8f5886aea671f0a13bdadcd17e0e9347f6f01ef29b7b3bb1e04c49c` |
| `crates/seacad-dxf-core/src/generated/header_schema.rs` | `791445255a23afd6fe5516c139f817d843b19a4480770fb9def60c646a965d01` |
| `crates/seacad-dxf-core/src/header_schema_directory.rs` | `d733504bef43186601dea966ae3498eeff9a89d37ef05c12fffdcc69ebbd3359` |
| `crates/seacad-dxf-core/tests/header_numeric_tests.rs` | `01549f8494601b6473ff39b0b241b10c63ed14feba2fca03930e096fc6fda903` |
| `docs/IMPLEMENTATION_PLAN.md` | `6d3b4aa1574a82740a1fb4aafb96a2bbeb697b8da1bfa0bab38d4d26373fba81` |
