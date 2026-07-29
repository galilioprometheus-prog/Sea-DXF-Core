# M6.7 HEADER evidence closure

M6.7 closes SeaCad's documented HEADER inventory at 206 Autodesk source rows
and 214 expanded schema slots. It adds no production parser branch or public
API. The checkpoint extends the existing format-neutral tests so every typed
directory is exercised across all nine supported dialects in both ASCII and
Binary DXF.

## Inventory accounting

The independently normalized Autodesk 2015, 2018, 2021, 2024, 2025, and 2026
HEADER inventories remain identical at 206 rows, with source-facts SHA-256
`d1034c4246758f368ac79982fa1c21d59328851d9fdca8ccebe944f20f71cfaf`.
The published `$USERI1 - 5` and `$USERR1 - 5` range rows expand by eight
additional field slots, producing 214 append-only schema entries.

The generated directory partitions those slots without overlap:

| Wire directory | Slots | Evidence |
| --- | ---: | --- |
| Numeric | 184 | Signed integers, strict Booleans, exact IEEE-754 scalars/tuples, and date/elapsed-day wrappers |
| Exact text | 25 | Exact source spans under the existing replacement-free encoding policy |
| Handle | 5 | Exact 1-to-16-digit hexadecimal spelling and parsed 64-bit identifiers |
| Total | 214 | Every schema ordinal `0..213` |

The source registry, manifest, bootstrap schema, generated registry, and
append-only ordinals are unchanged by M6.7. The normalized input receipt stays
`ff28e978910e9b9995afa6c10e6399e8a1e5b532fbc85cf6c25f8ce17da4ea4f`.

## Physical and dialect matrix

`DxfAcadVersion::SUPPORTED` contains AC1009, AC1012, AC1014, AC1015, AC1018,
AC1021, AC1024, AC1027, and AC1032. Each directory test now opens a distinct
ASCII and Binary fixture for every version, yielding eighteen physical/dialect
combinations per wire family.

All text group codes are representable in the pre-R13 one-byte Binary header,
so all 25 text entries are explicit in every combination. The existing numeric
matrix retains its reviewed physical rule: AC1009 fixtures omit group codes
above 255 from both formats. M6.7 applies the same rule to `$CEPSNID`,
`$DRAGVS`, `$INTERFEREOBJVS`, and `$INTERFEREVPVS`; `$HANDSEED` remains explicit
in AC1009. AC1012-AC1032 fixtures exercise all five handles in both formats.

Using matching ASCII and Binary absence for unencodable AC1009 group codes
preserves parity without inventing an applicability rule. Defaults, enums,
ranges, units, filesystem behavior, GUID validation, symbol lookup, pointer
resolution, ownership, and object topology remain unclaimed.

## Test-first result

Before M6.7, numeric parity already covered all nine versions, while the text
and handle integration fixtures were fixed to AC1032. The focused test change
first exposed that the legacy fixture needed a reviewed `ANSI_1252` declaration
for replacement-free decoding. With that source declaration and the correct
one-byte AC1009 Binary group-code framing, both directories pass the full
matrix while preserving all prior failure-state, cancellation, source-identity,
trait-bound, and redacted-debug coverage.

## Scope and architecture

Only integration tests and checkpoint documentation change. Runtime modules,
generated code, schema JSON, public API, source limits, dependencies,
`Cargo.lock`, writer behavior, and support for record/entity semantics remain
unchanged. No external parser, fixture, or implementation was copied,
translated, or ported.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `crates/seacad-dxf-core/tests/header_numeric_tests.rs` | `25f14d613a7bc8a9eeab176ae5f16640471dd0fa59f7e5fc9543d0ba618f81da` |
| `crates/seacad-dxf-core/tests/header_text_tests.rs` | `3ee1a3cd9606807d7ab201f66d1074e8d8c48a7387f365b637a34d4b06f22596` |
| `crates/seacad-dxf-core/tests/header_handle_tests.rs` | `f20acbb84d65e08842950ed0fa747fffd6140ac11c8ef1b79b83b1ba84ea73d6` |
| `schema/dxf/v1/header.bootstrap.json` | `8a871216ac42ccbccf86d6c13bcb469ba9b7bc55b2a749c6bbe82a5f0de1c5d2` |
| `crates/seacad-dxf-core/src/generated/header_schema.rs` | `1f93ce236436de7bbbebf0a17377b5a8f22f71e99b25c0f2c0960a26ef4915cb` |
| `docs/IMPLEMENTATION_PLAN.md` | `463164a0f24ee0560b57a1e4852207a5dd06fc8a51d77e01903670a282053243` |
| `docs/SUPPORT_MATRIX.md` | `68ab56e597f0b12b9ebaa9fa7d9ed495f68900167f93a1281f1e258f5711cc7f` |

The unchanged schema and generated-registry hashes demonstrate that M6.7
closes evidence over the existing 214-field contract rather than changing it.

## Required gates

Rust 1.97.1 passed:

- `cargo deny --locked check` (advisories, bans, licenses, and sources);
- `cargo +1.97.1 fmt --all -- --check`;
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`;
- `cargo +1.97.1 clippy --workspace --all-targets -- -D warnings`;
- `cargo +1.97.1 test --workspace`: 257 passed, comprising 202 core unit
  tests, four handle integration tests, 12 numeric integration tests, four
  text integration tests, 16 CLI tests, six corpus-receipt tests, and 13
  schema-generator tests;
- `git diff --check`.

The focused HEADER command passed all 20 integration tests. Before the patch,
Antigravity Agent Hub task 16 independently established the clean baseline at
HEAD `9d0eb530b2b0f65de28781a3fdb1958b8710112e`: the schema check and the same
20 tests passed with no repository mutation. Codex reproduced that baseline
before accepting the task and then ran every final gate above after the change.
