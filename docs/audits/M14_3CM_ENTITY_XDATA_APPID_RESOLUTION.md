# M14.3cm Entity XDATA APPID Resolution

Retrieved: 2026-08-03

## Normative basis

Autodesk's *About Extended Data (DXF)* states that group 1001 starts each
registered-application XDATA list, that list order is significant, and that
the application name corresponds to an entry in the APPID symbol table:

<https://help.autodesk.com/cloudhelp/2016/ENU/AutoCAD-DXF/files/GUID-A2A628B0-3699-4740-A215-C560E7242F63.htm>

Autodesk's APPID table reference identifies group 2 on an APPID record as the
user-supplied application name:

<https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-6E3140E9-E560-4C77-904E-480382F0553E.htm>

## Contract

- `DxfNamedSymbolTableKind::AppId` admits an exact `APPID` record only while
  scanning an exact APPID table in TABLES and publishes pending records only
  when that table receives an exact `ENDTAB` boundary.
- A record needs exactly one group-2 name outside group-102 application-control
  content. Duplicate group-2 occurrences exclude that record.
- `DxfEntityXDataAppIdResolutionDirectory` owns both source-bound evidence
  directories and publishes one result for every XDATA application in source
  order.
- Candidate selection uses a sorted SHA-256 digest index. Digest equality is
  never treated as name equality: the authoritative source spans must also be
  byte-for-byte equal.
- Results are `Missing`, `Unique { target }`, or
  `Ambiguous { target_count }`. No case folding, trimming, decoding,
  normalization, or first-match selection occurs.
- Construction and exact comparisons are cancellation-aware, allocation is
  fallible, foreign-source queries fail typed, public ordinals are bounded,
  and `Debug` output contains metadata rather than application bytes.

## Verification boundary

The focused three-test suite covers all nine supported versions in paired
ASCII and Binary. It proves exact unique resolution, two-target ambiguity,
missing and case-near names, wrong-table exclusion, malformed/lowercase table
declaration exclusion, and fail-closed unclosed tables. Separate assertions
cover source identity, foreign-source rejection, cancellation, public lookup
bounds, entry size, and non-disclosing debug output.

## Nonclaims

This checkpoint does not validate the APPID/application name syntax or 31-byte
limit, group-1002 brace balance, XDATA value domains, the separate 16-KiB
application limit, application payload meaning, group-1005 handle resolution
or remap, XDATA clone/write, or POINT/entity `Complete` support.

## Verification

The focused APPID-resolution suite passed 3/3 tests. The first non-quiet cold
workspace run reached its 180-second wrapper limit after only passing output;
the cache-hot quiet retry passed all 974 listed tests in 47.2 seconds. The
initial sandboxed `cargo deny` attempt could not lock the read-only advisory
cache; the same exact command with approved cache access passed all four
policy categories. Generated schema and release-evidence checks, formatting,
workspace Clippy with warnings denied, the production forbidden-construct
scan, and `git diff --check` all passed. Production changes are 271 insertions
and two deletions; the focused test target adds 283 lines. No manifest,
lockfile, dependency, generated schema, locale source, committed fixture, or
external corpus changed. This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 411 | `339934fc623630b13f0ba049683fa19dcba804429cdf4014b16a13aaee7413db` |
| `crates/seacad-dxf-core/src/entity_common_text_semantic.rs` | 745 | `2b911c5809a82c7cda6a79bf8b1e65a237f0a6460f7249357b72a128f054585a` |
| `crates/seacad-dxf-core/src/entity_xdata_appid_resolution.rs` | 260 | `58550311fe06093cf481288f1accb7af9c317806811d5d072e366b372442c14b` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,078 | `e5d8367c0cf7214a721a32b72d56c03c7942f072473f6e8f5d4d2d6142e37a69` |
| `crates/seacad-dxf-core/src/named_symbol_table.rs` | 276 | `57b53e542971ead68f0d789bb7f484be3708f533af5c545354e6c96fe747528c` |
| `crates/seacad-dxf-core/tests/entity_xdata_appid_resolution_tests.rs` | 283 | `bbeaa2c2a9c37947c93c2a6828d2af28baad8c506cfa0a83efa3d56f810fcb83` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,349 | `7fec4c8d82eb7e9adf35955214adda4248d8955eb4ca1c20f3a2d6004bd48797` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,773 | `9570639f9f4126db05fcb2b0d8f844aed8d9bd52f2afdd348bc346e9ed89123f` |
| `docs/SUPPORT_MATRIX.md` | 2,419 | `afdb1aa1da4c5680b6ed42a246ae0573d8eca46faa830c0abd6db97a8457c69e` |
