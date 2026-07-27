# M6.5j Surface Settings and Drawing Defaults Audit

## Scope

M6.5j appends twelve shape-reviewed scalar HEADER rows:

- `Int16` at group 70: `$SPLINESEGS`, `$SPLINETYPE`, `$SURFTAB1`,
  `$SURFTAB2`, `$SURFTYPE`, `$SURFU`, `$SURFV`, `$TILEMODE`, and
  `$TREEDEPTH`;
- `Double` at group 40: `$TEXTSIZE`, `$THICKNESS`, and `$TRACEWID`.

This checkpoint preserves the exact source scalar and provenance only. It does
not interpret enum domains, booleans, geometry, units, sizes, defaults,
version applicability, or cross-variable relationships.

## Normative evidence

Autodesk's published HEADER table lists every selected variable and group code:

<https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-A85E8E67-27CD-4C59-BE61-4DC9FADBE74A.htm>

The physical `Int16` and `Double` representations remain those audited in
M6.5a and M6.5b. Autodesk identifies `$TDCREATE` through `$TDUUPDATE` as
requiring special date/time handling; they are intentionally excluded from
this scalar batch. No external parser implementation was copied, translated,
or ported.

## Append-only ordinal contract

The prior 73 field ids remain frozen at ordinals 0-72. The new rows append at
ordinals 73-84 in the documented scope order. Generated Rust preserves
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
schema count 85, numeric count 82, unchanged old ordinals, appended new
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
removes 5. Test-only assertions and fixtures add 114 lines and remove 6. The
schema plus generated production registry adds 278 lines and removes 2, within
the approximate 200-500-line production target.

No dependency, unsafe block, field-specific production parser branch,
`panic!`, `unwrap`, `expect`, `todo!`, or `unimplemented!` was added.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `schema/dxf/v1/header.bootstrap.json` | `9f187881192c4678242d29532eb139e826345c5962b58d948e5c1062076de3e1` |
| `crates/seacad-schema-gen/src/main.rs` | `542ed5a21a7039f578049e7200863ac01f9081a48911969b80ffbc45c88e4fe2` |
| `crates/seacad-dxf-core/src/generated/header_schema.rs` | `60e67e276dbbc95caf5e5c102bec7601e7c9214732a3ebe2de7642e26560873f` |
| `crates/seacad-dxf-core/src/header_schema_directory.rs` | `53a549c762fd1160efa8ccfdaf283228bc75bec088b908c9fff2cbb37bd27240` |
| `crates/seacad-dxf-core/tests/header_numeric_tests.rs` | `d6b5399bef290f771b2808547750fb5679afce4cdb22bcdf4f1a6156627035d5` |
| `docs/IMPLEMENTATION_PLAN.md` | `06dd2010938e6b715fc7e1189ab6376c1d6dcf0e6573309ae81879468f82346c` |
