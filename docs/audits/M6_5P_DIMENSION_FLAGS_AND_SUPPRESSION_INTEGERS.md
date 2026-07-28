# M6.5p DIMENSION Flag and Suppression Integer HEADER Audit

## Scope

M6.5p appends sixteen reviewed `$DIM*` HEADER fields. Every field is one
group-code 70 signed 16-bit integer:

- `$DIMALT`, `$DIMASO`, `$DIMLIM`, `$DIMSAH`, `$DIMSD1`, `$DIMSD2`;
- `$DIMSE1`, `$DIMSE2`, `$DIMSHO`, `$DIMSOXD`, `$DIMTIH`, `$DIMTIX`;
- `$DIMTOFL`, `$DIMTOH`, `$DIMTOL`, and `$DIMUPT`.

The checkpoint preserves the exact source integer and provenance only. It does
not reinterpret a nonzero value as Boolean, apply suppression or placement
behavior, supply defaults, validate ranges, infer version applicability, or
combine related dimension variables. In particular, Autodesk marks `$DIMASO`
obsolete; SeaCad retains it independently and does not merge it with or use it
as a fallback for `$DIMASSOC`.

## Normative evidence

Autodesk's published HEADER table identifies every selected variable and group
code 70:

<https://help.autodesk.com/cloudhelp/2026/ENU/AutoCAD-DXF/files/GUID-A85E8E67-27CD-4C59-BE61-4DC9FADBE74A.htm>

The independently normalized 206-row Autodesk 2015, 2018, 2021, 2024, 2025,
and 2026 inventories remain identical, with SHA-256
`d1034c4246758f368ac79982fa1c21d59328851d9fdca8ccebe944f20f71cfaf`.
Sixty-eight rows begin with `$DIM`; these sixteen reviewed rows use group code
70.

Autodesk's DXF file value-type table assigns group codes 60-79 to signed
16-bit integer values:

<https://help.autodesk.com/cloudhelp/2026/ENU/AutoCAD-DXF/files/GUID-2553CF98-44F6-4828-82DD-FE3BC7448113.htm>

The ASCII numeric grammar, Binary little-endian representation, and bounded
semantic reads remain those audited in M6.5a and M6.5b. No external parser
implementation or fixture was copied, translated, or ported.

## Generic decode and physical coverage

All sixteen fields enter the existing schema-driven numeric directory. No
field-specific parser, lookup state machine, public value type, dependency,
source limit, writer behavior, or file-sized allocation is added.

Group code 70 is representable by the one-byte pre-R13 Binary group-code
header. The standard fixture therefore exercises every new field in both ASCII
and Binary for all supported AC1009-AC1032 versions.

Each explicit value retains its exact `i16`, field provenance, group
occurrence, value byte span, and original raw bytes. ASCII lexical or range
failure and structural conflicts remain typed invalid states. Binary signed
boundary values remain exact raw integers. Fixture values deliberately include
`-1`, `2`, and larger nonzero values to prove that the wire integers are not
silently narrowed to Boolean.

## Append-only ordinal contract

The prior 150 field ids remain frozen at ordinals 0-149:

| Ordinal | Field id | DXF name |
| ---: | --- | --- |
| 150 | `dimalt` | `$DIMALT` |
| 151 | `dimaso` | `$DIMASO` |
| 152 | `dimlim` | `$DIMLIM` |
| 153 | `dimsah` | `$DIMSAH` |
| 154 | `dimsd1` | `$DIMSD1` |
| 155 | `dimsd2` | `$DIMSD2` |
| 156 | `dimse1` | `$DIMSE1` |
| 157 | `dimse2` | `$DIMSE2` |
| 158 | `dimsho` | `$DIMSHO` |
| 159 | `dimsoxd` | `$DIMSOXD` |
| 160 | `dimtih` | `$DIMTIH` |
| 161 | `dimtix` | `$DIMTIX` |
| 162 | `dimtofl` | `$DIMTOFL` |
| 163 | `dimtoh` | `$DIMTOH` |
| 164 | `dimtol` | `$DIMTOL` |
| 165 | `dimupt` | `$DIMUPT` |

Stable field ids remain the canonical persisted identity. Schema ordinals are
append-only positions and must not replace field ids in durable external data.

## Test-first and verification coverage

Before the manifest changed, the new focused integration test failed because
`DIMALT` was absent, and the generator contract reported 150 rather than 166
rows. After the manifest was appended and the registry regenerated, the same
tests passed through the existing generic decoder without handwritten
production changes.

Tests cover:

- all 166 generated schema rows and 163 numeric directory entries;
- exact values and raw provenance for every new field across the complete
  AC1009-AC1032 ASCII/Binary matrix;
- append-only ordinals and deterministic generated output;
- a wrong group code, out-of-range ASCII integer, and duplicate variable with
  exact conflict evidence;
- exact Binary `i16::MIN` value and provenance for `$DIMUPT`;
- non-Boolean signed values, existing cancellation, and public
  `Copy`/`Send`/`Sync` bounds.

## Clean-architecture evidence

The milestone changes no handwritten decoder, parser branch, module boundary,
or public API. The only production data changes are the authoritative manifest
and its deterministic generated registry. This demonstrates that adding a
reviewed scalar field remains localized to schema data rather than expanding
`header_numeric.rs`.

`ponytail-review` returned `Lean already. Ship.`: it found no removable
abstraction or hand-written duplication in the diff. Complexity, module-cycle,
SplitRS, mutation, and semver refactor gates are not applicable because no
handwritten production function, module boundary, dependency, or public API
changed. Test efficacy is instead proven by the recorded red-to-green schema
and integration baselines.

## Verification result

Rust 1.97.1 passed:

- `cargo fmt --all -- --check`;
- deterministic schema generation in `--check` mode;
- Clippy for the workspace and all targets with warnings denied;
- 239 workspace tests: 202 core unit tests, 11 numeric integration tests,
  16 CLI tests, and 10 schema-generator tests;
- `git diff --check`.

The authoritative manifest/generated production registry diff adds 370 lines
and removes 2, within the approximate 200-500-line production target.
Test-only updates add 152 lines and remove 6. No dependency, public API,
handwritten production decoder, unsafe block, field-specific parser/lookup
branch, `panic!`, `unwrap`, `expect`, `todo!`, or `unimplemented!` was added.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `schema/dxf/v1/header.bootstrap.json` | `8e8e05a4403897d3e9c79df095f1e0c317a22b3f87cc8467fbdae4e9b53c4a36` |
| `crates/seacad-schema-gen/src/main.rs` | `d9fa1f468b8a7507e78b083913aa3c59f4b2c368cbcae81b123162e5f3bf646b` |
| `crates/seacad-dxf-core/src/generated/header_schema.rs` | `fbdd9a694018f3bfad55adbf587e92c1048c97e261bcaec7da7ed5d38dcfcd55` |
| `crates/seacad-dxf-core/src/header_schema_directory.rs` | `611ad95bc573819abe92fd76d104a38f4c0bec7dcb6709526e7abb28016badca` |
| `crates/seacad-dxf-core/tests/header_numeric_tests.rs` | `a155ecacf42559a93fc9704a1c8cbeb95de9701123061b7d32ee5595760be569` |
| `docs/IMPLEMENTATION_PLAN.md` | `42bef4761ca60ae1ad1d6b2e461c760eb52e6256935e03ff604718e77bfa1fd8` |
