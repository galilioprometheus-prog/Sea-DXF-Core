# M6.5q Remaining Expanded Numeric HEADER Audit

## Scope

M6.5q appends the final twenty-one numeric field slots planned before the text
and handle inventories:

- `$DIMASSOC` uses group code 280 and signed 16-bit storage;
- `$DIMATFIT`, `$DIMCLRD`, `$DIMCLRE`, `$DIMCLRT`, `$DIMJUST`, `$DIMLWD`,
  `$DIMLWE`, `$DIMTAD`, `$DIMTMOVE`, and `$DIMTOLJ` use group code 70 and
  signed 16-bit storage;
- `$USERI1` through `$USERI5` expand the published `$USERI1 - 5` row as five
  group-code 70 signed 16-bit fields;
- `$USERR1` through `$USERR5` expand the published `$USERR1 - 5` row as five
  group-code 40 double fields.

The checkpoint preserves exact numeric values, typed failures, and source
provenance. It does not interpret dimension behavior, colors, lineweights,
placement policies, user-variable purpose, defaults, ranges, or version
applicability.

## Normative evidence

Autodesk's published HEADER table identifies the eleven dimension variables,
the two five-variable user ranges, and their group codes:

<https://help.autodesk.com/cloudhelp/2026/ENU/AutoCAD-DXF/files/GUID-A85E8E67-27CD-4C59-BE61-4DC9FADBE74A.htm>

The independently normalized 206-row Autodesk 2015, 2018, 2021, 2024, 2025,
and 2026 inventories remain identical, with SHA-256
`d1034c4246758f368ac79982fa1c21d59328851d9fdca8ccebe944f20f71cfaf`.
Expanding the two published five-variable rows accounts for eight additional
field slots, so the eventual inventory is 214 expanded fields.

Autodesk's DXF value-type table assigns group codes 40-59 to double-precision
floating-point values, group codes 60-79 to signed 16-bit integers, and group
codes 280-289 to signed 16-bit integers:

<https://help.autodesk.com/cloudhelp/2026/ENU/AutoCAD-DXF/files/GUID-2553CF98-44F6-4828-82DD-FE3BC7448113.htm>

No external parser implementation or fixture was copied, translated, or
ported.

## Range-row evidence contract

The authoritative source has one row named `$USERI1 - 5` and one row named
`$USERR1 - 5`; inventing five independent source rows for either range would
misstate the evidence. The generator therefore accepts a range-row anchor only
when the field has the same prefix, has a wholly numeric suffix, and that suffix
falls within the inclusive published interval. A focused fail-closed test
accepts `$USERI3` for `row:$USERI1 - 5` while rejecting `$USERI6` and
`$USERJ3`.

Exact single-row anchors retain their prior behavior. This validation is build
tooling only and adds no runtime parser branch.

## Generic decode and physical coverage

All twenty-one fields enter the existing schema-driven numeric directory. No
field-specific decoder, lookup state machine, public value type, dependency,
source limit, writer behavior, or file-sized allocation is added.

Group codes 40 and 70 are representable in every supported Binary header, so
the twenty corresponding fields have ASCII/Binary fixture coverage across
AC1009-AC1032. The pre-R13 Binary group-code header cannot represent group code
280. To preserve equivalent standard fixtures across physical encodings, both
AC1009 fixtures leave `$DIMASSOC` absent; AC1012-AC1032 exercise it in ASCII and
Binary. This is fixture-level physical coverage, not a new schema applicability
claim.

Tests retain exact raw bytes for representative `$DIMASSOC`, `$USERI1`, and
`$USERR5` values; exact Binary `i16::MIN` evidence for `$USERI5`; and exact
Binary IEEE-754 NaN payload bits for `$USERR5`. Wrong group code, out-of-range
ASCII integer, and duplicate variable cases remain typed invalid states with
raw provenance.

## Append-only ordinal contract

The prior 166 field ids remain frozen at ordinals 0-165:

