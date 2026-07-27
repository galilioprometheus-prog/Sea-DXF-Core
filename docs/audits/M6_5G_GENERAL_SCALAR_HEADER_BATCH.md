# M6.5g General Scalar HEADER Batch Audit

## Scope

M6.5g appends twelve shape-reviewed scalar HEADER rows:

- `Int16`: `$CECOLOR` at group 62, plus `$CMLJUST` and `$FILLMODE` at group 70;
- `Double`: `$CELTSCALE`, `$CHAMFERA`, `$CHAMFERB`, `$CHAMFERC`, `$CHAMFERD`,
  `$CMLSCALE`, `$ELEVATION`, `$FILLETRAD`, and `$LTSCALE` at group 40.

This checkpoint preserves the exact source scalar and provenance only. It does
not interpret colors, units, angles, distances, flags, enum domains, defaults,
version applicability, or relationships between variables.

## Normative evidence

Autodesk's published HEADER table lists every selected variable and the group
code above:

<https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-A85E8E67-27CD-4C59-BE61-4DC9FADBE74A.htm>

The physical `Int16` and `Double` representations remain those audited in
M6.5a and M6.5b. No external parser implementation was copied, translated, or
ported.

## Append-only ordinal contract

The prior 37 field ids remain frozen at ordinals 0-36. The new rows append at
ordinals 37-48 in the documented scope order. Generated Rust preserves
manifest order deterministically, and tests freeze every prior id before
checking the first new ordinal.

Field ids remain the canonical persisted identity. An ordinal is only an
append-only position in this schema generation and must not replace a field id
in durable external data.

## Semantic and bounds evidence

No field-specific production parser branch is added. Existing schema-wide
matching and the generic scalar decoders resolve all twelve rows in ASCII and
Binary documents. Each scalar reads at most one bounded value payload and
retains cancellation checks, exact IEEE-754 bits for doubles, and exact raw
provenance.

Construction remains O(F) in generated field count. Adding these fields does
not change raw framing, source limits, record limits, or value-size limits.

## Verification coverage

The AC1009-AC1032 ASCII/Binary matrix includes all twelve rows. Tests verify
schema count 49, numeric count 46, unchanged old ordinals, appended new
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
removes 5. Test-only assertions and fixtures add 124 lines and remove 6. The
schema plus generated production registry adds 278 lines and removes 2, within
the approximate 200-500-line production target.

No dependency, unsafe block, field-specific production parser branch,
`panic!`, `unwrap`, `expect`, `todo!`, or `unimplemented!` was added.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `schema/dxf/v1/header.bootstrap.json` | `f17f7de7db6b545ac6321b940dd3c72ac10c7b2eb512c533b6040a2e6465795f` |
| `crates/seacad-schema-gen/src/main.rs` | `d1775a524a7975be4729a820ae9aa273a5b27e82776eb64ba2b2c3160d3ac5d8` |
| `crates/seacad-dxf-core/src/generated/header_schema.rs` | `94c13367b3856c457cfbd9f03bb1928634234211cf7669711b08f34cdb739e77` |
| `crates/seacad-dxf-core/src/header_schema_directory.rs` | `9361f4619295c267f56ec280998b8e379603824a985dbc37961d099db2e6f4cb` |
| `crates/seacad-dxf-core/tests/header_numeric_tests.rs` | `685ad42031d8c60f1cddf5d305e9cf7571e02f30fdeb810b4d6b3b3d2bc4d52f` |
| `docs/IMPLEMENTATION_PLAN.md` | `f6d3d5a38d68b6c9f93c6b4dda5c1d766afa910cd15b380e60a01ebd38677e55` |
