# M14.3cp Entity XDATA Point Tuples

Retrieved: 2026-08-04

## Normative basis

Autodesk's *About Extended Data (DXF)* specifies four three-real XDATA
families. Codes 1010/1020/1030 are an application-defined point or vector;
1011/1021/1031 are a transformed world-space position;
1012/1022/1032 are a transformed world-space displacement; and
1013/1023/1033 are a transformed world direction. Each family is stored in
X/Y/Z order, and XDATA order is significant:

<https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-A2A628B0-3699-4740-A215-C560E7242F63.htm>

Autodesk separately documents that the 16-KiB XDATA limit is per entity and
can be shared by multiple applications. It is not a per-APPID allowance:

<https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-AutoLISP/files/GUID-A94BC605-5517-437F-A6FE-D3EB8116A01A.htm>

## Contract

`DxfEntityXDataPointTupleDirectory` owns the generic typed XDATA directory and
publishes one maximal candidate for each immediately adjacent, same-family,
same-context component run. A complete candidate contains X, Y, and Z in
source order. Missing, reordered, duplicated, mismatched-family, and orphan
components remain partial candidates rather than being discarded or silently
paired across unrelated values.

Each axis is a compact private-constructor member handle resolving to the exact
typed entry. Resolution also validates the owning tuple, so foreign or forged
tuple/member relationships fail closed. A normal DXF group gap, new group-1001
application, entity boundary, or source boundary always ends a candidate.
Numeric validity remains on the typed member: structural completeness never
upgrades an invalid double to a usable point.

## Verification boundary

The focused suite pairs ASCII and Binary fixtures for AC1009, AC1012, AC1014,
AC1015, AC1018, AC1021, AC1024, AC1027, and AC1032. It covers all four complete
families, multiple applications and entities, every partial axis pattern used
by malformed ordering, duplicated axes, invalid numbers, orphans, normal-group
gaps, application/entity/source boundaries, cancellation, stable lookup,
metadata bounds, and non-disclosing debug output.

This checkpoint does not apply world-coordinate transformations, resolve
group-1003 layer names, calculate Autodesk `xdsize` or enforce the per-entity
16-KiB limit, assign application-specific payload meaning, resolve or remap
group-1005 targets, clone/write XDATA, or advance any entity to `Complete`.

## Gate receipts

The focused point-tuple suite passed 3/3 tests and the full workspace passed
all 983 listed tests across 184 targets. Generated schema and release-evidence
checks, `cargo deny --locked check`, formatting, workspace Clippy with warnings
denied, the production forbidden-construct scan, and `git diff --check` all
passed. Production changes are 525 insertions; the focused test target adds 459
lines. No manifest, lockfile, dependency, generated schema, locale source,
committed fixture, or external corpus changed. Four earlier XDATA audit
nonclaims were corrected from per-application to per-entity 16-KiB scope. This
audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 431 | `6046f11b6b379dedfd905654245b8219ada5aa1d87bc6619e06ee0052d0b749f` |
| `crates/seacad-dxf-core/src/entity_xdata_point_tuple.rs` | 520 | `05f3ef6aab517ac9e8413b21218687e0252b8e1b5b5d2bb400bca809ccc7fdda` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,095 | `afe4d635b657829af57b8d8d95d20953010b1e18f8513791920ad2cf33a09ce1` |
| `crates/seacad-dxf-core/tests/entity_xdata_point_tuple_tests.rs` | 459 | `72b8adbb7a07538f35a7ce8ad092c3bca92c563c781ca9b8c33fe06f0abef740` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,388 | `dc12af3b7a4dfa4f5b29ddf43560d7be366c65aaeeed7e235952484cf3165694` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,808 | `6135ac24a6e278893b863e2612316d13cc9a08e2baf31fbb110e25a1ca981454` |
| `docs/SUPPORT_MATRIX.md` | 2,450 | `da8a88a6734578a6ea3575aa424d2dde2bf715ce17480b179f13a9b62552a448` |
