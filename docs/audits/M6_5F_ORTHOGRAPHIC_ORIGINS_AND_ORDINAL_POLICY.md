# M6.5f Orthographic Origins and Ordinal Policy Audit

## Scope

M6.5f adds twelve shape-reviewed `Double3` HEADER rows:

- paper space: `$PUCSORGBACK`, `$PUCSORGBOTTOM`, `$PUCSORGFRONT`,
  `$PUCSORGLEFT`, `$PUCSORGRIGHT`, and `$PUCSORGTOP`;
- model space: `$UCSORGBACK`, `$UCSORGBOTTOM`, `$UCSORGFRONT`, `$UCSORGLEFT`,
  `$UCSORGRIGHT`, and `$UCSORGTOP`.

Every row uses ordered group codes 10, 20, and 30. This checkpoint preserves
the exact source tuple only; it does not switch UCS, compute a transform,
interpret `UCSBASE`/`PUCSBASE`, fill a world default, or validate geometric
relationships.

## Normative evidence

Autodesk's published HEADER table lists all six paper-space and six model-space
orthographic origin variables with group codes 10, 20, and 30:

<https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-A85E8E67-27CD-4C59-BE61-4DC9FADBE74A.htm>

The `Double3` physical representation and component provenance rules remain
those audited in M6.5d. No external parser implementation was copied,
translated, or ported.

## Append-only ordinal decision

The public directory exposes both a stable field id and a schema ordinal.
Lexicographically inserting the new `PUCSORG*` and `UCSORG*` ids would have
shifted previously published ordinals 20-24. M6.5f instead makes manifest array
order authoritative:

- existing ordinals 0-24 remain byte-for-byte ordered as before;
- new fields append at ordinals 25-36;
- field ids and DXF names remain validated lowercase/uppercase canonical and
  globally unique, but field ids are no longer required to be alphabetically
  sorted;
- generated Rust preserves manifest order deterministically;
- API documentation directs persisted references to the stable field id while
  describing ordinal as an append-only position.

Tests freeze the old tail (`pucsxdir` at 20 and `ucsydir` at 24) and the first
new row (`pucsorgback` at 25). The complete numeric directory expectation
freezes every current numeric ordinal.

## Semantic and bounds evidence

No field-specific parser branch is added. Existing schema-wide matching and the
generic `Double3` decoder automatically produce component-wise
`DxfSemanticValue` entries for ASCII and Binary documents. Construction remains
O(F) in generated field count and each new tuple reads at most three bounded
payloads with cancellation checks.

## Verification coverage

The AC1009-AC1032 ASCII/Binary matrix includes all twelve new rows. Tests verify
schema count 37, numeric count 34, unchanged old ordinals, appended new
ordinals, exact values and raw provenance for `$UCSORGTOP`, and independent
ASCII numeric failure for the middle `$PUCSORGLEFT` component.

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

The authoritative schema/generator/generated/core documentation diff adds 287
lines and removes 12, within the approximate 200-500-line production target.
Test-only updates add 116 lines and remove 5. No dependency, unsafe block,
`panic!`, `unwrap`, `expect`, `todo!`, or `unimplemented!` was added.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `schema/dxf/v1/header.bootstrap.json` | `0321cd88c8c42a062fbed4b5302adea56f2cd63e9b2b78b3d0d69bfc5b108c11` |
| `crates/seacad-schema-gen/src/main.rs` | `401eeec83c2c688ecfa97b4b391b6f9bdde7fab6f4a7afb86da887237313c3e7` |
| `crates/seacad-dxf-core/src/generated/header_schema.rs` | `f3b6586abcff1fe7103daff817392f4b91191c232a971fcb238236ef534b6df8` |
| `crates/seacad-dxf-core/src/header_numeric.rs` | `6909ce020866ff9abc02cbc0e51e062e7174e165f35b0727267c81f1ba955839` |
| `crates/seacad-dxf-core/src/header_schema_directory.rs` | `dd57df8d0b05c2ee3ffe2d620f6a57302eca31c46f8f09c585d3d92ec6a1a78f` |
| `crates/seacad-dxf-core/tests/header_numeric_tests.rs` | `0d37438aa76be10c9dcf781ac6b4122254d35963625ca04310bb110b9a0ef7f4` |
| `docs/IMPLEMENTATION_PLAN.md` | `d8f3455adbb00557e5cf28fd4d91a60db0ceb021fa5cf35e43a6f6ace0cdd565` |
