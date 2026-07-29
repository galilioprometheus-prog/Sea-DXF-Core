# M9.1b LWPOLYLINE Integer Evidence

Retrieved: 2026-07-29

## Normative boundary

- Autodesk [LWPOLYLINE (DXF)](https://help.autodesk.com/cloudhelp/2016/ENU/AutoCAD-DXF/files/GUID-748FC305-F3F2-4F74-825A-61F04D757A50.htm)
  identifies group `90` as number of vertices, group `70` as bit-coded
  polyline flags, and repeated group `91` as vertex identifiers.
- Autodesk [Group Code Value Types Reference (DXF)](https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-2553CF98-44F6-4828-82DD-FE3BC7448113.htm)
  defines groups `60..79` as 16-bit integer values and groups `90..99` as
  32-bit integer values.

M9.1b preserves those signed physical domains and all occurrences. Entity-level
constraints, bit meanings, and relationships between values are not inferred
from the wire type.

## Implementation contract

`crates/seacad-dxf-core/src/lightweight_polyline_integer.rs` provides:

- exact uppercase `LWPOLYLINE` recognition in complete `BLOCKS` and `ENTITIES`;
- distinct vertex-count, flags, and vertex-identifier roles;
- a tagged `I16`/`I32` value that cannot erase the documented wire domain;
- source-order occurrence slices with exact group and value spans;
- exact ASCII lexical failures and little-endian Binary values;
- duplicate and empty slices without canonical selection;
- binary-search lookup by raw-record ordinal and source group occurrence;
- shared raw-document, ASCII-document, and Binary-document adapters;
- explicit cancellation and fallible allocation behavior.

`ascii_numeric.rs` now uses one bounded signed parser for both i16 and i32.
`raw_double.rs` exposes internal bounded ASCII/fixed-payload helpers reused by
the new `raw_integer.rs`, so cancellation and resource handling do not diverge.

## Test evidence

`crates/seacad-dxf-core/tests/lightweight_polyline_integer_tests.rs` covers:

- ASCII/Binary parity for all supported AC1009-AC1032 physical dialects;
- every documented integer role and repeated vertex identifiers;
- exact `i16::MIN/MAX` and `i32::MIN/MAX` boundaries;
- syntax and range failures in both integer domains;
- duplicate and empty integer evidence;
- exact case-sensitive matching and section scoping;
- record/group lookup, cancellation, and public traits.

Focused regression tests also exercise all existing raw-double geometry
directories after the bounded numeric-reader refactor. The dialect parity
fixtures are physical decoding evidence only and do not claim LWPOLYLINE
version applicability.

## Non-claims

M9.1b does not select canonical values; require non-negative count or identifier
values; interpret flag bits; associate identifiers with floating vertex groups;
compare declared and observed vertex counts; validate order, cardinality,
uniqueness, version applicability, or geometry; apply defaults; transform OCS;
assemble segments; edit/write; render; snap; or infer topology.

## Required checkpoint gates

- `cargo deny --locked check`
- `cargo +1.97.1 fmt --all -- --check`
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`
- `cargo +1.97.1 clippy --locked --workspace --all-targets -- -D warnings`
- `cargo +1.97.1 test --locked --workspace`
- `git diff --check`

All checkpoint gates passed on 2026-07-29:

- dependency policy: advisories, bans, licenses, and sources all `ok`;
- formatting and generated-schema drift checks passed;
- workspace Clippy passed with warnings denied;
- workspace tests: 342 passed, 0 failed, 0 ignored, including all 4 focused
  M9.1b integration tests and the new signed-i32 parser unit test;
- `git diff --check` passed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/ascii_numeric.rs` | 172 | `5c3c6147163e8f10ad27b3e48a70deb56b9e4edfdf6b4308a09871317e996538` |
| `crates/seacad-dxf-core/src/raw_double.rs` | 124 | `0fd09c856ccac16fb1b4890e608d2b57659af7ad060e30c6ea2c57e4344bb505` |
| `crates/seacad-dxf-core/src/raw_integer.rs` | 42 | `4c08b68765aedb6169edd46678eabba85e28e87307350dc244ae34e75c94987d` |
| `crates/seacad-dxf-core/src/lightweight_polyline_integer.rs` | 335 | `ff97358c7cef30b95b675fe8ededdf6a35d9a74794e8d423a2265843fbfc0601` |
| `crates/seacad-dxf-core/src/lib.rs` | 275 | `a0c9fb1ec07079c9101124817d2e5572fb3ef34cc77e089d215bfd870bc55025` |
| `crates/seacad-dxf-core/tests/lightweight_polyline_integer_tests.rs` | 295 | `d2f028e393beada59b93966201efdc5f8cea242fc2eda998ee42b0ae1cb60d23` |
| `docs/IMPLEMENTATION_PLAN.md` | 414 | `a213a3006ba54c7d6bbd8220cedc72c9ba7e41fed2bc80d7bd0fad1ce725e787` |
| `docs/SUPPORT_MATRIX.md` | 326 | `ef8eb977e381a0db6f9e3068c5bdae8667323a1f44b50e086d4e682a176a1be2` |