| Ordinal | Field id | DXF name | Group code |
| ---: | --- | --- | ---: |
| 166 | `dimassoc` | `$DIMASSOC` | 280 |
| 167 | `dimatfit` | `$DIMATFIT` | 70 |
| 168 | `dimclrd` | `$DIMCLRD` | 70 |
| 169 | `dimclre` | `$DIMCLRE` | 70 |
| 170 | `dimclrt` | `$DIMCLRT` | 70 |
| 171 | `dimjust` | `$DIMJUST` | 70 |
| 172 | `dimlwd` | `$DIMLWD` | 70 |
| 173 | `dimlwe` | `$DIMLWE` | 70 |
| 174 | `dimtad` | `$DIMTAD` | 70 |
| 175 | `dimtmove` | `$DIMTMOVE` | 70 |
| 176 | `dimtolj` | `$DIMTOLJ` | 70 |
| 177 | `useri1` | `$USERI1` | 70 |
| 178 | `useri2` | `$USERI2` | 70 |
| 179 | `useri3` | `$USERI3` | 70 |
| 180 | `useri4` | `$USERI4` | 70 |
| 181 | `useri5` | `$USERI5` | 70 |
| 182 | `userr1` | `$USERR1` | 40 |
| 183 | `userr2` | `$USERR2` | 40 |
| 184 | `userr3` | `$USERR3` | 40 |
| 185 | `userr4` | `$USERR4` | 40 |
| 186 | `userr5` | `$USERR5` | 40 |

Stable field ids remain the canonical persisted identity. Schema ordinals are
append-only positions and must not replace field ids in durable external data.

## Test-first and verification coverage

Before the manifest changed, the new focused integration test failed because
`DIMASSOC` was absent, and the generator contract reported 166 rather than 187
rows. After appending the manifest, tightening evidence validation, and
regenerating the registry, the same tests passed through the existing generic
runtime decoder.

Tests cover:

- all 187 generated schema rows and 184 numeric directory entries;
- exact values and raw provenance across the complete supported-version matrix;
- deterministic generated output and append-only ordinals;
- range-row evidence acceptance and fail-closed prefix/range rejection;
- wrong group, out-of-range ASCII integer, duplicate variable, signed Binary
  boundary, and exact IEEE-754 payload evidence;
- existing cancellation and public `Copy`/`Send`/`Sync` bounds.

## Clean-architecture evidence

The runtime decoder, public API, module boundaries, `Cargo.lock`, dependencies,
DXF writer, source limits, and schema format remain unchanged. The runtime data
change is localized to the authoritative manifest and deterministic generated
registry. The only handwritten production logic is the generic generator-side
range-row evidence validator; it is independent of DXF decoding and rejects
out-of-range or mismatched expansions.

The authoritative manifest and generated registry add 485 lines and remove 2,
within the approximate 200-500-line production target. The focused numeric test
adds 179 lines and removes 1. No unsafe block, field-specific runtime parser or
lookup branch, `panic!`, `unwrap`, `expect`, `todo!`, or `unimplemented!` was
added.

## Verification result

Rust 1.97.1 passed:

- `cargo deny --locked check`;
- `cargo +1.97.1 fmt --all -- --check`;
- deterministic schema generation in `--check` mode;
- Clippy for the workspace and all targets with warnings denied;
- 247 workspace tests: 202 core unit tests, 12 numeric integration tests,
  16 CLI tests, 6 corpus-receipt tests, and 11 schema-generator tests;
- `git diff --check`.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `schema/dxf/v1/header.bootstrap.json` | `45ff28befc738aecb3587b2e7b4e1771b1086f8681b594dd7b585ff5927c7d2a` |
| `crates/seacad-schema-gen/src/main.rs` | `ed856ca0957d25e39d9e2117e16b7c443f3d6a3295899dc4aa7998a4c83a6ba9` |
| `crates/seacad-dxf-core/src/generated/header_schema.rs` | `9ff1e29409c06731b872727afe4c86920685586f5dc47f8430707730ed3286ac` |
| `crates/seacad-dxf-core/src/header_schema_directory.rs` | `3747a53673b739369617d52defed13ead04c04715079fd5d408c4aa0e2614bc5` |
| `crates/seacad-dxf-core/tests/header_numeric_tests.rs` | `25f14d613a7bc8a9eeab176ae5f16640471dd0fa59f7e5fc9543d0ba618f81da` |
| `docs/IMPLEMENTATION_PLAN.md` | `a5c1914513d8fb271b7f780d2fe5cb50f922370e58cf3321b5acc8b39f9baae1` |