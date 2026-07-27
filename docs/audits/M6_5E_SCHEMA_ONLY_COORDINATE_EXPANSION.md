# M6.5e Schema-Only Coordinate Expansion Audit

## Scope

M6.5e adds eight reviewed HEADER rows to the existing `Double2`/`Double3`
pipeline:

- `$PLIMMAX` and `$PLIMMIN` use ordered group codes 10 and 20;
- `$PUCSORG`, `$PUCSXDIR`, `$PUCSYDIR`, `$UCSORG`, `$UCSXDIR`, and `$UCSYDIR`
  use ordered group codes 10, 20, and 30.

All rows remain optional `shape_only` entries with applicability and default
policy explicitly not yet reviewed. This checkpoint does not normalize axis
vectors, construct a coordinate frame, transform UCS/WCS values, validate
orthogonality, or infer a missing component.

## Normative evidence

Autodesk's published HEADER table identifies the two paper-space limit rows,
the paper-space UCS origin and axes, and the model-space UCS origin and axes
with their exact ordered group codes:

<https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-A85E8E67-27CD-4C59-BE61-4DC9FADBE74A.htm>

The physical double representation and component group-code families remain
those audited in M6.5d from Autodesk's group-code value-type and Binary DXF
references. No external implementation source was copied, translated, or
ported.

## Architectural evidence

No change to `header_numeric.rs` is needed. The M6.5c directory enumerates
generated numeric fields, and the M6.5d `Double2`/`Double3` decoder consumes
their generated group-code slices. Adding these rows therefore extends both
ASCII and Binary semantic coverage without a field-specific parser or
accessor.

The schema grows from 17 to 25 fields. Numeric directory entries grow from 14
to 22; the three text/handle rows remain outside it. Existing stable schema
ordinals shift nowhere because the new ids sort after the prior final row.

## Verification coverage

The AC1009-AC1032 ASCII/Binary matrix now contains all eight new variables.
Tests verify:

- generated schema order, ordinals, names, group-code slices, and schema-wide
  field count;
- explicit tuple state and exact binary64 component bits;
- component-level raw provenance for `$UCSORG` in both physical formats;
- a malformed `$UCSXDIR` middle component is invalid while its valid siblings
  remain explicit;
- deterministic code generation and unchanged public directory behavior.

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

The authoritative schema/generator/generated implementation diff adds 187
lines and removes 3. It is slightly below the approximate 200-line checkpoint
target because the existing generic decoder required no production change.
Test-only updates add 80 lines and remove 4. No dependency or unsafe code was
added.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `schema/dxf/v1/header.bootstrap.json` | `16af5389221d8fde80f6a7407b051d72bfccd4cc72b88d128866924a90331cf4` |
| `crates/seacad-schema-gen/src/main.rs` | `b3741c72186e9dcf25312a5facb69fad085708f35c9b38011f62370adf6f24a1` |
| `crates/seacad-dxf-core/src/generated/header_schema.rs` | `0e6281ea84ad044c128293055ba17f6d4a33161b449858e97360bc62172c0d6e` |
| `crates/seacad-dxf-core/src/header_schema_directory.rs` | `85977b37e65d6e0ce8b02a20c0c86e4a63f66b1be466bd3297a59025fda72cf2` |
| `crates/seacad-dxf-core/tests/header_numeric_tests.rs` | `7b2c47de226d4b02c72436e82cf4a2e5de5149f5e9c274ac72011554cc59b388` |
| `docs/IMPLEMENTATION_PLAN.md` | `e5f4227c1f357229024c6c5d3ea172d9816cc8c376ccd473f945b85a76564a56` |
