# M6.5k Date and Elapsed-Time HEADER Values Audit

## Scope

M6.5k appends six group-code 40 HEADER fields with reviewed special meaning:

- Julian-date plus fractional-day scalars: `$TDCREATE`, `$TDUCREATE`,
  `$TDUPDATE`, and `$TDUUPDATE`;
- elapsed-day plus fractional-day scalars: `$TDINDWG` and `$TDUSRTIMER`.

The schema distinguishes `JulianDate` and `ElapsedDays` from a generic
`Double`. Each semantic value retains its exact `DxfDouble` IEEE-754 payload,
field provenance, group occurrence, and byte span.

## Normative evidence

Autodesk's HEADER table gives group code 40 and distinguishes local,
universal, cumulative-editing, and user-timer fields:

<https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-A85E8E67-27CD-4C59-BE61-4DC9FADBE74A.htm>

Autodesk's special-handling topic specifies the real-number forms
`<Julian date>.<Fraction of day>` and
`<Number of days>.<Fraction of day>`:

<https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-6942BAF3-095F-4217-9F61-6931975D3A64.htm>

That topic also warns that a DATE value is a true Julian date only when the
system clock is UTC. SeaCad therefore does not infer a timezone, convert local
and universal values, or claim that the stored scalar is UTC. Local/universal
meaning remains attached to the generated field identity.

No parser implementation or date conversion algorithm was copied, translated,
or ported from another project.

## Exactness and fallibility

`DxfJulianDate::raw` and `DxfElapsedDays::raw` expose the original
bit-preserving `DxfDouble`. `day_parts` is a derived, fallible view:

- it uses truncation toward zero, matching Autodesk's published extraction
  expression;
- it returns whole days as `i64` and the computed fraction as `DxfDouble`;
- it returns `None` for non-finite values or a whole-day component outside the
  `i64` domain;
- it performs no calendar, locale, daylight-saving, leap-second, or timezone
  operation.

ASCII non-finite and out-of-range spellings remain typed invalid numeric
values. Binary IEEE payloads, including NaN payload bits, remain explicit raw
values while the derived split fails closed.

## Append-only ordinal contract

The prior 85 field ids remain frozen at ordinals 0-84. The new fields append in
the approved order:

| Ordinal | Field id | DXF name | Storage |
| ---: | --- | --- | --- |
| 85 | `tdcreate` | `$TDCREATE` | `JulianDate` |
| 86 | `tducreate` | `$TDUCREATE` | `JulianDate` |
| 87 | `tdupdate` | `$TDUPDATE` | `JulianDate` |
| 88 | `tduupdate` | `$TDUUPDATE` | `JulianDate` |
| 89 | `tdindwg` | `$TDINDWG` | `ElapsedDays` |
| 90 | `tdusrtimer` | `$TDUSRTIMER` | `ElapsedDays` |

Stable field ids remain the canonical persisted identity; ordinals are only
append-only positions in this schema generation.

## Resource and verification contract

The six fields reuse the bounded generic scalar decoder. Construction remains
O(F) in generated field count, and no file-size, record, diagnostic, or value
limit changes. Tests cover every AC1009-AC1032 dialect in ASCII and Binary,
exact raw provenance, storage-type separation, malformed ASCII, binary NaN
payload preservation, fractional-day splitting, negative truncation, and
`i64`/finite failure boundaries.

Final gate results and artifact SHA-256 receipts are appended after the
checkpoint verification run.

## Verification result

Rust 1.97.1 passed:

- `cargo fmt --all -- --check`;
- deterministic schema generation in `--check` mode;
- Clippy for the workspace and all targets with warnings denied;
- 229 workspace tests: 200 core unit tests, 7 numeric integration tests,
  12 CLI tests, and 10 schema-generator tests;
- `git diff --check`.

The code/schema/generated production diff adds 328 lines and removes 17,
within the approximate 200-500-line production target. Test-only assertions
and fixtures add 142 lines and remove 5. No dependency, unsafe block,
field-specific decode state machine, `panic!`, `unwrap`, `expect`, `todo!`, or
`unimplemented!` was added.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `schema/dxf/v1/header.bootstrap.json` | `520a30808a0193b3da4b5afff5ac446e7a402148c9e82e57b078c125ca979d57` |
| `crates/seacad-schema-gen/src/main.rs` | `bde21e79785bd6a884d0b23b0d5a8ba49cc4993601833135d6ea11a8040c0709` |
| `crates/seacad-dxf-core/src/generated/header_schema.rs` | `2b1491b2055f0011620ede8983fe7773200a9118a9edd58e09d82fd616f0639f` |
| `crates/seacad-dxf-core/src/header_numeric.rs` | `9f481cb7adc39265c6eb54686ed25717ebe6acad1f06bab1a9a84e6cae85ba0e` |
| `crates/seacad-dxf-core/src/header_schema_directory.rs` | `6bfdbb5ec7adbe9583697c06e1f7fb24f51b6563e62f8bb678e9c94d234cca5a` |
| `crates/seacad-dxf-core/src/lib.rs` | `2612fd6db455a0b386e5f68b7a6c5a408db9668944602ca08afcbd77c5893873` |
| `crates/seacad-dxf-core/tests/header_numeric_tests.rs` | `4e05a160dbe6406b734ab388059c9adbc545b7a87254a8b04601d1f071aae1ec` |
| `docs/IMPLEMENTATION_PLAN.md` | `3085c6d6e0adb8a1755074083d22840a7a8541e5c657afb0cdb9d74bcab59449` |
