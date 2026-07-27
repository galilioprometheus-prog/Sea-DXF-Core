# M6.5i Display and Regeneration Controls Audit

## Scope

M6.5i appends twelve shape-reviewed scalar HEADER rows:

- `Int16` at group 70: `$PLINEGEN`, `$PROXYGRAPHICS`, `$PSLTSCALE`,
  `$PUCSORTHOVIEW`, `$QTEXTMODE`, `$REGENMODE`, `$SHADEDGE`, `$SHADEDIF`,
  and `$SKPOLY`;
- `Double` at group 40: `$PSVPSCALE`, `$SHADOWPLANELOCATION`, and
  `$SKETCHINC`.

This checkpoint preserves the exact source scalar and provenance only. It does
not interpret enum domains, booleans, percentages, coordinates, units,
defaults, version applicability, or cross-variable relationships.

## Normative evidence

Autodesk's published HEADER table lists every selected variable and group code:

<https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-A85E8E67-27CD-4C59-BE61-4DC9FADBE74A.htm>

The physical `Int16` and `Double` representations remain those audited in
M6.5a and M6.5b. No external parser implementation was copied, translated, or
ported.

## Append-only ordinal contract

The prior 61 field ids remain frozen at ordinals 0-60. The new rows append at
ordinals 61-72 in the documented scope order. Generated Rust preserves
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
schema count 73, numeric count 70, unchanged old ordinals, appended new
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

The authoritative schema/generated/implementation-plan diff adds 286 lines and
removes 4. Test-only assertions and fixtures add 114 lines and remove 6. The
schema plus generated production registry adds 278 lines and removes 2, within
the approximate 200-500-line production target.

No dependency, unsafe block, field-specific production parser branch,
`panic!`, `unwrap`, `expect`, `todo!`, or `unimplemented!` was added.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `schema/dxf/v1/header.bootstrap.json` | `d401a2a3f536217e467ed78d0d57530de29839652ed49f6b88a8a0f92a175516` |
| `crates/seacad-schema-gen/src/main.rs` | `d56bdc1a338455f33e96030b46d9f24013920d8d7c3c01e73ff56e21b3946869` |
| `crates/seacad-dxf-core/src/generated/header_schema.rs` | `31323f273742ffd00f702860dacc8273749c451ebb4953ceba6be4ccddc200bf` |
| `crates/seacad-dxf-core/src/header_schema_directory.rs` | `838cd0e94c7f7b103570af24a08c6c3295d870e873edaf9d92fcc4f5b6f3f638` |
| `crates/seacad-dxf-core/tests/header_numeric_tests.rs` | `a33d8edc5c1b99bb97f504c8e820d5836f367d1e62fb5f9e2f811d6e94e1e91e` |
| `docs/IMPLEMENTATION_PLAN.md` | `02afd2fc9d94839c2312decac04a63b0d0d6319b6a1b05c4d139d4134cfa1b37` |
